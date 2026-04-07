from __future__ import annotations

import json
import re
from dataclasses import dataclass
from datetime import date, datetime, timedelta, timezone
from hashlib import sha256
from pathlib import Path
from typing import Any

import requests

from tradingagents.dataflows.disclosure_models import (
    DisclosureAttachment,
    DisclosureEvent,
    SourceSnapshot,
    build_disclosure_dedupe_key,
    normalize_disclosure_ticker,
)
from tradingagents.dataflows.disclosure_store import DisclosureStore


CNINFO_BASE_URL = "https://www.cninfo.com.cn/new"
CNINFO_STATIC_HOST = "https://static.cninfo.com.cn"
SHANGHAI_TZ = timezone(timedelta(hours=8))


# 2026-03-27: M3-2 首轮需要先把巨潮证券定位结果固化成显式对象，目的是真正隔离后续抓取逻辑与裸 JSON。
@dataclass(slots=True)
class CninfoSecurityRef:
    sec_code: str
    org_id: str
    plate: str
    sec_name: str
    category: str

    @property
    def exchange(self) -> str:
        # 2026-03-27: 先只支持 A 股主板/深市/北交所映射，港股后续由 HKEXnews 单独接入。
        plate_key = (self.plate or "").strip().lower()
        if plate_key in {"sse", "hzb", "kcb"}:
            return "SSE"
        if plate_key in {"szse", "szb", "cyb"}:
            return "SZSE"
        if plate_key == "bjs":
            return "BSE"
        raise ValueError(f"Unsupported CNInfo plate '{self.plate}'")

    @property
    def normalize_market_key(self) -> str:
        # 2026-03-27: 统一把巨潮板块映射到已有 ticker 规范化函数的输入格式，避免重复分支。
        return {"SSE": "CN-SH", "SZSE": "CN-SZ", "BSE": "CN-BJ"}[self.exchange]


