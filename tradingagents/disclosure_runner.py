from __future__ import annotations

import json
import os
import re
from dataclasses import asdict, dataclass
from pathlib import Path

from tradingagents.dataflows.disclosure_cninfo import CninfoDisclosureClient
from tradingagents.dataflows.disclosure_models import normalize_disclosure_ticker
from tradingagents.dataflows.disclosure_sse_verifier import (
    SseAnnouncementVerifier,
    compare_sse_bulletins_with_store,
)
from tradingagents.dataflows.disclosure_store import DisclosureStore


# 2026-03-27: M3-4 统一运行时路径先集中定义，目的是让后续打包成便携版或 exe 时不依赖开发目录结构。
@dataclass(slots=True)
class DisclosureRuntimePaths:
    db_path: Path
    snapshot_root: Path
    report_path: Path


# 2026-03-27: M3-4 统一摘要对象用于 CLI 输出和 JSON 报告落盘，目的是固定入口行为便于打包与自动化。
@dataclass(slots=True)
class DisclosureRunSummary:
    ticker: str
    start_date: str
    end_date: str
    cninfo_ingested_count: int
    sse_fetched_count: int
    matched_count: int
    sse_only_count: int
    cninfo_only_count: int
    db_path: Path
    snapshot_root: Path
    report_path: Path

    def to_dict(self) -> dict[str, object]:
        # 2026-03-27: JSON 报告对 Path 做显式字符串化，避免不同运行环境序列化结果不一致。
        payload = asdict(self)
        payload["db_path"] = str(self.db_path)
        payload["snapshot_root"] = str(self.snapshot_root)
        payload["report_path"] = str(self.report_path)
        payload["paths"] = {
            "db_path": str(self.db_path),
            "snapshot_root": str(self.snapshot_root),
            "report_path": str(self.report_path),
        }
        return payload


def build_disclosure_runtime_paths(
    data_root: str | Path | None,
    ticker: str,
    start_date: str,
    end_date: str,
) -> DisclosureRuntimePaths:
    # 2026-03-27: 运行时路径按“缓存”和“报告”分层，目的是让最终便携版或 exe 使用时目录结构稳定可预期。
    root = Path(data_root) if data_root else get_default_disclosure_data_root()
    safe_ticker = _slugify_runtime_part(ticker)
    safe_range = f"{_slugify_runtime_part(start_date)}_{_slugify_runtime_part(end_date)}"
    run_dir = root / "runs" / f"{safe_ticker}_{safe_range}"
    return DisclosureRuntimePaths(
        db_path=root / "cache" / "disclosure" / "disclosures.sqlite3",
        snapshot_root=root / "cache" / "disclosure_snapshots" / "cninfo" / safe_ticker,
        report_path=run_dir / "run_summary.json",
    )


def get_default_disclosure_data_root() -> Path:
    # 2026-03-27: 默认写入用户目录而不是工程目录，目的是降低普通用户使用门槛并适配未来打包场景。
    local_appdata = os.getenv("LOCALAPPDATA")
    if local_appdata:
        return Path(local_appdata) / "TradingAgents" / "disclosure"
    return Path.home() / ".tradingagents" / "disclosure"


def run_disclosure_pipeline(
    ticker: str,
    start_date: str,
    end_date: str,
    data_root: str | Path | None = None,
    fetch_cninfo: bool = True,
    verify_sse: bool = True,
    cninfo_client: CninfoDisclosureClient | None = None,
    sse_verifier: SseAnnouncementVerifier | None = None,
) -> DisclosureRunSummary:
    # 2026-03-27: 统一入口先串起“巨潮抓取 + 上交所校验”，目的是给未来免环境交付提供稳定编排层。
    paths = build_disclosure_runtime_paths(
        data_root=data_root,
        ticker=ticker,
        start_date=start_date,
        end_date=end_date,
    )
    paths.db_path.parent.mkdir(parents=True, exist_ok=True)
    paths.snapshot_root.mkdir(parents=True, exist_ok=True)
    paths.report_path.parent.mkdir(parents=True, exist_ok=True)

    store = DisclosureStore(paths.db_path)
    cninfo_client = cninfo_client or CninfoDisclosureClient()
    sse_verifier = sse_verifier or SseAnnouncementVerifier()

    cninfo_events = []
    if fetch_cninfo:
        cninfo_events = cninfo_client.ingest_stock_disclosures(
            ticker=ticker,
            start_date=start_date,
            end_date=end_date,
            store=store,
            snapshot_root=paths.snapshot_root,
        )

    sse_records = []
    matched_count = 0
    sse_only_count = 0
    cninfo_only_count = 0
    if verify_sse:
        sse_records = sse_verifier.fetch_bulletins(
            ticker=ticker,
            start_date=start_date,
            end_date=end_date,
        )
        comparison = compare_sse_bulletins_with_store(
            store=store,
            ticker_normalized=normalize_disclosure_ticker("CN-SH", ticker),
            start_date=start_date,
            end_date=end_date,
            sse_rows=[
                {
                    "SECURITY_CODE": record.ticker_raw,
                    "SECURITY_NAME": record.issuer_name,
                    "TITLE": record.title,
                    "SSEDATE": record.document_date,
                    "URL": record.document_url,
                    "ORG_BULLETIN_ID": record.bulletin_id,
                    "BULLETIN_TYPE_DESC": record.bulletin_type_desc or "",
                }
                for record in sse_records
            ],
        )
        matched_count = len(comparison.matched_pairs)
        sse_only_count = len(comparison.sse_only)
        cninfo_only_count = len(comparison.cninfo_only)

    summary = DisclosureRunSummary(
        ticker=ticker,
        start_date=start_date,
        end_date=end_date,
        cninfo_ingested_count=len(cninfo_events),
        sse_fetched_count=len(sse_records),
        matched_count=matched_count,
        sse_only_count=sse_only_count,
        cninfo_only_count=cninfo_only_count,
        db_path=paths.db_path,
        snapshot_root=paths.snapshot_root,
        report_path=paths.report_path,
    )
    paths.report_path.write_text(
        json.dumps(summary.to_dict(), ensure_ascii=False, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    return summary


def _slugify_runtime_part(value: str) -> str:
    # 2026-03-27: 路径片段先做轻量清洗，目的是保证 Windows 打包后的输出目录稳定可写。
    return re.sub(r"[^0-9A-Za-z._-]+", "-", str(value).strip()).strip("-") or "run"
