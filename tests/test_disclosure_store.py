import tempfile
import unittest
from pathlib import Path

from tradingagents.dataflows.disclosure_models import (
    DisclosureAttachment,
    DisclosureEvent,
    SourceSnapshot,
    build_disclosure_dedupe_key,
    normalize_disclosure_ticker,
)
from tradingagents.dataflows.disclosure_store import DisclosureStore


class DisclosureTickerNormalizationTests(unittest.TestCase):
    def test_normalize_disclosure_ticker_formats_a_share_and_hk_symbols(self):
        self.assertEqual(normalize_disclosure_ticker("CN-SH", "600519"), "600519.SH")
        self.assertEqual(normalize_disclosure_ticker("CN-SZ", "000001"), "000001.SZ")
        self.assertEqual(normalize_disclosure_ticker("CN-BJ", "430047"), "430047.BJ")
        self.assertEqual(normalize_disclosure_ticker("HK", "700"), "0700.HK")

    def test_build_disclosure_dedupe_key_is_stable_for_equivalent_title_variants(self):
        left = build_disclosure_dedupe_key(
            market="CN-A",
            exchange="SSE",
            ticker_normalized="600519.SH",
            title="关于年度报告的公告 ",
            published_at="2026-03-27T09:30:00+08:00",
            document_url="https://example.com/docs/abc123.PDF",
        )
        right = build_disclosure_dedupe_key(
            market="CN-A",
            exchange="SSE",
            ticker_normalized="600519.SH",
            title="关于年度报告的公告",
            published_at="2026-03-27T09:59:59+08:00",
            document_url="https://example.com/docs/abc123.pdf",
        )
        self.assertEqual(left, right)


class DisclosureStoreTests(unittest.TestCase):
    def test_disclosure_store_round_trips_event_and_snapshot(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            db_path = Path(temp_dir) / "disclosure.db"
            store = DisclosureStore(db_path)

            event = DisclosureEvent(
                event_id="evt-001",
                market="CN-A",
                exchange="SSE",
                ticker_raw="600519",
                ticker_normalized="600519.SH",
                issuer_name="贵州茅台",
                title="2025年年度报告",
                category="annual_report",
                published_at="2026-03-27T20:15:00+08:00",
                document_date="2026-03-26",
                language="zh-CN",
                source_name="cninfo",
                source_url="https://www.cninfo.com.cn/example",
                document_url="https://static.cninfo.com.cn/annual-report.pdf",
                document_type="pdf",
                content_text="年度报告摘要",
                dedupe_key="dedupe-001",
                ingested_at="2026-03-27T20:16:00+08:00",
                attachments=[
                    DisclosureAttachment(
                        name="annual-report.pdf",
                        url="https://static.cninfo.com.cn/annual-report.pdf",
                        media_type="application/pdf",
                    )
                ],
                snapshot=SourceSnapshot(
                    source_name="cninfo",
                    source_url="https://www.cninfo.com.cn/example",
                    storage_path=str(Path(temp_dir) / "snapshots" / "evt-001.json"),
                    content_type="application/json",
                    sha256="abc123",
                ),
            )

            store.upsert_event(event)
            stored = store.get_event("evt-001")

            self.assertIsNotNone(stored)
            self.assertEqual(stored.ticker_normalized, "600519.SH")
            self.assertEqual(stored.attachments[0].name, "annual-report.pdf")
            self.assertEqual(stored.snapshot.sha256, "abc123")


if __name__ == "__main__":
    unittest.main()
