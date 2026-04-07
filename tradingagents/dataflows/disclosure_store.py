from __future__ import annotations

import sqlite3
from contextlib import closing
from pathlib import Path

from tradingagents.dataflows.disclosure_models import DisclosureEvent


# 2026-03-27: M3-1 先用标准库 sqlite3 落地，避免为了首轮契约引入额外 ORM 复杂度。
class DisclosureStore:
    def __init__(self, db_path: str | Path):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._initialize_schema()

    def upsert_event(self, event: DisclosureEvent) -> None:
        # 2026-03-27: 先支持幂等写入，后续抓取器重复回灌时可以直接覆盖同 event_id 记录。
        payload = event.to_record()
        # 2026-03-27 追加：修复 Windows 下临时库文件被锁定的问题，原因是 sqlite3 上下文不会自动 close 连接。
        with closing(sqlite3.connect(self.db_path)) as connection:
            connection.execute(
                """
                INSERT INTO disclosure_events (
                    event_id,
                    market,
                    exchange,
                    ticker_raw,
                    ticker_normalized,
                    issuer_name,
                    title,
                    category,
                    published_at,
                    document_date,
                    language,
                    source_name,
                    source_url,
                    document_url,
                    document_type,
                    content_text,
                    dedupe_key,
                    ingested_at,
                    attachments_json,
                    snapshot_json
                ) VALUES (
                    :event_id,
                    :market,
                    :exchange,
                    :ticker_raw,
                    :ticker_normalized,
                    :issuer_name,
                    :title,
                    :category,
                    :published_at,
                    :document_date,
                    :language,
                    :source_name,
                    :source_url,
                    :document_url,
                    :document_type,
                    :content_text,
                    :dedupe_key,
                    :ingested_at,
                    :attachments_json,
                    :snapshot_json
                )
                ON CONFLICT(event_id) DO UPDATE SET
                    market = excluded.market,
                    exchange = excluded.exchange,
                    ticker_raw = excluded.ticker_raw,
                    ticker_normalized = excluded.ticker_normalized,
                    issuer_name = excluded.issuer_name,
                    title = excluded.title,
                    category = excluded.category,
                    published_at = excluded.published_at,
                    document_date = excluded.document_date,
                    language = excluded.language,
                    source_name = excluded.source_name,
                    source_url = excluded.source_url,
                    document_url = excluded.document_url,
                    document_type = excluded.document_type,
                    content_text = excluded.content_text,
                    dedupe_key = excluded.dedupe_key,
                    ingested_at = excluded.ingested_at,
                    attachments_json = excluded.attachments_json,
                    snapshot_json = excluded.snapshot_json
                """,
                payload,
            )
            connection.commit()

    def get_event(self, event_id: str) -> DisclosureEvent | None:
        # 2026-03-27: 首轮先提供按主键读取，满足回放验证和后续接入层调试。
        # 2026-03-27 追加：读取路径同样显式关闭连接，避免测试和后续批量回放时残留文件句柄。
        with closing(sqlite3.connect(self.db_path)) as connection:
            connection.row_factory = sqlite3.Row
            row = connection.execute(
                """
                SELECT
                    event_id,
                    market,
                    exchange,
                    ticker_raw,
                    ticker_normalized,
                    issuer_name,
                    title,
                    category,
                    published_at,
                    document_date,
                    language,
                    source_name,
                    source_url,
                    document_url,
                    document_type,
                    content_text,
                    dedupe_key,
                    ingested_at,
                    attachments_json,
                    snapshot_json
                FROM disclosure_events
                WHERE event_id = ?
                """,
                (event_id,),
            ).fetchone()

        if row is None:
            return None
        return DisclosureEvent.from_record(dict(row))

    def list_events_by_ticker_and_date_range(
        self,
        ticker_normalized: str,
        start_date: str,
        end_date: str,
    ) -> list[DisclosureEvent]:
        # 2026-03-27: 上交所校验需要按代码和日期窗口批量读取巨潮事件，这里先提供最小查询接口。
        with closing(sqlite3.connect(self.db_path)) as connection:
            connection.row_factory = sqlite3.Row
            rows = connection.execute(
                """
                SELECT
                    event_id,
                    market,
                    exchange,
                    ticker_raw,
                    ticker_normalized,
                    issuer_name,
                    title,
                    category,
                    published_at,
                    document_date,
                    language,
                    source_name,
                    source_url,
                    document_url,
                    document_type,
                    content_text,
                    dedupe_key,
                    ingested_at,
                    attachments_json,
                    snapshot_json
                FROM disclosure_events
                WHERE ticker_normalized = ?
                  AND document_date >= ?
                  AND document_date <= ?
                ORDER BY document_date DESC, event_id DESC
                """,
                (ticker_normalized, start_date, end_date),
            ).fetchall()

        return [DisclosureEvent.from_record(dict(row)) for row in rows]

    def _initialize_schema(self) -> None:
        # 2026-03-27: schema 保持单表 + 索引，先满足契约落盘和最基础查询，再决定是否拆表。
        # 2026-03-27 追加：建表阶段也显式关闭连接，避免初始化后马上删除临时库时出现句柄占用。
        with closing(sqlite3.connect(self.db_path)) as connection:
            connection.execute(
                """
                CREATE TABLE IF NOT EXISTS disclosure_events (
                    event_id TEXT PRIMARY KEY,
                    market TEXT NOT NULL,
                    exchange TEXT NOT NULL,
                    ticker_raw TEXT NOT NULL,
                    ticker_normalized TEXT NOT NULL,
                    issuer_name TEXT NOT NULL,
                    title TEXT NOT NULL,
                    category TEXT NOT NULL,
                    published_at TEXT NOT NULL,
                    document_date TEXT,
                    language TEXT,
                    source_name TEXT NOT NULL,
                    source_url TEXT NOT NULL,
                    document_url TEXT,
                    document_type TEXT,
                    content_text TEXT,
                    dedupe_key TEXT NOT NULL,
                    ingested_at TEXT NOT NULL,
                    attachments_json TEXT NOT NULL,
                    snapshot_json TEXT
                )
                """
            )
            connection.execute(
                """
                CREATE INDEX IF NOT EXISTS idx_disclosure_events_dedupe_key
                ON disclosure_events (dedupe_key)
                """
            )
            connection.execute(
                """
                CREATE INDEX IF NOT EXISTS idx_disclosure_events_ticker_published
                ON disclosure_events (ticker_normalized, published_at)
                """
            )
            connection.commit()
