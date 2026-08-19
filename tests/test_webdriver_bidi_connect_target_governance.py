"""Governance regression for explicit WebDriver BiDi socket destinations."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"


class WebDriverBiDiConnectTargetGovernanceTests(unittest.TestCase):
    """Keep the active no-DNS transport boundary visible in release evidence."""

    def test_changelog_records_explicit_no_dns_connect_target_boundary(self) -> None:
        """The production connect-target slice must have a truthful Unreleased record."""
        changelog = CHANGELOG.read_text(encoding="utf-8")
        self.assertIn(
            "Explicit no-DNS WebDriver BiDi loopback connection targets",
            changelog,
        )
        self.assertIn("localhost", changelog)
        self.assertIn("no socket I/O", changelog)
        self.assertIn("no Agent authority", changelog)


if __name__ == "__main__":
    unittest.main()
