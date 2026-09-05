"""Regression contracts for current and historical product-gap inventory evidence."""

from __future__ import annotations

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "docs" / "product-technical-gap-baseline.md"
CHANGELOG = ROOT / "CHANGELOG.md"


class GapSnapshotInventoryConsistencyTests(unittest.TestCase):
    """Keep volatile live truth separate from immutable dated snapshots."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.baseline = BASELINE.read_text(encoding="utf-8")
        cls.changelog = CHANGELOG.read_text(encoding="utf-8")

    def test_current_live_state_is_distinct_from_dated_snapshot(self) -> None:
        """The live section must bind fresh evidence without promoting queued work."""
        current = self.baseline.split("## Current live delivery state", 1)[1].split(
            "## Observed snapshot: 2026-08-29", 1
        )[0]
        for marker in (
            "125 open pull requests",
            "14 Ready/non-draft",
            "111 Draft",
            "13 open non-PR issues",
            "87c4daa1830bac5a5228b6036752ad5633232085",
            "18156473",
            "7 central required workflows",
            "codeql-pr",
            "Live GitHub PR/base/head/check APIs are authoritative over PR bodies",
            "Ready roots are #37 `28721edcc67bb5379ffb11a259d746396ac7ae03`, #50 `30d032b64c8eca669fa029a9d9915519cc467e99`, #166 `e84a1a2cc82b1c666218efd441da97849f47b8c2`, #219 `65e4315d80137badc0b55e1b9617015beb1db568`, #220 `e545b94e1de499b96b867694f80ac04ad247becd`, #229 `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6`, #240 `24930a3a9ee79c0b712ee3df6589b0592eb6e18f`, #248 `7d6db16b2ead201fcec320854923f90d3ad0d8bc`, #272 `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3`, #274 `802d0bdff7536d9ac253305d3e0237b4e4a1789e`, #285 `f455c2cd64b3dd3f027c91d396103792a205ddd0`, #287 `af83c40dd2990a03064a92ca75430a9cc400f098`, and #290 `ebeefcd534db4324498fdb18046ebc6255ddcdf2`",
            "PR #238's moving exact head is intentionally omitted",
            "PR #166 exact head `e84a1a2cc82b1c666218efd441da97849f47b8c2`",
            "PR #220 exact head `e545b94e1de499b96b867694f80ac04ad247becd`",
            "retain formal `CHANGES_REQUESTED` decisions",
            "every current review thread is resolved",
            "22 are queued and none is in progress",
            "rerunning would only add duplicate queue load",
            "PR #249 is Draft at exact head `017d6e816f5a86544a63821b3ceaba94d5f17f44`",
            "#250 `0eab23d5e388c5c8b984c0021a58316680c9ba8b`",
            "#251 `86e8ad76838f2a64aa7e0cd56ba1f931c8d0c3dc`",
            "#252 `2015259529ada99af836989079cc85a15779a2d8`",
            "#253 `0d72082e595c0e1fcc03d609ba337896ed14e2fc`",
            "#254 `cbaf50dcc97753cc73135497ea8225e8b18de190`",
            "#255 `a13de5f9321e72c1867974eb7a43230f031e58df`",
            "#256 `9f2e6f29be46371762e3031a97c1cac04720694f`",
            "#257 `ea2b5b78868917219c46f1304558b92490a7f6fe`",
            "Issue #279",
            "Issue #28 remains the P0 governed-browser integration target",
            "PR #260 is Draft at exact head `3a651967c421f77088fe25e86a63faae295390b3`",
            "PR #258 is Draft at exact head `f2ceabb3ea50b1959e936503c50cae12f3e6e480`",
            "PR #259 is Draft at exact head `e1105ddf86f6c79443af8b4d306b9d34cb703c17`",
            "PR #261 is Draft at exact head `323ac9e147691e9f6572711f5a748e13f1036624`",
            "Repair PR #277 is Draft at exact head `01038ba71fb276426cc67f90a91a3c431e194db5`",
            "PR #70 is Draft at exact head `77eb0f2ee71783e06171784b7173c0b4cd530e61`",
            "DDD/MCP repair #272 is Ready at exact head `b1cae8ad1cbd8eb6992037c830aea30b9aa436b3`",
            "PR #229 is Ready at exact head `024f63690cf05cfe6f0d4a430f0e18ea8fd2c4d6`",
            "PR #281 remains Draft at exact head `adaca6427d68f550b39293a69b7c733430d1c385`",
            "PR #282 is Draft at externally advanced exact head `b64e0708584beff3fb54acf226cb3e667773e473`",
            "PR #283 is Draft at exact head `c904300a6a1bda83af24f84d586f1c5f6a6491aa`",
            "PR #288 is Draft at exact head `39e36256651f62940ec3ca6149067f0cfcb2285a`",
            "PR #269 is Draft at exact head `7854394266d3f292e779193c01413a34f6798d7c`",
            "PR #270 is Draft at exact head `191a14535219ea8033777fa4c970efb281b62418`",
            "PR #271 is Draft at exact head `802ec806cdd4560eab48c484f435766ecabda353`",
            "PR #285 is Ready at exact head `f455c2cd64b3dd3f027c91d396103792a205ddd0`",
            "CI `33930234387`",
            "Security Scan `33924016851`",
            "SAST Semgrep `33924016903`",
            "CodeQL PR `33924016883`",
            "did not materialize fresh central required-workflow runs",
            "PR #287 is Ready at exact head `af83c40dd2990a03064a92ca75430a9cc400f098`",
            "PR #273 is Draft at exact head `e5c8fcb66bf644dfa750bb1b40ba3d600cb7805a`",
            "GitHub Releases is empty",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)
        self.assertNotRegex(
            current,
            re.compile(r"PR #238 exact head `[0-9a-f]{40}`"),
            "The self-referential baseline cannot pin the SHA produced by its own commit; read #238 live metadata instead.",
        )
        self.assertRegex(
            current,
            re.compile(r"Observed at \(UTC\): `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z`"),
        )
        for non_passing in (
            "queued reviewer evidence is non-passing",
            "discard queued, skipped, cancelled, absent, predecessor, synthetic, status-only, and model-only evidence",
        ):
            with self.subTest(non_passing=non_passing):
                self.assertIn(non_passing.casefold(), current.casefold())
        self.assertIn("binds Git rename/copy similarity to blob identity", current)
        self.assertNotIn("remaining rename/copy similarity-binding RED", self.changelog)
        for stale in (
            "Protected `main` is `c789b802fc98a8d7fd8c09d9327f36828054d2a1` through #280",
            "143 open pull requests",
            "142 open pull requests",
            "24 Ready/non-draft",
            "138 Draft",
            "121 Draft",
            "12 open non-PR issues",
            "10 central required workflows",
            "6 central required workflows",
            "PR #260 is Draft at exact head `a3741389fdc491c7ecccc20f77609c55bc56d20f`",
            "PR #261 remains stale-parent Draft at exact head `127e02503e48938e29a9a07410574c7e72fc661a`",
            "DDD/MCP repair #272 is Draft at exact head `80272f18422c9946077ad9bd674f603db8f020da`",
            "PR #229 is Draft at exact head `7ae426e760e8351ee792ce9df4266d7e7483d0d4`",
            "PR #282 is Draft at exact head `b2a120f892973e76c0ea0f06e7105bdf7a268009`",
            "PR #283 is Draft at exact head `a9603170e848a7c029531fe75727f02992ade2de`",
            "PR #282 is Draft at exact head `e3a40a4f78a3fbfc751ab9efad321b8207fb43e5`",
            "PR #283 is Draft at exact head `85d31596c8b7251135f773b1d54d0f656fa10bbf`",
            "PR #260 is Draft at exact head `56600a6fd982cfafd784f4b7bb659d918113ca90`",
            "PR #281 remains Draft at exact head `3ed5d7e8cf77547c96feff2cfb24c46d74a73ebb`",
        ):
            with self.subTest(stale=stale):
                self.assertNotIn(stale, current)

    def test_current_baseline_inventory_matches_the_verified_snapshot(self) -> None:
        """The dated 2026-08-29 snapshot must keep its exact historical inventory."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        for marker in (
            "108 open pull requests",
            "24 non-draft",
            "84 draft",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, current)

        for stale in (
            "128 open pull requests",
            "126 open pull requests",
            "54 non-draft",
            "72 draft",
            "115 open pull requests",
            "31 non-draft",
            "116 open pull requests",
            "32 non-draft",
            "111 open pull requests",
            "27 non-draft",
            "153 open pull requests",
            "114 draft PRs",
        ):
            with self.subTest(stale=stale):
                self.assertNotIn(stale, current)

    def test_unreleased_changelog_uses_one_current_inventory(self) -> None:
        """The current Unreleased entry must match the verified volatile inventory."""
        unreleased = self.changelog.split("## [Unreleased]", 1)[1]
        preamble, remainder = unreleased.split("### Added", 1)
        added = remainder.split("### Changed", 1)[0]

        expected = "125 open pull requests (13 ready, 112 draft)"
        self.assertIn(expected, preamble)
        self.assertIn("13 open non-PR issues", preamble)
        self.assertIn(expected, added)
        self.assertIn("13 open non-PR issues", added)
        self.assertNotIn("143 open pull requests", preamble)
        self.assertNotIn("108 open pull requests (24 ready, 84 draft)", preamble)

    def test_current_snapshot_records_recent_stack_merges_and_revalidation(self) -> None:
        """A stacked merge must update the dated queue and parent exact-head evidence."""
        current = self.baseline.split("### Open pull requests", 1)[1].split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[0]
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        self.assertIn("108 open pull requests", current)
        self.assertIn("24 non-draft", current)
        self.assertIn("#217 was squash-merged", record)
        self.assertIn(
            "#53 was exact head `4ecc81e59ae7bc3a640e65e2442bf30c079bd94c`",
            record,
        )
        self.assertIn(
            "#217 was exact head `6b8a3fdeae52ad94b90086bbc9b42863b90c9614`",
            record,
        )
        self.assertIn("66f360ccac5cec60c72222cc79d58e39f6f00088", record)
        self.assertIn("#67 was squash-merged", record)
        self.assertIn("5021d142583cb5a8e393248048bb824762a98056", record)
        self.assertIn("PR #64 consequently advanced", record)
        self.assertIn(
            "| #210 | Ready | `542ca1e9c0a863595b8b6697790005d2471f5413` | "
            "`7946dce9a3dd074047d93fca299d48c7aef40e47` |",
            self.baseline,
        )
        self.assertNotIn("| #217 |", current)

    def test_current_documentation_head_records_security_and_review_state(self) -> None:
        """The dated self-referential record must preserve its historical gate truth."""
        record = self.baseline.split(
            "#### 2026-08-29 maintenance-loop record", 1
        )[1].split("#### Current exact-head active PR evidence", 1)[0]
        for marker in (
            "The immediately preceding PR #238 head `d0b0d1ed92f891f14646fc673b8e1c0d912586fd` remains historical",
            "automatic OpenCode run `33193822920` / job `98926243116` failed closed",
            "current Strix run `33193822929` / job `98925769697` succeeded",
            "central dispatch run `33194506918` / job `98928580387` also failed closed at OpenCode",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, record)


if __name__ == "__main__":
    unittest.main()
