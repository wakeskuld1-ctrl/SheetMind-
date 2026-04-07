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


# 2026-03-27: M3-4 统一运行时路径集中定义，目的是让后续便携版或 exe 包装时不依赖开发目录结构。
@dataclass(slots=True)
class DisclosureRuntimePaths:
    db_path: Path
    snapshot_root: Path
    report_path: Path


# 2026-03-27: M3-5 新增市场路由对象，目的是把“代码 -> 市场 -> 校验源”的决策从 runner 主流程中拆出来。
@dataclass(slots=True)
class DisclosureMarketRoute:
    market_key: str
    exchange: str
    normalized_ticker: str
    verification_source: str | None = None


# 2026-03-27: M3-4 统一摘要对象用于 CLI 输出和 JSON 报告落盘，目的是固定入口行为便于打包与自动化。
# 2026-03-27 追加：M3-5 补充市场路由元数据，原因是后续深市/北交所/港股扩展需要在报告里明确当前运行按哪个市场决策。
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
    market_key: str | None = None
    exchange: str | None = None
    verification_source: str | None = None

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
    # 2026-03-27: 运行时路径按“缓存”和“报告”分层，目的是让最终便携版使用时目录结构稳定可预期。
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


def resolve_disclosure_market_route(ticker: str) -> DisclosureMarketRoute:
    # 2026-03-27: M3-5 先用轻量代码规则做市场路由，目的是先移除 runner 对 `CN-SH` 的硬编码依赖。
    # 2026-03-27 追加：当前只给上交所挂接校验器，其它市场保持“可抓取、可落盘、校验先跳过”的稳定行为。
    compact = re.sub(r"[^0-9A-Za-z.]", "", ticker or "").upper()
    if not compact:
        raise ValueError("ticker must not be empty")

    code = compact
    suffix = None
    if "." in compact:
        code, suffix = compact.rsplit(".", 1)
        suffix = suffix.upper()

    if suffix == "SH":
        return _build_market_route("CN-SH", "SSE", compact, "sse")
    if suffix == "SZ":
        return _build_market_route("CN-SZ", "SZSE", compact, None)
    if suffix == "BJ":
        return _build_market_route("CN-BJ", "BSE", compact, None)
    if suffix == "HK":
        return _build_market_route("HK", "HKEX", compact, None)

    # 2026-03-27: A 股首版先按常见证券代码前缀做显式分流，避免 000001 这类深市代码被误当成沪市去校验。
    if code.startswith(("4", "8")):
        return _build_market_route("CN-BJ", "BSE", code, None)
    if code.startswith(("0", "1", "2", "3")):
        return _build_market_route("CN-SZ", "SZSE", code, None)
    if code.startswith(("5", "6", "7", "9")):
        return _build_market_route("CN-SH", "SSE", code, "sse")

    raise ValueError(f"Unable to resolve disclosure market route for ticker '{ticker}'")


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
    # 2026-03-27 追加：M3-5 在入口处先解析市场路由，原因是校验器选择和 ticker 规范化不能再默认按沪市处理。
    paths = build_disclosure_runtime_paths(
        data_root=data_root,
        ticker=ticker,
        start_date=start_date,
        end_date=end_date,
    )
    route = resolve_disclosure_market_route(ticker)

    paths.db_path.parent.mkdir(parents=True, exist_ok=True)
    paths.snapshot_root.mkdir(parents=True, exist_ok=True)
    paths.report_path.parent.mkdir(parents=True, exist_ok=True)

    store = DisclosureStore(paths.db_path)
    cninfo_client = cninfo_client or CninfoDisclosureClient()

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
    if verify_sse and route.verification_source == "sse":
        # 2026-03-27: 当前只有上交所校验器已实现，因此只在沪市路由下启用它，避免深市/北交所误走错误对比逻辑。
        sse_verifier = sse_verifier or SseAnnouncementVerifier()
        sse_records = sse_verifier.fetch_bulletins(
            ticker=ticker,
            start_date=start_date,
            end_date=end_date,
        )
        comparison = compare_sse_bulletins_with_store(
            store=store,
            ticker_normalized=route.normalized_ticker,
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
        market_key=route.market_key,
        exchange=route.exchange,
        verification_source=route.verification_source,
    )
    paths.report_path.write_text(
        json.dumps(summary.to_dict(), ensure_ascii=False, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    return summary


def _build_market_route(
    market_key: str,
    exchange: str,
    ticker: str,
    verification_source: str | None,
) -> DisclosureMarketRoute:
    # 2026-03-27: 路由构造单独收口，目的是避免每个分支重复 ticker 规范化和对象拼装逻辑。
    return DisclosureMarketRoute(
        market_key=market_key,
        exchange=exchange,
        normalized_ticker=normalize_disclosure_ticker(market_key, ticker),
        verification_source=verification_source,
    )


def _slugify_runtime_part(value: str) -> str:
    # 2026-03-27: 路径片段先做轻量清洗，目的是保证 Windows 打包后的输出目录稳定可写。
    return re.sub(r"[^0-9A-Za-z._-]+", "-", str(value).strip()).strip("-") or "run"