# 2026-03-27: M3-2 真实抓取先定义独立客户端，目的是把网络交互、解析和入库流程压在稳定边界内。
class CninfoDisclosureClient:
    def __init__(self, session: requests.Session | None = None, timeout: int = 20):
        self.session = session or requests.Session()
        self.timeout = timeout
        self.session.headers.update(
            {
                "User-Agent": (
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                    "AppleWebKit/537.36 (KHTML, like Gecko) "
                    "Chrome/135.0.0.0 Safari/537.36"
                ),
                "Referer": "https://www.cninfo.com.cn/new/index",
            }
        )

    def lookup_security_ref(self, ticker: str) -> CninfoSecurityRef:
        # 2026-03-27: 先通过顶部搜索接口拿到 orgId 与 plate，这是后续公告列表和详情接口的必要键。
        payload = self._request_json(
            method="POST",
            path="/information/topSearch/detailOfQuery",
            data={"keyWord": ticker, "maxSecNum": 10, "maxListNum": 5},
        )
        return resolve_cninfo_security_ref(ticker, payload)

    def fetch_announcements_page(
        self,
        security_ref: CninfoSecurityRef,
        page_num: int = 1,
        page_size: int = 30,
    ) -> dict[str, Any]:
        # 2026-03-27: 列表抓取走 hisAnnouncement/query，先覆盖单证券历史公告分页场景。
        return self._request_json(
            method="POST",
            path="/hisAnnouncement/query",
            data={
                "stock": f"{security_ref.sec_code},{security_ref.org_id}",
                "pageNum": page_num,
                "pageSize": page_size,
                "tabName": "fulltext",
            },
        )

    def fetch_announcement_detail(
        self,
        announcement_id: str,
        announcement_date: str,
        plate: str,
    ) -> dict[str, Any]:
        # 2026-03-27: 详情抓取改走 bulletin_detail JSON 接口，比直接解析详情页 HTML 更稳。
        return self._request_json(
            method="POST",
            path="/announcement/bulletin_detail",
            params={
                "announceId": announcement_id,
                "flag": str((plate or "").strip().lower() == "szse").lower(),
                "announceTime": announcement_date,
            },
        )

    def ingest_stock_disclosures(
        self,
        ticker: str,
        start_date: str,
        end_date: str,
        store: DisclosureStore,
        snapshot_root: str | Path,
        max_pages: int = 5,
        page_size: int = 30,
    ) -> list[DisclosureEvent]:
        # 2026-03-27: 首个真实抓取闭环从单 ticker 和日期窗口开始，目的是尽快验证在线链路能落盘。
        security_ref = self.lookup_security_ref(ticker)
        snapshot_root = Path(snapshot_root)
        ingested_events: list[DisclosureEvent] = []
        start_boundary = date.fromisoformat(start_date)
        end_boundary = date.fromisoformat(end_date)

        lookup_snapshot = write_cninfo_snapshot(
            snapshot_root=snapshot_root,
            snapshot_kind="lookup",
            identity_parts=["cninfo", security_ref.sec_code, "lookup"],
            payload={
                "ticker": ticker,
                "secCode": security_ref.sec_code,
                "orgId": security_ref.org_id,
                "plate": security_ref.plate,
                "secName": security_ref.sec_name,
            },
            source_url=f"{CNINFO_BASE_URL}/information/topSearch/detailOfQuery",
        )

        for page_num in range(1, max_pages + 1):
            page_payload = self.fetch_announcements_page(
                security_ref=security_ref,
                page_num=page_num,
                page_size=page_size,
            )
            write_cninfo_snapshot(
                snapshot_root=snapshot_root,
                snapshot_kind="list",
                identity_parts=["cninfo", security_ref.sec_code, f"page-{page_num:03d}"],
                payload=page_payload,
                source_url=f"{CNINFO_BASE_URL}/hisAnnouncement/query",
            )

            announcements = page_payload.get("announcements") or []
            if not announcements:
                break

            page_has_in_range_items = False
            for announcement in announcements:
                announcement_day = _timestamp_ms_to_date(announcement.get("announcementTime"))
                if announcement_day > end_boundary:
                    continue
                if announcement_day < start_boundary:
                    continue

                page_has_in_range_items = True
                detail_payload = self.fetch_announcement_detail(
                    announcement_id=str(announcement["announcementId"]),
                    announcement_date=announcement_day.isoformat(),
                    plate=security_ref.plate,
                )
                detail_snapshot = write_cninfo_snapshot(
                    snapshot_root=snapshot_root,
                    snapshot_kind="detail",
                    identity_parts=[
                        "cninfo",
                        security_ref.sec_code,
                        str(announcement["announcementId"]),
                    ],
                    payload=detail_payload,
                    source_url=f"{CNINFO_BASE_URL}/announcement/bulletin_detail",
                )
                event = build_cninfo_disclosure_event(
                    security_ref=security_ref,
                    detail_payload=detail_payload,
                    snapshot=detail_snapshot,
                    fallback_snapshot=lookup_snapshot,
                )
                store.upsert_event(event)
                ingested_events.append(event)

            oldest_announcement_day = _timestamp_ms_to_date(
                announcements[-1].get("announcementTime")
            )
            if oldest_announcement_day < start_boundary and not page_has_in_range_items:
                break

        return ingested_events

    def _request_json(
        self,
        method: str,
        path: str,
        params: dict[str, Any] | None = None,
        data: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        # 2026-03-27: 所有巨潮请求统一从这里走，方便后续补重试、限流和审计。
        response = self.session.request(
            method=method.upper(),
            url=f"{CNINFO_BASE_URL}{path}",
            params=params,
            data=data,
            timeout=self.timeout,
        )
        response.raise_for_status()
        return response.json()


def resolve_cninfo_security_ref(ticker: str, payload: dict[str, Any]) -> CninfoSecurityRef:
    # 2026-03-27: 证券定位必须先精确命中代码，否则巨潮返回多个候选时容易抓错发行人。
    candidates = payload.get("keyBoardList") or []
    if not candidates:
        raise ValueError(f"CNInfo did not return any security candidates for '{ticker}'")

    target = re.sub(r"[^0-9A-Za-z]", "", ticker or "").upper()
    exact_match = None
    for item in candidates:
        if re.sub(r"[^0-9A-Za-z]", "", str(item.get("code", ""))).upper() == target:
            exact_match = item
            break

    chosen = exact_match or candidates[0]
    return CninfoSecurityRef(
        sec_code=str(chosen["code"]),
        org_id=str(chosen["orgId"]),
        plate=str(chosen.get("plate") or ""),
        sec_name=str(chosen.get("zwjc") or chosen.get("secName") or chosen["code"]),
        category=str(chosen.get("category") or ""),
    )


def write_cninfo_snapshot(
    snapshot_root: str | Path,
    snapshot_kind: str,
    identity_parts: list[str],
    payload: dict[str, Any],
    source_url: str,
) -> SourceSnapshot:
    # 2026-03-27: 快照先统一写 JSON 文件，目的是让离线回放和结构漂移排查尽快可用。
    snapshot_root = Path(snapshot_root)
    safe_parts = [_slugify_snapshot_part(part) for part in identity_parts]
    snapshot_path = snapshot_root.joinpath(*safe_parts[:-1], f"{snapshot_kind}-{safe_parts[-1]}.json")
    snapshot_path.parent.mkdir(parents=True, exist_ok=True)
    serialized = json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True)
    snapshot_path.write_text(serialized, encoding="utf-8")
    digest = sha256(serialized.encode("utf-8")).hexdigest()
    return SourceSnapshot(
        source_name="cninfo",
        source_url=source_url,
        storage_path=str(snapshot_path),
        content_type="application/json",
        sha256=digest,
    )


