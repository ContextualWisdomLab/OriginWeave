"""Regression contract: protected-main MCP tools/list shipment must be documented as shipped."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class McpShippedContractTests(unittest.TestCase):
    """Buyer-facing docs must not describe merged tools/list code as active-PR only."""

    def test_tools_list_code_is_present_on_protected_main(self) -> None:
        """The shipped tools/list boundary must exist in tree before docs claim it."""
        mcp_source = (ROOT / "crates/originweave-core/src/mcp.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("McpToolsListPage", mcp_source)
        self.assertIn("mcp_tools_list_page", mcp_source)
        self.assertTrue(
            (ROOT / "crates/originweave-core/tests/mcp_tools_list_cache.rs").is_file()
        )

    def test_readme_does_not_claim_merged_tools_list_is_active_pr_only(self) -> None:
        """README must not retain the pre-merge Active PR #170 wording after #170 merged."""
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        self.assertNotIn("Active PR #170", readme)
        self.assertNotIn("active `tools/list` refinement", readme)

    def test_readme_describes_shipped_tools_list_boundary(self) -> None:
        """README must describe both tools/call and tools/list as protected-main behavior."""
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("tools/call", readme)
        self.assertIn("tools/list", readme)
        self.assertIn("Protected main", readme)

    def test_changelog_does_not_claim_merged_tools_list_is_active_pr_only(self) -> None:
        """CHANGELOG must not retain the pre-merge Active PR #170 wording after #170 merged."""
        changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        self.assertNotIn("Active PR #170", changelog)

    def test_doctoring_does_not_claim_merged_mcp_foundations_are_active_pr_only(self) -> None:
        """The standards doctoring record must not retain pre-merge MCP maturity wording."""
        doctoring = (ROOT / "docs/doctoring.md").read_text(encoding="utf-8")
        self.assertNotIn("Active PR #168", doctoring)
        self.assertIn("merged through PR #168", doctoring)
        self.assertIn("`tools/list` discovery boundary merged through PR #170", doctoring)
        self.assertNotIn("request-metadata, discovery, OAuth", doctoring)


if __name__ == "__main__":
    unittest.main()
