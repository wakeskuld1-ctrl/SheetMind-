import json
import tempfile
import unittest
from pathlib import Path

from tradingagents.dataflows.disclosure_cninfo import (
    build_cninfo_disclosure_event,
    resolve_cninfo_security_ref,
    write_cninfo_snapshot,
)


class CninfoSecurityResolutionTests(unittest.TestCase):
    def test_resolve_cninfo_security_ref_prefers_exact_code_match(self):
        payload = {
            "keyBoardList": [
                {
                    "code": "600519",
                    "orgId": "gssh0600519",
                    "plate": "sse",
                    "category": "A股",
                    "type": "shj",
                    "zwjc": "贵州茅台",
                }
            ]
        }

        security_ref = resolve_cninfo_security_ref("600519", payload)

        self.assertEqual(security_ref.sec_code, "600519")
        self.assertEqual(security_ref.org_id, "gssh0600519")
        self.assertEqual(security_ref.plate, "sse")
        self.assertEqual(security_ref.sec_name, "贵州茅台")


class CninfoSnapshotTests(unittest.TestCase):
    def test_write_cninfo_snapshot_persists_json_payload(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot = write_cninfo_snapshot(
                snapshot_root=Path(temp_dir),
                snapshot_kind="detail",
                identity_parts=["cninfo", "600519", "1225009431"],
                payload={"announcementId": "1225009431", "ok": True},
                source_url="https://www.cninfo.com.cn/new/announcement/bulletin_detail",
            )

            self.assertTrue(Path(snapshot.storage_path).exists())
            self.assertEqual(snapshot.source_name, "cninfo")
            self.assertEqual(snapshot.content_type, "application/json")

            content = json.loads(Path(snapshot.storage_path).read_text(encoding="utf-8"))
            self.assertEqual(content["announcementId"], "1225009431")
            self.assertTrue(content["ok"])


class CninfoDisclosureMappingTests(unittest.TestCase):
    def test_build_cninfo_disclosure_event_maps_detail_payload_to_contract(self):
        security_ref = resolve_cninfo_security_ref(
            "600519",
            {
                "keyBoardList": [
                    {
                        "code": "600519",
                        "orgId": "gssh0600519",
                        "plate": "sse",
                        "category": "A股",
                        "type": "shj",
                        "zwjc": "贵州茅台",
                    }
                ]
            },
        )
        detail_payload = {
            "announcement": {
                "secCode": "600519",
                "secName": "贵州茅台",
                "orgId": "gssh0600519",
                "announcementId": "1225009431",
                "announcementTitle": "贵州茅台关于高级管理人员被实施留置的公告",
                "announcementTime": 1773417600000,
                "adjunctUrl": "finalpage/2026-03-14/1225009431.PDF",
                "adjunctSize": 71,
                "adjunctType": "PDF",
                "announcementType": "01010501||010113||012399",
                "announcementContent": None,
                "important": False,
            },
            "fileUrl": "http://static.cninfo.com.cn/finalpage/2026-03-14/1225009431.PDF",
            "bulletinStatus": None,
            "columIdentify": "qw",
        }

        with tempfile.TemporaryDirectory() as temp_dir:
            snapshot = write_cninfo_snapshot(
                snapshot_root=Path(temp_dir),
                snapshot_kind="detail",
                identity_parts=["cninfo", "600519", "1225009431"],
                payload=detail_payload,
                source_url="https://www.cninfo.com.cn/new/announcement/bulletin_detail",
            )

            event = build_cninfo_disclosure_event(
                security_ref=security_ref,
                detail_payload=detail_payload,
                snapshot=snapshot,
            )

        self.assertEqual(event.event_id, "cninfo-1225009431")
        self.assertEqual(event.market, "CN-A")
        self.assertEqual(event.exchange, "SSE")
        self.assertEqual(event.ticker_normalized, "600519.SH")
        self.assertEqual(event.issuer_name, "贵州茅台")
        self.assertEqual(event.title, "贵州茅台关于高级管理人员被实施留置的公告")
        self.assertEqual(event.document_type, "pdf")
        self.assertEqual(
            event.document_url,
            "https://static.cninfo.com.cn/finalpage/2026-03-14/1225009431.PDF",
        )
        self.assertEqual(event.attachments[0].name, "1225009431.PDF")
        self.assertEqual(event.snapshot.source_name, "cninfo")


if __name__ == "__main__":
    unittest.main()
