"""Regression contract for the dated commercial-completion gap baseline."""

from __future__ import annotations

import pathlib
import re
import unittest
from unittest.mock import patch

from test_documentation_active_pr_evidence_contract import active_pr_row, bounded_section

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs/product-technical-gap-baseline.md"


class ProductCompletionGapContractTests(unittest.TestCase):
    """Keep the exact repository snapshot and completion tracks reviewable."""

    def test_ack_checkpoint_separates_local_repair_from_review_acceptance(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published intent acknowledgment verification",
            "#### Published repeated-control repair",
        )
        for marker in (
            "46db0045904f0289738df843d0a2f179c26673d3", "34186280263",
            "8eda96915dbbe4cc617f834267c7464689c2844d",
            "1406/14894/19018/1552", "pre-consumption", "resolved",
            "terminal SUCCESS", "CLEAN", "not shipped", "148 repository",
            "10d5e1ff78d46bbad004d4e0971d0ffceac757fa", "superseded",
            "6b6c90ed3919ea84b69527ab688a087eeb45224d",
            "1105/11574/14787/1218", "34188785932", "server-sent",
            "RED", "1010", "1011", "Actual Edge", "terminal SUCCESS",
            "6b5241c164f5283f8dd51b1846ef0e4dacec0b29",
            "587/4886/5881/748", "#293", "GitHub-rendered exact-head",
            "no clipping or overlap", "rustdoc visual acceptance remains unproven",
            "Viewport", "DevicePixelRatio", "TimeZone", "ReducedMotion",
        ):
            self.assertIn(marker, current)

    def test_repeated_control_checkpoint_keeps_operational_gaps_open(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published repeated-control repair",
            "#### Published deadline and Close-code repairs",
        )
        for marker in (
            "3a8e6f4f89db2f53c144adb3351c153d89adca58", "34184829970",
            "1104/11553/14758/1216", "20 closure", "64", "65th",
            "not hosted acceptance", "process exit", "profile cleanup",
            "not shipped", "fresh", "exhausted", "reused",
        ):
            self.assertIn(marker, current)

    def test_close_code_checkpoint_does_not_claim_hosted_acceptance(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published deadline and Close-code repairs",
            "#### Transport-closure verification and test-integrity repair",
        )
        for marker in (
            "8716b9d441960a112446c1f89ab417ad9abe28d2", "34183766437",
            "1101/11492/14662/1218", "18 closure", "142 repository",
            "not hosted acceptance", "repeated Ping/Pong", "1016–2999",
            "process exit", "profile cleanup", "not shipped",
        ):
            self.assertIn(marker, current)

    def test_closure_checkpoint_separates_verified_and_pending_heads(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "### Latest verified cut: 2026-09-08",
            "### Historical verified cut: 2026-09-07",
        )
        for marker in (
            "d126242c7198c447d0fab7983d529441340fd1c9",
            "07ef43ec71b6dbd8540629bf1df5a63b81541ee4",
            "363a78e36e7690e9ed5bf49829567e00e2ec5d59",
            "34179452950", "34180304951", "1090/11218/14328/1210",
            "126 open pull requests", "114 Draft", "14 open non-PR issues",
            "not protected-main acceptance", "operation-wide deadline",
            "repeated Ping/Pong", "no package release",
        ):
            self.assertIn(marker, current)

    def test_end_reply_checkpoint_binds_published_evidence(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published session-end reply binding: 05:35 UTC",
            "#### Published status-response repair: 04:45 UTC",
        )
        for owner, head, parent, coverage in (
            (251, "924ad97551750d4a901ded38b89488cc5438e54f", "#250 `bbdc6ace`", "1057/10840/13873/1194"),
            (252, "363a78e36e7690e9ed5bf49829567e00e2ec5d59", "#251 `924ad975`", "1064/10898/13947/1194"),
        ):
            row = active_pr_row(current, owner)
            for marker in (head, parent, coverage, "Published; Draft"):
                self.assertIn(marker, row)
        for marker in (
            "34085877650", "34087239755",
            "9f2249637f31916acf9874baec724ddedfe621e5b14fdb6f4bd850620e89f307",
            "not hosted acceptance", "retained receipts", "#255", "#292",
        ):
            self.assertIn(marker, current)

    def test_historical_evidence_cannot_replace_end_reply_checkpoint(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(
            text, "#### Published session-end reply binding: 05:35 UTC",
            "#### Published status-response repair: 04:45 UTC",
        )
        for owner in (251, 252):
            row = active_pr_row(current, owner)
            stale = text.replace(row, row.replace("Published; Draft", "Local only"), 1)
            with patch.object(pathlib.Path, "read_text", return_value=stale + row):
                with self.assertRaises(AssertionError):
                    self.test_end_reply_checkpoint_binds_published_evidence()

    def test_status_repair_checkpoint_binds_published_evidence(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published status-response repair: 04:45 UTC",
            "#### Published semantic-action adoption: 04:11 UTC",
        )
        for marker in (
            "bbdc6ace7a5932adf24836700f806850e6b230bc",
            "Published; Draft",
            "1043/10711/13717/1188",
            "34084134654",
            "e1fccefc6b56eabe653ae41377fbcec0aa89b4be1ada92dba73d3ef87e841029",
            "not hosted acceptance",
        ):
            self.assertIn(marker, current)

    def test_historical_evidence_cannot_replace_status_repair(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(
            text, "#### Published status-response repair: 04:45 UTC",
            "#### Published semantic-action adoption: 04:11 UTC",
        )
        stale = text.replace(current, current.replace("Published; Draft", "Local only"), 1)
        with patch.object(pathlib.Path, "read_text", return_value=stale + current):
            with self.assertRaises(AssertionError):
                self.test_status_repair_checkpoint_binds_published_evidence()

    def test_current_snapshot_checks_do_not_route_by_phrase_substrings(self) -> None:
        """Current-snapshot assertions must not depend on count-word substrings."""
        source = pathlib.Path(__file__).read_text(encoding="utf-8")
        brittle_condition = '"pull requests" in phrase or ' + '"draft" in phrase'
        self.assertNotIn(brittle_condition, source)

    def test_baseline_records_current_inventory_and_completion_issues(self) -> None:
        """The dated baseline must not retain superseded queue counts or omit buyer tracks."""
        text = BASELINE.read_text(encoding="utf-8")
        end_marker = "#### 2026-08-29 maintenance-loop record"
        self.assertIn(end_marker, text)
        current = text.split("## Observed snapshot: ", 1)[1].split(end_marker, 1)[0]

        for phrase in (
            "108 open pull requests",
            "24 non-draft",
            "84 draft",
            "2026-08-28 116-PR snapshot",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, current)

        for phrase in (
            "#198",
            "#199",
            "#200",
            "#201",
            "#202",
            "#203",
            "Shrink the open-PR queue",
            "durable WARC/PROV replay",
            "stable BAP/MCP runtime API",
            "signed cross-platform Chromium distribution",
            "enterprise control and experience plane",
            "commercial acceptance gate",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

        self.assertNotIn("Shrink the 109-PR queue", text)
        self.assertNotIn(
            "current-head hosted CI, security, Noema, scheduler, and OpenCode workflows were still regenerating at the recheck",
            text,
        )

        for stale_phrase in (
            "100 open pull requests",
            "22 non-draft",
            "78 draft",
            "148 open pull requests",
            "79 draft PRs",
            "40 non-draft",
            "110 draft",
            "150 open pull requests",
            "prior 150-PR snapshot",
            "128 open pull requests",
            "74 draft",
            "115 open pull requests",
            "31 non-draft",
        ):
            with self.subTest(stale_phrase=stale_phrase):
                self.assertNotIn(stale_phrase, current)

    def test_active_github_approval_rule_is_not_documented_as_bypassable(self) -> None:
        """An active counted-approval rule must stop merge without an eligible approver."""
        text = BASELINE.read_text(encoding="utf-8")

        self.assertIn("eligible non-author", text)
        self.assertIn("reviewer-provisioning gap", text)
        self.assertNotIn("owner-directed administrative merge", text)

    def test_current_snapshot_records_the_pr_284_admin_bypass_incident(self) -> None:
        """A bypassed main update must not be presented as a policy-compliant merge."""
        text = BASELINE.read_text(encoding="utf-8")
        current = text.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: ", 1
        )[0]

        for marker in (
            "87c4daa1830bac5a5228b6036752ad5633232085",
            "#284",
            "61bcf88c960c6c437ccd29b3fbb73cd4325f9e5a",
            "rule-suite `3948421709`",
            "`result: bypass`",
            "#215",
            "fresh native Rust and production-coverage checks now succeed",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

        self.assertNotIn("through #280", current)

    def test_current_snapshot_records_repaired_webdriver_bidi_lineage(self) -> None:
        """Only the latest cut may establish current foundation and transport heads."""
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Connection-bound text responses: 14:54 UTC",
            "#### Session repair and child adoption: 13:35 UTC",
        )
        self.assertIn("#195 `63997bcf555e2c5c8e91ba287734ffba3837a1b7`", current)
        self.assertIn("#267 `3346d8ecc72932b98ec495d9cc52d6e5727c3064`", current)
        self.assertIn("#268 `e567af9e678fd4791776df795e89ed666975e6c2`", current)
        for marker in (
            "4632f2df", "d6889c80", "e1188c86",
            "34040202356", "34040306372", "queued",
            "pointer and status", "not product-browser acceptance",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

    def test_latest_executable_queue_uses_current_ready_roots(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(
            text, "#### Published session-end reply binding: 05:35 UTC",
            "#### Published status-response repair: 04:45 UTC",
        )
        roots = [line for line in current.splitlines() if line.startswith("Ready roots:")]
        self.assertEqual(len(roots), 1)
        self.assertEqual(
            set(re.findall(r"#(\d+)", roots[0])),
            {"37", "50", "166", "219", "220", "229", "238", "240", "272", "274", "285", "287"},
        )
        self.assertIn("## Historical next executable queue", text)
        self.assertNotIn("## Next executable queue", text)

    def test_pointer_checkpoint_records_published_receipt_repair(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "##### Pointer receipt follow-up: 15:28 UTC",
            "#### Session repair and child adoption: 13:35 UTC",
        )
        for marker in (
            "#257 `9451fd8a23dec95b31749376bc78c2eaca977fe8`",
            "#258 `5417ce32ed957aa166807f1023647caccc2920cb`",
            "8193fcd5", "d9396f05", "588fe731", "0234b587",
            "2 → 2 → 1", "34041977863", "34042223733", "queued",
            "not product-browser acceptance", "outbound session authority",
            "#249", "#250", "descendant adoption",
        ):
            self.assertIn(marker, current)

    def test_historical_text_cannot_supply_missing_pointer_receipt_head(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        marker = "#258 `5417ce32ed957aa166807f1023647caccc2920cb`"
        mutated = text.replace(marker, "response head removed", 1)
        self.assertNotEqual(mutated, text)
        mutated += "\nHistorical evidence: " + marker
        with patch.object(pathlib.Path, "read_text", return_value=mutated):
            with self.assertRaises(AssertionError):
                self.test_pointer_checkpoint_records_published_receipt_repair()

    def test_descendant_checkpoint_records_exact_receipt_adoptions(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "##### Receipt descendant adoption: 16:17 UTC",
            "#### Session repair and child adoption: 13:35 UTC",
        )
        for marker in (
            "180c168ecbdcd5eb7a4ad14ab4a53e8670646bf7",
            "3807aabeb22f9622610c3c8d504d1c686d25d896",
            "ba100fbc39e1ac4f10ee4faade38418551bb8298",
            "121578a43adda12221a9ca9ab8ada0a4fd03efef",
            "34043222194", "34043664230", "34044193429", "34044857929",
            "queued", "144 Python", "subscription response provenance",
            "protected-main asset", "not product-browser acceptance",
        ):
            self.assertIn(marker, current)

    def test_historical_text_cannot_supply_missing_descendant_head(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        marker = "121578a43adda12221a9ca9ab8ada0a4fd03efef"
        mutated = text.replace(marker, "subscription head removed", 1)
        self.assertNotEqual(mutated, text)
        mutated += "\nHistorical evidence: " + marker
        with patch.object(pathlib.Path, "read_text", return_value=mutated):
            with self.assertRaises(AssertionError):
                self.test_descendant_checkpoint_records_exact_receipt_adoptions()

    def test_subscription_checkpoint_records_verified_repair_and_visual_gap(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "##### Subscription response repair: 16:37 UTC",
            "#### Session repair and child adoption: 13:35 UTC",
        )
        for marker in (
            "46ae62aa31e35c702cd61c16322d05c7a9c35da1", "122ca139", "8b1508c8",
            "0/2", "14 focused", "144 Python", "1222/12824/16453/1418",
            "34045953423", "queued", "Mac is locked", "visual inspection remains pending",
            "outbound session binding", "protected-main asset", "not product-browser acceptance",
        ):
            self.assertIn(marker, current)

    def test_observation_safeguards_bind_each_published_owner(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published observation safeguards: 03:00 UTC",
            "#### Published response and observation: 01:45 UTC",
        )
        for owner, head, parent, coverage in (
            (270, "8eda96915dbbe4cc617f834267c7464689c2844d", "3df2a631", "1346/14113/17928/1464"),
            (271, "b0410ae92bd20eaf31d09b7d49390e13cb045999", "8eda9691", "1393/14770/18899/1546"),
        ):
            row = active_pr_row(current, owner)
            for marker in ("Published; Draft", parent, coverage, f"/commit/{head}"):
                self.assertIn(marker, row)
        for marker in (
            "29cd0d66", "14efb678", "148 Python", "34076117534", "34077987302",
            "queued", "actual visual inspection", "not product-browser acceptance",
            "original connection", "status receipts", "#195/#279",
        ):
            self.assertIn(marker, current)

    def test_action_adoption_checkpoint_binds_published_owners(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published semantic-action adoption: 04:11 UTC",
            "#### Published observation safeguards: 03:00 UTC",
        )
        for owner, head, parent, coverage in (
            (93, "82056d13aa94c106060b84ee76be56fcb7787fc8", "b0410ae9", "1403/14852/18976/1552"),
            (95, "6b29d890245ed2f612c2998198f4e8c8a06da312", "82056d13", "1408/14897/19022/1552"),
            (96, "cbabf55c6a25b979fa0d9e3c1677338665975ab7", "6b29d890", "1410/14908/19030/1552"),
        ):
            row = active_pr_row(current, owner)
            for marker in ("Published; Draft", parent, coverage, f"/commit/{head}"):
                self.assertIn(marker, row)
        for marker in (
            "34079739018", "34080772063", "34081979602", "queued",
            "actual visual inspection", "not product-browser acceptance",
            "original connection", "status receipts", "publish = false",
        ):
            self.assertIn(marker, current)

    def test_historical_rows_cannot_replace_action_adoption(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(text, "#### Published semantic-action adoption: 04:11 UTC", "#### Published observation safeguards: 03:00 UTC")
        for owner in (93, 95, 96):
            row = active_pr_row(current, owner)
            for replacement in ("", row.replace("Published; Draft", "Local only; Draft")):
                mutated = text.replace(row, replacement, 1) + "\nHistorical evidence:\n" + row
                with patch.object(pathlib.Path, "read_text", return_value=mutated):
                    with self.assertRaises(AssertionError):
                        self.test_action_adoption_checkpoint_binds_published_owners()

    def test_historical_rows_cannot_replace_observation_safeguards(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(text, "#### Published observation safeguards: 03:00 UTC", "#### Published response and observation: 01:45 UTC")
        for owner in (270, 271):
            row = active_pr_row(current, owner)
            for replacement in ("", row.replace("Published; Draft", "Local only; Draft")):
                mutated = text.replace(row, replacement, 1) + "\nHistorical evidence:\n" + row
                with patch.object(pathlib.Path, "read_text", return_value=mutated):
                    with self.assertRaises(AssertionError):
                        self.test_observation_safeguards_bind_each_published_owner()

    def test_response_observation_checkpoint_binds_each_published_owner(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published response and observation: 01:45 UTC",
            "#### Published input descendants: 00:40 UTC",
        )
        for owner, head, parent, coverage in (
            (268, "ff27220cb5eb4d11ca1dc5614a4181e1a397a3f1", "ebd507ae", "1325/13907/17682/1456"),
            (269, "3df2a631bacd7109b3982fdd7ac599d0bd92a589", "ff27220c", "1338/14025/17843/1460"),
        ):
            row = active_pr_row(current, owner)
            for marker in ("Published; Draft", parent, coverage):
                self.assertIn(marker, row)
            self.assertIn(f"[`{head[:8]}`](https://github.com/ContextualWisdomLab/OriginWeave/commit/{head})", row)
        for marker in ("658fb676", "49d18f5f", "147 Python", "34072645796", "34073733364", "34073733355", "queued", "actual visual inspection", "not product-browser acceptance", "#270"):
            self.assertIn(marker, current)

    def test_historical_rows_cannot_replace_response_observation_checkpoint(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(text, "#### Published response and observation: 01:45 UTC", "#### Published input descendants: 00:40 UTC")
        for owner in (268, 269):
            row = active_pr_row(current, owner)
            for replacement in ("", row.replace("Published; Draft", "Local only; Draft")):
                mutated = text.replace(row, replacement, 1) + "\nHistorical evidence:\n" + row
                with patch.object(pathlib.Path, "read_text", return_value=mutated):
                    with self.assertRaises(AssertionError):
                        self.test_response_observation_checkpoint_binds_each_published_owner()

    def test_input_descendants_bind_publication_and_coverage_to_each_owner(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published input descendants: 00:40 UTC",
            "### Latest verified cut: 2026-09-06",
        )
        for owner, head, parent, coverage in (
            (265, "e94a2372fe3771f9ddf70291d34fc9a7e4770ec9", "43395711", "1297/13618/17315/1442"),
            (266, "e3885f69df2cf3899184209efdee5b11bba1bd86", "e94a2372", "1310/13766/17524/1452"),
            (267, "ebd507ae56c3064e3cae5566502f539c20618a8f", "e3885f69", "1318/13852/17610/1456"),
        ):
            row = active_pr_row(current, owner)
            for marker in ("Published; Draft", parent, coverage):
                self.assertIn(marker, row)
            self.assertIn(
                f"[`{head[:8]}`](https://github.com/ContextualWisdomLab/OriginWeave/commit/{head})",
                row,
            )
        for marker in (
            "e7fb1527", "009f9a41", "4e020e16", "23 focused", "145 Python",
            "34068277243", "34068882527", "34070121583", "queued",
            "zero unresolved review threads", "one counted approval", "seven required workflows",
            "actual screenshots", "not product-browser acceptance", "own checks and visual inspection",
            "#268", "protected-main asset preservation", "status receipt",
        ):
            self.assertIn(marker, current)

    def test_historical_rows_cannot_hide_current_input_evidence_changes(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(
            text, "#### Published input descendants: 00:40 UTC",
            "### Latest verified cut: 2026-09-06",
        )
        for owner in (265, 266, 267):
            row = active_pr_row(current, owner)
            for replacement in ("", row.replace("Published; Draft", "Local only; Draft")):
                with self.subTest(owner=owner, replacement=replacement):
                    mutated = text.replace(row, replacement, 1) + "\nHistorical evidence:\n" + row
                    self.assertNotEqual(text, mutated)
                    with patch.object(pathlib.Path, "read_text", return_value=mutated):
                        with self.assertRaises(AssertionError):
                            self.test_input_descendants_bind_publication_and_coverage_to_each_owner()

    def test_published_descendants_bind_evidence_to_current_owner_rows(self) -> None:
        current = bounded_section(
            BASELINE.read_text(encoding="utf-8"),
            "#### Published subscription descendants: 23:27 UTC",
            "#### Connection-bound text responses: 14:54 UTC",
        )
        for owner, head, parent, coverage in (
            (263, "4868d3e9f19133ac3382ee8532878aef27468893", "46ae62aa", "1244/13053/16729/1422"),
            (264, "433957117ad9e29b26715b062f5adcc9789744ba", "4868d3e9", "1282/13461/17140/1440"),
        ):
            row = active_pr_row(current, owner)
            for marker in ("Published; Draft", head, parent, coverage):
                self.assertIn(marker, row)
            self.assertIn(
                f"[`{head[:8]}`](https://github.com/ContextualWisdomLab/OriginWeave/commit/{head})",
                row,
            )
        for marker in (
            "zero unresolved review threads", "supersedes the older sole-#147",
            "12 Ready candidates remain BLOCKED", "one counted approval", "seven required workflows",
            "13 focused", "145 Python", "actual in-app screenshots", "not product-browser acceptance",
            "34065055213", "34066516991", "34066516992", "queued",
            ".github#1929", "protected-main asset preservation", "own checks and visual inspection",
        ):
            self.assertIn(marker, current)

    def test_historical_rows_cannot_hide_current_descendant_evidence_changes(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        current = bounded_section(
            text,
            "#### Published subscription descendants: 23:27 UTC",
            "#### Connection-bound text responses: 14:54 UTC",
        )
        for owner in (263, 264):
            row = active_pr_row(current, owner)
            for replacement in ("", row.replace("Published; Draft", "Local only; Draft")):
                with self.subTest(owner=owner, replacement=replacement):
                    mutated = text.replace(row, replacement, 1) + "\nHistorical evidence:\n" + row
                    self.assertNotEqual(text, mutated)
                    with patch.object(pathlib.Path, "read_text", return_value=mutated):
                        with self.assertRaises(AssertionError):
                            self.test_published_descendants_bind_evidence_to_current_owner_rows()

    def test_historical_text_cannot_supply_missing_subscription_repair_head(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        marker = "46ae62aa31e35c702cd61c16322d05c7a9c35da1"
        mutated = text.replace(marker, "subscription repair head removed", 1)
        self.assertNotEqual(mutated, text)
        mutated += "\nHistorical evidence: " + marker
        with patch.object(pathlib.Path, "read_text", return_value=mutated):
            with self.assertRaises(AssertionError):
                self.test_subscription_checkpoint_records_verified_repair_and_visual_gap()

    def test_historical_prose_cannot_hide_latest_root_or_lineage_removal(self) -> None:
        text = BASELINE.read_text(encoding="utf-8")
        cases = (
            (
                "#195 `63997bcf555e2c5c8e91ba287734ffba3837a1b7`",
                "foundation evidence removed",
                self.test_current_snapshot_records_repaired_webdriver_bidi_lineage,
            ),
            (
                "Ready roots: #37, #50,",
                "Ready roots: #37,",
                self.test_latest_executable_queue_uses_current_ready_roots,
            ),
        )
        for original, replacement, check in cases:
            with self.subTest(original=original):
                mutated = text.replace(original, replacement, 1)
                self.assertNotEqual(mutated, text)
                with patch.object(pathlib.Path, "read_text", return_value=mutated):
                    with self.assertRaises(AssertionError):
                        check()

    def test_historical_snapshot_preserves_webdriver_bidi_lineage(self) -> None:
        """The active stack must retain exact heads and the macOS race boundary."""
        text = BASELINE.read_text(encoding="utf-8")
        current = text.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: ", 1
        )[0]

        for marker in (
            "#195 is Draft at exact head `48eb2d23009c1c804520dd5efcd0d4d072aacef1`",
            "50 consecutive focused regression passes",
            "#242 is Draft at exact head `2d0e9f69df9ade21d8e8e3d807c3ff644d83b310`",
            "PR #243 adopts that corrected parent by ordinary merge at `97fab641ed9d76e6c515eadcef0629edfc8064a3`",
            "#93 is Draft at exact head `0664f0452cb329cd692cce7f61f9001652abfda2`",
            "SemanticNodeActionBinding",
            "does not authorize policy or execute input",
            "#95 is Draft at exact head `97aa0f2e340ee6fd920d0418f97af276b190554f`",
            "Only `Decision::Allow`",
            "`NotAdmitted`",
            "#96 is Draft at exact head `b7ba8dd1433410cee43084a73e31816da841b2a2`",
            "callback is never invoked",
            "#101 is Draft at exact head `bc810b121bb0303f55afa8777a23cc0f9748c1db`",
            "known-disabled interactive action",
            "#102 is Draft at exact head `a123c55d4839dae1db7e6671f7d4d158c7cfd9db`",
            "ObservationAuthorityMismatch",
            "#103 is Draft at exact head `8b3416169346fa04b53c915b813d55ccf47d1876`",
            "browser authority first",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

        self.assertIn(
            "| #195 | Draft | `6922dd98779e8f8aad132a3b1f563d7ba6e6d070` | "
            "`48eb2d23009c1c804520dd5efcd0d4d072aacef1` |",
            text,
        )
        self.assertIn(
            "| #93 | Draft | `802ec806cdd4560eab48c484f435766ecabda353` | "
            "`0664f0452cb329cd692cce7f61f9001652abfda2` |",
            text,
        )
        self.assertIn(
            "| #242 | Draft | `48eb2d23009c1c804520dd5efcd0d4d072aacef1` | "
            "`55fef0c3fae1724eddada53e52c4a0311f509aa3` |",
            text,
        )
        self.assertIn(
            "| #95 | Draft | `0664f0452cb329cd692cce7f61f9001652abfda2` | "
            "`97aa0f2e340ee6fd920d0418f97af276b190554f` |",
            text,
        )
        self.assertIn(
            "| #96 | Draft | `97aa0f2e340ee6fd920d0418f97af276b190554f` | "
            "`b7ba8dd1433410cee43084a73e31816da841b2a2` |",
            text,
        )
        self.assertIn(
            "| #101 | Draft | `b7ba8dd1433410cee43084a73e31816da841b2a2` | "
            "`bc810b121bb0303f55afa8777a23cc0f9748c1db` |",
            text,
        )
        self.assertIn(
            "| #102 | Draft | `bc810b121bb0303f55afa8777a23cc0f9748c1db` | "
            "`a123c55d4839dae1db7e6671f7d4d158c7cfd9db` |",
            text,
        )
        self.assertIn(
            "| #103 | Draft | `a123c55d4839dae1db7e6671f7d4d158c7cfd9db` | "
            "`8b3416169346fa04b53c915b813d55ccf47d1876` |",
            text,
        )

    def test_current_snapshot_records_origin_bound_socket_ports(self) -> None:
        """The live network root must retain its exact port-authority repair."""
        text = BASELINE.read_text(encoding="utf-8")
        current = text.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: ", 1
        )[0]

        for marker in (
            "PR #50 is Ready at exact head `ad87cfea59db711cb29ef90559790ba77e22029f`",
            "Fresh independent verification passed all seven fresh-resolution tests and 152 Python contracts",
            "origin-approved IP could be paired with a different service port",
            "binds the socket port to the effective scheme-host-port origin",
            "function/line/region/branch coverage pass locally at 100%",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

    def test_current_snapshot_records_content_length_persistence_repair(self) -> None:
        """A declared body boundary must not be confused with transport closure."""
        text = BASELINE.read_text(encoding="utf-8")
        current = text.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: ", 1
        )[0]

        for marker in (
            "PR #37 is Ready at exact head `1e2f41072854edcdbaf0f9ecf14697a3bfd62195`",
            "returns after the exact declared bytes",
            "already-buffered surplus remains fail-closed",
            "does not require TLS EOF",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

    def test_changelog_marks_superseded_warc_head_as_historical(self) -> None:
        """A predecessor WARC head must not look like the current exact evidence."""
        text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")

        self.assertIn("`d83748a70bd1b16dbfec46007fe02989ba6ce188` was superseded", text)
        self.assertIn("0341079331f9cea669eb9a5cc21842fd6027431e", text)

    def test_baseline_records_central_required_workflow_failures(self) -> None:
        """The baseline must preserve exact central workflow provenance and failures."""
        text = BASELINE.read_text(encoding="utf-8")

        for phrase in (
            "central `.github` repository",
            "repository ID `1274066402`",
            "close-empty-pr",
            "opencode-review",
            "pr-review-merge-scheduler",
            "security-scan",
            "strix",
            "sast-semgrep",
            "noema-review",
            "OpenCode current-head verdict",
            "Strix provider/backend",
            "internal server error",
            "33177641855",
            "33177641888",
            "33182772296",
            "MODEL_OUTPUT_UNAVAILABLE",
            "model pool exhausted",
            "33182749298",
            "#1391",
            "e4ba6b599cd1e50d0139762885682607b731655d",
            "did not prove missing workflow identities",
            "does not authorize bypass",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, text)

    def test_same_day_prior_inventory_is_bound_to_the_maintenance_record(self) -> None:
        """A same-day comparison must retain its exact prior observation in the record."""
        text = BASELINE.read_text(encoding="utf-8")
        record_end = "#### Current exact-head active PR evidence"
        self.assertIn(record_end, text)
        record = text.split("#### 2026-08-29 maintenance-loop record", 1)[1].split(
            record_end, 1
        )[0]
        self.assertIn("115 open pull requests (31 ready, 84 draft)", record)

    def test_historical_pr_219_head_is_a_full_exact_sha(self) -> None:
        """The retained PR #219 regression anchor must identify its real commit."""
        text = BASELINE.read_text(encoding="utf-8")

        self.assertIn(
            "| #219 | Ready | `0841d2ab3d8b5e60a03c0a8e818cf438e2716829` | "
            "`8145d40f1b028a8f4dc7e7da47ac89bb9e5bb2c7` |",
            text,
        )

    def test_issue_table_distinguishes_open_issues_from_governance_signals(self) -> None:
        """The table total must distinguish product issues from governance signals."""
        text = BASELINE.read_text(encoding="utf-8")
        table_end = "## Buyer-visible and technical gap matrix"
        self.assertIn(table_end, text)
        table = text.split("### Open issues and governance signals", 1)[1].split(
            table_end, 1
        )[0]
        self.assertIn("11 open issues plus 2 governance signals", table)
        self.assertIn("Issue or signal", table)

    def test_evidence_commands_reproduce_inventory_checks_and_review_state(self) -> None:
        """The evidence procedure must paginate the queue and inspect each exact PR head."""
        text = BASELINE.read_text(encoding="utf-8")
        evidence = text.split("## Evidence commands", 1)[1].split("\n## ", 1)[0]
        shell = evidence.split("```bash", 1)[1].split("```", 1)[0]

        for phrase in (
            "--paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/pulls?state=open&per_page=100'",
            "set -euo pipefail",
            'EVIDENCE_DIR="$(mktemp -d /tmp/originweave-evidence.XXXXXX)"',
            '"$EVIDENCE_DIR/open-pr-pages.json"',
            "jq '[.[][]]' \"$EVIDENCE_DIR/open-pr-pages.json\"",
            "--paginate --slurp 'repos/ContextualWisdomLab/OriginWeave/issues?state=open&per_page=100'",
            '"$EVIDENCE_DIR/open-issue-pages.json"',
            'map(select(has("pull_request") | not))',
            "open_non_pr_issues",
            '"repos/ContextualWisdomLab/OriginWeave/pulls/$PR"',
            '"repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/check-runs?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/commits/$HEAD_SHA/statuses?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/pulls/$PR/reviews?per_page=100"',
            '"repos/ContextualWisdomLab/OriginWeave/actions/runs?head_sha=$HEAD_SHA&per_page=100"',
            "check_runs: [$checks[][].check_runs[]?],",
            "legacy_statuses: [$statuses[][][]?]",
            "workflow_runs: [$workflow_runs[][].workflow_runs[]?],",
            "reviewThreads(first: 100, after: $endCursor)",
            "rules/branches/main?per_page=100",
            '"$EVIDENCE_DIR/main-branch-rule-pages.json"',
            '"$EVIDENCE_DIR/collaborator-pages.json"',
            '"$EVIDENCE_DIR/collaborators.json"',
            '"$EVIDENCE_DIR/pr-${PR}-merge-verdict.json.tmp"',
            '.state == "APPROVED"',
            ".submitted_at != null",
            ".commit_id == $head",
            "group_by(.reviewer)",
            "required_approving_review_count",
            "require_last_push_approval",
            "last_push_approval_authority",
            '"github_rule_evaluation_required"',
            "if $pull_request_parameters.require_last_push_approval == true then false",
            "$pr[0].user.login",
            '.type == "workflows"',
            ".parameters.workflows",
            "required_status_checks",
            '"$EVIDENCE_DIR/pr-${PR}-merge-verdict.json"',
            "for ATTEMPT in 1 2 3; do",
            "RECHECKED_HEAD_SHA=",
            "RECHECKED_BASE_SHA=",
            'if [[ "$RECHECKED_HEAD_SHA" == "$HEAD_SHA" && "$RECHECKED_BASE_SHA" == "$BASE_SHA" ]]; then',
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, shell)

        self.assertNotIn("while :; do", shell)
        self.assertNotIn("/tmp/originweave-open-pr", shell)
        self.assertNotIn("check_runs: [$checks[]?.check_runs[]?],", shell)
        self.assertNotIn("legacy_statuses: [$statuses[][]?]", shell)
        self.assertNotIn("workflow_runs: [$workflow_runs[]?.workflow_runs[]?],", shell)
        self.assertNotIn("$reviews[][]?\n          | select(.state", shell)
        self.assertNotIn("head-commit.json", shell)
        self.assertNotIn("$head_commit[0].committer.login", shell)
        self.assertNotIn("$head_commit[0].author.login", shell)


if __name__ == "__main__":
    unittest.main()
