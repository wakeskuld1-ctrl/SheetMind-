from __future__ import annotations

import json
import re
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Any

import requests

from tradingagents.dataflows.disclosure_models import normalize_disclosure_ticker
from tradingagents.dataflows.disclosure_store import DisclosureStore


SSE_QUERY_BASE_URL = "https://query.sse.com.cn/"
SSE_SITE_BASE_URL = "https://www.sse.com.cn"
SHANGHAI_TZ = timezone(timedelta(hours=8))


# 2026-03-27: M3-3 先把上交所公告行压成可比对对象，目的是真正把“抓取源”和“校验逻辑”分开。
@dataclass(slots=True)
class SseBulletinRecord:
    bulletin_id: str
    ticker_raw: str
    ticker_normalized: str
    issuer_name: str
    title: str
    document_date: str
    document_url: str
    bulletin_type_desc: str | None = None

    @property
    def comparison_key(self) -> tuple[str, str, str]:
        # 2026-03-27: 校验首版按“代码 + 日期 + 标题”做保守匹配，先追求可解释和低误并。
        return (
            self.ticker_normalized,
            self.document_date,
            _normalize_title_for_compare(self.title),
        )


# 2026-03-27: M3-3 校验输出先只关心三类结果，目的是真正快速发现“匹配 / 上交所独有 / 巨潮独有”。
@dataclass(slots=True)
class SseStoreComparisonResult:
    matched_pairs: list[tuple[SseBulletinRecord, Any]]
    sse_only: list[SseBulletinRecord]
    cninfo_only: list[Any]


# 2026-03-27: M3-3 先做上交所列表抓取客户端，目的是尽快把沪市校验源跑通，不做正式入库。
class SseAnnouncementVerifier:
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
                "Referer": "https://www.sse.com.cn/disclosure/listedinfo/announcement/",
            }
        )

    def fetch_bulletins(
        self,
        ticker: str,
        start_date: str,
        end_date: str,
        stock_type: str = "1",
        page_no: int = 1,
        page_size: int = 25,
    ) -> list[SseBulletinRecord]:
        # 2026-03-27: 上交所校验只需要列表接口即可，先不解析详情页或摘要页。
        callback_name = f"jsonpCallback{int(datetime.now(tz=SHANGHAI_TZ).timestamp() * 1000)}"
        response = self.session.get(
            SSE_QUERY_BASE_URL + "security/stock/queryCompanyBulletinNew.do",
            params={
                "jsonCallBack": callback_name,
                "isPagination": "true",
                "pageHelp.pageSize": str(page_size),
                "pageHelp.pageNo": str(page_no),
                "pageHelp.beginPage": str(page_no),
                "pageHelp.cacheSize": "1",
                "pageHelp.endPage": str(page_no),
                "SECURITY_CODE": ticker,
                "TITLE": "",
                "BULLETIN_TYPE": "",
                "stockType": stock_type,
                "START_DATE": start_date,
                "END_DATE": end_date,
            },
            timeout=self.timeout,
        )
        response.raise_for_status()
        payload = parse_sse_jsonp_payload(response.text)
        rows = _flatten_sse_result_rows(payload)
        return [normalize_sse_bulletin(row) for row in rows]


def parse_sse_jsonp_payload(raw_text: str) -> dict[str, Any]:
    # 2026-03-27: 上交所接口默认返回 JSONP，这里先显式剥壳，避免后续请求层重复写解析代码。
    match = re.search(r"^[^(]+\((.*)\)\s*$", raw_text or "", re.S)
    if not match:
        raise ValueError("SSE JSONP payload format is invalid")
    return json.loads(match.group(1))


def normalize_sse_bulletin(row: dict[str, Any]) -> SseBulletinRecord:
    # 2026-03-27: 公告行标准化只保留校验真正需要的字段，后续若升级正式双源入库再扩字段。
    raw_code = str(row.get("SECURITY_CODE") or "").strip()
    return SseBulletinRecord(
        bulletin_id=str(row.get("ORG_BULLETIN_ID") or ""),
        ticker_raw=raw_code,
        ticker_normalized=normalize_disclosure_ticker("CN-SH", raw_code),
        issuer_name=str(row.get("SECURITY_NAME") or "").strip(),
        title=str(row.get("TITLE") or "").strip(),
        document_date=str(row.get("SSEDATE") or "").strip(),
        document_url=_normalize_sse_document_url(row.get("URL")),
        bulletin_type_desc=str(row.get("BULLETIN_TYPE_DESC") or "").strip() or None,
    )


def compare_sse_bulletins_with_store(
    store: DisclosureStore,
    ticker_normalized: str,
    start_date: str,
    end_date: str,
    sse_rows: list[dict[str, Any]],
) -> SseStoreComparisonResult:
    # 2026-03-27: 校验首版直接对比上交所抓取结果与巨潮已入库事件，目的是尽快发现漏抓和标题偏差。
    sse_records = [normalize_sse_bulletin(row) for row in sse_rows]
    cninfo_events = store.list_events_by_ticker_and_date_range(
        ticker_normalized=ticker_normalized,
        start_date=start_date,
        end_date=end_date,
    )

    cninfo_index: dict[tuple[str, str, str], list[Any]] = {}
    for event in cninfo_events:
        key = (
            event.ticker_normalized,
            str(event.document_date or ""),
            _normalize_title_for_compare(event.title),
        )
        cninfo_index.setdefault(key, []).append(event)

    matched_pairs: list[tuple[SseBulletinRecord, Any]] = []
    sse_only: list[SseBulletinRecord] = []
    matched_cninfo_ids: set[str] = set()

    for sse_record in sse_records:
        candidates = cninfo_index.get(sse_record.comparison_key) or []
        unmatched_candidate = next(
            (candidate for candidate in candidates if candidate.event_id not in matched_cninfo_ids),
            None,
        )
        if unmatched_candidate is None:
            sse_only.append(sse_record)
            continue
        matched_pairs.append((sse_record, unmatched_candidate))
        matched_cninfo_ids.add(unmatched_candidate.event_id)

    cninfo_only = [
        event for event in cninfo_events if event.event_id not in matched_cninfo_ids
    ]

    return SseStoreComparisonResult(
        matched_pairs=matched_pairs,
        sse_only=sse_only,
        cninfo_only=cninfo_only,
    )


def _flatten_sse_result_rows(payload: dict[str, Any]) -> list[dict[str, Any]]:
    # 2026-03-27: 上交所结果是“result 里嵌单元素列表”的结构，这里先拍平成普通行列表便于校验。
    rows: list[dict[str, Any]] = []
    for item in payload.get("result") or []:
        if isinstance(item, list):
            for child in item:
                if isinstance(child, dict):
                    rows.append(child)
        elif isinstance(item, dict):
            rows.append(item)
    return rows


def _normalize_sse_document_url(raw_url: Any) -> str:
    # 2026-03-27: 上交所列表返回相对路径，这里统一补全成绝对地址，方便后续人工核对和下载。
    url = str(raw_url or "").strip()
    if url.startswith("http://") or url.startswith("https://"):
        return url
    return f"{SSE_SITE_BASE_URL}/{url.lstrip('/')}"


def _normalize_title_for_compare(title: str) -> str:
    # 2026-03-27: 标题对比只做轻量空白标准化，避免把不同公告误合并成一条。
    return re.sub(r"\s+", " ", (title or "").strip()).casefold()
