from __future__ import annotations

import hashlib
import json
import re
from dataclasses import asdict, dataclass, field
from datetime import datetime
from pathlib import PurePosixPath
from typing import Any
from urllib.parse import urlparse


# 2026-03-27: M3-1 首轮先定义统一披露数据契约，目的是让 A 股/港股公告源先共用同一套内部对象。
@dataclass(slots=True)
class DisclosureAttachment:
    name: str
    url: str
    media_type: str | None = None


# 2026-03-27: M3-1 需要保留原始来源快照定位信息，便于后续离线回放和结构漂移验证。
@dataclass(slots=True)
class SourceSnapshot:
    source_name: str
    source_url: str
    storage_path: str
    content_type: str | None = None
    sha256: str | None = None


# 2026-03-27: M3-1 统一事件对象只放首版真正会消费的字段，避免过早把抓取细节泄漏到上层。
@dataclass(slots=True)
class DisclosureEvent:
    event_id: str
    market: str
    exchange: str
    ticker_raw: str
    ticker_normalized: str
    issuer_name: str
    title: str
    category: str
    published_at: str
    document_date: str | None
    language: str | None
    source_name: str
    source_url: str
    document_url: str | None
    document_type: str | None
    content_text: str | None
    dedupe_key: str
    ingested_at: str
    attachments: list[DisclosureAttachment] = field(default_factory=list)
    snapshot: SourceSnapshot | None = None

    def to_record(self) -> dict[str, Any]:
        # 2026-03-27: SQLite 先采用 JSON 文本列落盘，减少首轮 schema 演进成本。
        record = asdict(self)
        record["attachments_json"] = json.dumps(
            record.pop("attachments"), ensure_ascii=False, sort_keys=True
        )
        record["snapshot_json"] = json.dumps(
            record.pop("snapshot"), ensure_ascii=False, sort_keys=True
        )
        return record

    @classmethod
    def from_record(cls, record: dict[str, Any]) -> "DisclosureEvent":
        # 2026-03-27: 回读时恢复为强类型对象，避免上层直接处理裸 dict。
        attachments_payload = json.loads(record.pop("attachments_json") or "[]")
        snapshot_payload = json.loads(record.pop("snapshot_json") or "null")
        return cls(
            **record,
            attachments=[DisclosureAttachment(**item) for item in attachments_payload],
            snapshot=SourceSnapshot(**snapshot_payload) if snapshot_payload else None,
        )


def normalize_disclosure_ticker(market: str, ticker_raw: str) -> str:
    # 2026-03-27: 先把 A 股/港股常见公告代码统一成稳定符号，后续 parser 和去重都依赖它。
    compact = re.sub(r"[^0-9A-Za-z.]", "", ticker_raw or "").upper()
    if not compact:
        raise ValueError("ticker_raw must not be empty")

    if compact.endswith((".SH", ".SZ", ".HK", ".BJ")):
        code, suffix = compact.rsplit(".", 1)
        if suffix == "HK":
            return f"{code.zfill(4)}.{suffix}"
        return f"{code.zfill(6)}.{suffix}"

    market_key = (market or "").strip().upper()
    if market_key in {"CN-SH", "SH", "SSE", "CN-A-SH"}:
        return f"{compact.zfill(6)}.SH"
    if market_key in {"CN-SZ", "SZ", "SZSE", "CN-A-SZ"}:
        return f"{compact.zfill(6)}.SZ"
    if market_key in {"CN-BJ", "BJ", "BSE", "CN-A-BJ"}:
        return f"{compact.zfill(6)}.BJ"
    if market_key in {"HK", "HKEX", "HK-SEHK"}:
        return f"{compact.zfill(4)}.HK"

    raise ValueError(f"Unsupported disclosure market '{market}' for ticker '{ticker_raw}'")


def build_disclosure_dedupe_key(
    market: str,
    exchange: str,
    ticker_normalized: str,
    title: str,
    published_at: str,
    document_url: str | None,
) -> str:
    # 2026-03-27: 首版 dedupe key 采用“代码 + 标题 + 小时桶 + 文件名”组合，先保证稳和可解释。
    normalized_title = _normalize_title_for_dedupe(title)
    published_bucket = _normalize_published_bucket(published_at)
    document_name = _normalize_document_name(document_url)
    raw_key = "|".join(
        [
            (market or "").strip().upper(),
            (exchange or "").strip().upper(),
            (ticker_normalized or "").strip().upper(),
            normalized_title,
            published_bucket,
            document_name,
        ]
    )
    return hashlib.sha256(raw_key.encode("utf-8")).hexdigest()


def _normalize_title_for_dedupe(title: str) -> str:
    # 2026-03-27: 这里只做保守标准化，避免过度语义合并导致不同公告误判为同一条。
    compact = re.sub(r"\s+", " ", (title or "").strip())
    return compact.casefold()


def _normalize_published_bucket(published_at: str) -> str:
    # 2026-03-27: 公告源跨站点分钟和秒可能不同，先按小时桶做首轮跨源去重。
    dt = datetime.fromisoformat(published_at)
    return dt.strftime("%Y-%m-%dT%H")


def _normalize_document_name(document_url: str | None) -> str:
    # 2026-03-27: 先取 URL 文件名而不是全链接，降低 CDN 参数变化对去重的干扰。
    if not document_url:
        return ""
    parsed = urlparse(document_url)
    return PurePosixPath(parsed.path).name.casefold()
