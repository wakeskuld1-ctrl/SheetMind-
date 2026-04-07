import tempfile
import unittest
from pathlib import Path

from tradingagents.dataflows.disclosure_models import DisclosureAttachment, DisclosureEvent
from tradingagents.dataflows.disclosure_store import DisclosureStore
from tradingagents.dataflows.disclosure_sse_verifier import (
    compare_sse_bulletins_with_store,
    normalize_sse_bulletin,
    parse_sse_jsonp_payload,
)


class SseJsonpParsingTests(unittest.TestCase):
    def test_parse_sse_jsonp_payload_extracts_inner_json(self):
        raw_text = (
            'jsonpCallback123({"result":[[{"SECURITY_CODE":"600519","TITLE":"测试公告",'
            '"SSEDATE":"2026-03-14","URL":"/disclosure/test.pdf"}]],"pageHelp":{"total":1}})'
        )

        payload = parse_sse_jsonp_payload(raw_text)

        self.assertEqual(payload["pageHelp"]["total"], 1)
        self.assertEqual(payload["result"][0][0]["SECURITY_CODE"], "600519")


class SseBulletinNormalizationTests(unittest.TestCase):
    def test_normalize_sse_bulletin_maps_row_to_comparable_shape(self):
        bulletin = normalize_sse_bulletin(
            {
                "SECURITY_CODE": "600519",
                "SECURITY_NAME": "贵州茅台",
                "TITLE": "贵州茅台关于回购股份实施进展的公告",
                "SSEDATE": "2026-03-04",
                "URL": "/disclosure/listedinfo/announcement/c/new/2026-03-04/600519_20260304_HMVE.pdf",
                "ORG_BULLETIN_ID": "6834195426960051",
                "BULLETIN_TYPE_DESC": "回购股份",
            }
        )

        self.assertEqual(bulletin.ticker_normalized, "600519.SH")
        self.assertEqual(
            bulletin.document_url,
            "https://www.sse.com.cn/disclosure/listedinfo/announcement/c/new/2026-03-04/600519_20260304_HMVE.pdf",
        )
        self.assertEqual(bulletin.title, "贵州茅台关于回购股份实施进展的公告")
        self.assertEqual(bulletin.document_date, "2026-03-04")


class SseStoreComparisonTests(unittest.TestCase):
    def test_compare_sse_bulletins_with_store_reports_match_and_sse_only_rows(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            store = DisclosureStore(Path(temp_dir) / "disclosures.sqlite3")
            store.upsert_event(
                DisclosureEvent(
                    event_id="cninfo-1224994303",
                    market="CN-A",
                    exchange="SSE",
                    ticker_raw="600519",
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
                    ingested_at="2026-03-27T15:00:00+08:00",
                    attachments=[
                        DisclosureAttachment(
                            name="1224994303.PDF",
                            url="https://static.cninfo.com.cn/finalpage/2026-03-04/1224994303.PDF",
                            media_type="application/pdf",
                        )
                    ],
                )
            )

            comparison = compare_sse_bulletins_with_store(
                store=store,
                ticker_normalized="600519.SH",
                start_date="2026-03-01",
                end_date="2026-03-27",
                sse_rows=[
                    {
                        "SECURITY_CODE": "600519",
                        "SECURITY_NAME": "贵州茅台",
                        "TITLE": "贵州茅台关于回购股份实施进展的公告",
                        "SSEDATE": "2026-03-04",
                        "URL": "/disclosure/listedinfo/announcement/c/new/2026-03-04/600519_20260304_HMVE.pdf",
                        "ORG_BULLETIN_ID": "6834195426960051",
                        "BULLETIN_TYPE_DESC": "回购股份",
                    },
                    {
                        "SECURITY_CODE": "600519",
                        "SECURITY_NAME": "贵州茅台",
                        "TITLE": "贵州茅台关于高级管理人员被实施留置的公告",
                        "SSEDATE": "2026-03-14",
                        "URL": "/disclosure/listedinfo/announcement/c/new/2026-03-14/600519_20260314_OY00.pdf",
                        "ORG_BULLETIN_ID": "6534195463662844",
                        "BULLETIN_TYPE_DESC": "其他重大事项",
                    },
                ],
            )

            self.assertEqual(len(comparison.matched_pairs), 1)
            self.assertEqual(len(comparison.sse_only), 1)
            self.assertEqual(len(comparison.cninfo_only), 0)
            self.assertEqual(
                comparison.sse_only[0].title,
                "贵州茅台关于高级管理人员被实施留置的公告",
            )


if __name__ == "__main__":
    unittest.main()
