import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from typer.testing import CliRunner

from cli.disclosure import app
from tradingagents.dataflows.disclosure_models import DisclosureAttachment, DisclosureEvent
from tradingagents.dataflows.disclosure_sse_verifier import SseBulletinRecord
from tradingagents.disclosure_runner import (
    DisclosureMarketRoute,
    DisclosureRunSummary,
    build_disclosure_runtime_paths,
    resolve_disclosure_market_route,
    run_disclosure_pipeline,
)


class DisclosureRuntimePathTests(unittest.TestCase):
    def test_build_disclosure_runtime_paths_keeps_outputs_under_selected_root(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            paths = build_disclosure_runtime_paths(
                data_root=Path(temp_dir),
                ticker="600519",
                start_date="2026-03-01",
                end_date="2026-03-27",
            )

            self.assertTrue(str(paths.db_path).startswith(temp_dir))
            self.assertTrue(str(paths.snapshot_root).startswith(temp_dir))
            self.assertTrue(str(paths.report_path).startswith(temp_dir))
            self.assertEqual(paths.report_path.name, "run_summary.json")


class DisclosureMarketRouteTests(unittest.TestCase):
    def test_resolve_disclosure_market_route_maps_mainland_tickers_to_expected_market(self):
        sh_route = resolve_disclosure_market_route("600519")
        sz_route = resolve_disclosure_market_route("000001")
        bj_route = resolve_disclosure_market_route("430047")

        self.assertEqual(
            sh_route,
            DisclosureMarketRoute(
                market_key="CN-SH",
                exchange="SSE",
                normalized_ticker="600519.SH",
                verification_source="sse",
            ),
        )
        self.assertEqual(
            sz_route,
            DisclosureMarketRoute(
                market_key="CN-SZ",
                exchange="SZSE",
                normalized_ticker="000001.SZ",
                verification_source=None,
            ),
        )
        self.assertEqual(
            bj_route,
            DisclosureMarketRoute(
                market_key="CN-BJ",
                exchange="BSE",
                normalized_ticker="430047.BJ",
                verification_source=None,
            ),
        )


class DisclosureRunnerTests(unittest.TestCase):
    def test_run_disclosure_pipeline_writes_summary_report(self):
        class FakeCninfoClient:
            def ingest_stock_disclosures(
                self,
                ticker,
                start_date,
                end_date,
                store,
                snapshot_root,
                max_pages=5,
                page_size=30,
            ):
                event = DisclosureEvent(
                    event_id="cninfo-1224994303",
                    market="CN-A",
                    exchange="SSE",
                    ticker_raw=ticker,
                    ticker_normalized="600519.SH",
                    issuer_name="贵州茅台",
                    title="贵州茅台关于回购股份实施进展的公告",
                    category="buyback",
                    published_at="2026-03-04T00:00:00+08:00",
                    document_date="2026-03-04",
                    language="zh-CN",
                    source_name="cninfo",
                    source_url="https://www.cninfo.com.cn/new/disclosure/detail",
                    document_url="https://static.cninfo.com.cn/finalpage/2026-03-04/1224994303.PDF",
                    document_type="pdf",
                    content_text=None,
                    dedupe_key="dedupe-1",
                    ingested_at="2026-03-27T16:00:00+08:00",
                    attachments=[
                        DisclosureAttachment(
                            name="1224994303.PDF",
                            url="https://static.cninfo.com.cn/finalpage/2026-03-04/1224994303.PDF",
                            media_type="application/pdf",
                        )
                    ],
                )
                store.upsert_event(event)
                return [event]

        class FakeSseVerifier:
            def fetch_bulletins(
                self,
                ticker,
                start_date,
                end_date,
                stock_type="1",
                page_no=1,
                page_size=25,
            ):
                return [
                    SseBulletinRecord(
                        bulletin_id="6834195426960051",
                        ticker_raw=ticker,
                        ticker_normalized="600519.SH",
                        issuer_name="贵州茅台",
                        title="贵州茅台关于回购股份实施进展的公告",
                        document_date="2026-03-04",
                        document_url="https://www.sse.com.cn/disclosure/listedinfo/announcement/c/new/2026-03-04/600519_20260304_HMVE.pdf",
                        bulletin_type_desc="回购股份",
                    )
                ]

        with tempfile.TemporaryDirectory() as temp_dir:
            summary = run_disclosure_pipeline(
                ticker="600519",
                start_date="2026-03-01",
                end_date="2026-03-27",
                data_root=Path(temp_dir),
                fetch_cninfo=True,
                verify_sse=True,
                cninfo_client=FakeCninfoClient(),
                sse_verifier=FakeSseVerifier(),
            )

            self.assertEqual(summary.ticker, "600519")
            self.assertEqual(summary.cninfo_ingested_count, 1)
            self.assertEqual(summary.sse_fetched_count, 1)
            self.assertEqual(summary.matched_count, 1)
            self.assertEqual(summary.sse_only_count, 0)
            self.assertEqual(summary.cninfo_only_count, 0)
            self.assertTrue(summary.report_path.exists())

            payload = json.loads(summary.report_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["ticker"], "600519")
            self.assertEqual(payload["matched_count"], 1)
            self.assertEqual(payload["paths"]["report_path"], str(summary.report_path))

    def test_run_disclosure_pipeline_skips_sse_verification_for_non_sse_route(self):
        class FakeCninfoClient:
            def ingest_stock_disclosures(
                self,
                ticker,
                start_date,
                end_date,
                store,
                snapshot_root,
                max_pages=5,
                page_size=30,
            ):
                event = DisclosureEvent(
                    event_id="cninfo-000001-1",
                    market="CN-A",
                    exchange="SZSE",
                    ticker_raw=ticker,
                    ticker_normalized="000001.SZ",
                    issuer_name="平安银行",
                    title="平安银行关于董事会决议的公告",
                    category="board_meeting",
                    published_at="2026-03-05T00:00:00+08:00",
                    document_date="2026-03-05",
                    language="zh-CN",
                    source_name="cninfo",
                    source_url="https://www.cninfo.com.cn/new/disclosure/detail",
                    document_url="https://static.cninfo.com.cn/finalpage/2026-03-05/000001.PDF",
                    document_type="pdf",
                    content_text=None,
                    dedupe_key="dedupe-000001-1",
                    ingested_at="2026-03-27T18:20:00+08:00",
                    attachments=[],
                )
                store.upsert_event(event)
                return [event]

        class GuardSseVerifier:
            def fetch_bulletins(
                self,
                ticker,
                start_date,
                end_date,
                stock_type="1",
                page_no=1,
                page_size=25,
            ):
                raise AssertionError("non-SSE ticker must not call SSE verifier")

        with tempfile.TemporaryDirectory() as temp_dir:
            summary = run_disclosure_pipeline(
                ticker="000001",
                start_date="2026-03-01",
                end_date="2026-03-27",
                data_root=Path(temp_dir),
                fetch_cninfo=True,
                verify_sse=True,
                cninfo_client=FakeCninfoClient(),
                sse_verifier=GuardSseVerifier(),
            )

            self.assertEqual(summary.ticker, "000001")
            self.assertEqual(summary.cninfo_ingested_count, 1)
            self.assertEqual(summary.sse_fetched_count, 0)
            self.assertEqual(summary.matched_count, 0)
            self.assertEqual(summary.sse_only_count, 0)
            self.assertEqual(summary.cninfo_only_count, 0)
            self.assertEqual(summary.market_key, "CN-SZ")
            self.assertEqual(summary.exchange, "SZSE")
            self.assertIsNone(summary.verification_source)

            payload = json.loads(summary.report_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["market_key"], "CN-SZ")
            self.assertEqual(payload["exchange"], "SZSE")
            self.assertIsNone(payload["verification_source"])


class DisclosureCliTests(unittest.TestCase):
    def test_cli_run_prints_summary(self):
        runner = CliRunner()

        with tempfile.TemporaryDirectory() as temp_dir:
            fake_summary = DisclosureRunSummary(
                ticker="600519",
                start_date="2026-03-01",
                end_date="2026-03-27",
                cninfo_ingested_count=2,
                sse_fetched_count=2,
                matched_count=2,
                sse_only_count=0,
                cninfo_only_count=0,
                db_path=Path(temp_dir) / "cache" / "disclosure" / "disclosures.sqlite3",
                snapshot_root=Path(temp_dir) / "cache" / "disclosure_snapshots" / "cninfo",
                report_path=Path(temp_dir) / "runs" / "600519_2026-03-01_2026-03-27" / "run_summary.json",
            )
            fake_summary.report_path.parent.mkdir(parents=True, exist_ok=True)
            fake_summary.report_path.write_text("{}", encoding="utf-8")

            with patch("cli.disclosure.run_disclosure_pipeline", return_value=fake_summary):
                result = runner.invoke(
                    app,
                    [
                        "run",
                        "--ticker",
                        "600519",
                        "--start-date",
                        "2026-03-01",
                        "--end-date",
                        "2026-03-27",
                        "--data-root",
                        temp_dir,
                    ],
                )

            self.assertEqual(result.exit_code, 0)
            self.assertIn("600519", result.stdout)
            self.assertIn("CNInfo 入库: 2", result.stdout)
            self.assertIn("SSE 匹配: 2", result.stdout)


if __name__ == "__main__":
    unittest.main()