def build_cninfo_disclosure_event(
    security_ref: CninfoSecurityRef,
    detail_payload: dict[str, Any],
    snapshot: SourceSnapshot,
    fallback_snapshot: SourceSnapshot | None = None,
) -> DisclosureEvent:
    # 2026-03-27: 详情映射集中在这里，目的是让真实抓取和离线回放共用同一套规范化规则。
    announcement = detail_payload.get("announcement") or {}
    announcement_id = str(announcement["announcementId"])
    announcement_date = _timestamp_ms_to_date(announcement.get("announcementTime")).isoformat()
    published_at = _timestamp_ms_to_iso(announcement.get("announcementTime"))
    ticker_normalized = normalize_disclosure_ticker(
        security_ref.normalize_market_key,
        str(announcement.get("secCode") or security_ref.sec_code),
    )
    document_url = _normalize_cninfo_document_url(
        detail_payload.get("fileUrl") or announcement.get("adjunctUrl")
    )
    detail_page_url = (
        f"{CNINFO_BASE_URL}/disclosure/detail?"
        f"orgId={announcement.get('orgId') or security_ref.org_id}&"
        f"announcementId={announcement_id}&"
        f"announcementTime={announcement_date}"
    )
    title = str(announcement.get("announcementTitle") or "").strip()
    dedupe_key = build_disclosure_dedupe_key(
        market="CN-A",
        exchange=security_ref.exchange,
        ticker_normalized=ticker_normalized,
        title=title,
        published_at=published_at,
        document_url=document_url,
    )
    attachment_name = Path(document_url).name if document_url else f"{announcement_id}.pdf"
    ingested_at = datetime.now(tz=SHANGHAI_TZ).isoformat(timespec="seconds")

    return DisclosureEvent(
        event_id=f"cninfo-{announcement_id}",
        market="CN-A",
        exchange=security_ref.exchange,
        ticker_raw=str(announcement.get("secCode") or security_ref.sec_code),
        ticker_normalized=ticker_normalized,
        issuer_name=str(announcement.get("secName") or security_ref.sec_name),
        title=title,
        category=_infer_cninfo_category(
            title=title,
            announcement_type=str(announcement.get("announcementType") or ""),
        ),
        published_at=published_at,
        document_date=announcement_date,
        language="zh-CN",
        source_name="cninfo",
        source_url=detail_page_url,
        document_url=document_url,
        document_type=str(announcement.get("adjunctType") or "pdf").lower(),
        content_text=announcement.get("announcementContent"),
        dedupe_key=dedupe_key,
        ingested_at=ingested_at,
        attachments=[
            DisclosureAttachment(
                name=attachment_name,
                url=document_url,
                media_type=_guess_media_type_from_url(document_url),
            )
        ]
        if document_url
        else [],
        snapshot=snapshot or fallback_snapshot,
    )


def _timestamp_ms_to_date(value: Any) -> date:
    # 2026-03-27: 巨潮时间戳是毫秒 Unix 时间，这里统一转成上海时区日期，减少日期边界歧义。
    return _timestamp_ms_to_datetime(value).date()


def _timestamp_ms_to_iso(value: Any) -> str:
    # 2026-03-27: published_at 统一输出 ISO 8601，后续 SQLite 和去重逻辑直接消费。
    return _timestamp_ms_to_datetime(value).isoformat(timespec="seconds")


def _timestamp_ms_to_datetime(value: Any) -> datetime:
    # 2026-03-27: 转换逻辑独立封装，目的是保证列表过滤和详情映射共用同一时区处理。
    if value is None:
        raise ValueError("CNInfo announcement timestamp must not be empty")
    return datetime.fromtimestamp(int(value) / 1000, tz=SHANGHAI_TZ)


def _normalize_cninfo_document_url(raw_url: str | None) -> str | None:
    # 2026-03-27: 巨潮 detail 接口经常返回 http 链接，这里统一升级到 https 并补静态域名。
    if not raw_url:
        return None
    raw_url = str(raw_url).strip()
    if raw_url.startswith("http://"):
        return "https://" + raw_url[len("http://") :]
    if raw_url.startswith("https://"):
        return raw_url
    return f"{CNINFO_STATIC_HOST}/{raw_url.lstrip('/')}"


def _infer_cninfo_category(title: str, announcement_type: str) -> str:
    # 2026-03-27: 首版分类只做保守关键词映射，目的是真正保持可解释和低误判。
    normalized_title = (title or "").strip()
    if "年度报告" in normalized_title:
        return "annual_report"
    if "半年度报告" in normalized_title:
        return "interim_report"
    if "第一季度报告" in normalized_title or "第三季度报告" in normalized_title:
        return "quarterly_report"
    if "业绩预告" in normalized_title or "业绩快报" in normalized_title:
        return "earnings_preannouncement"
    if "董事会" in normalized_title:
        return "board_meeting"
    if "股东大会" in normalized_title:
        return "shareholder_meeting"
    if "停牌" in normalized_title or "复牌" in normalized_title:
        return "suspension_resume"
    if "回购" in normalized_title:
        return "buyback"
    if "分红" in normalized_title or "利润分配" in normalized_title:
        return "dividend"
    if "重大合同" in normalized_title:
        return "material_contract"
    if "重组" in normalized_title or "收购" in normalized_title or "并购" in normalized_title:
        return "mna_restructuring"
    if announcement_type.startswith("0101"):
        return "other_disclosure"
    return "other_disclosure"


def _guess_media_type_from_url(document_url: str | None) -> str | None:
    # 2026-03-27: 附件 MIME 先按后缀做最小推断，够支撑第一版下载与展示。
    if not document_url:
        return None
    suffix = Path(document_url).suffix.lower()
    return {
        ".pdf": "application/pdf",
        ".doc": "application/msword",
        ".docx": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".xls": "application/vnd.ms-excel",
        ".xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".ppt": "application/vnd.ms-powerpoint",
        ".pptx": "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ".csv": "text/csv",
    }.get(suffix)


def _slugify_snapshot_part(value: Any) -> str:
    # 2026-03-27: 快照路径需要稳定且跨平台可写，因此统一做轻量字符清洗。
    text = re.sub(r"[^0-9A-Za-z._-]+", "-", str(value or "").strip())
    return text.strip("-") or "snapshot"
