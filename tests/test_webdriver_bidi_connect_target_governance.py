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

    def test_changelog_records_exact_connected_peer_verification_boundary(self) -> None:
        """Verified socket-peer metadata must be visible without overstating transport trust."""
        changelog = CHANGELOG.read_text(encoding="utf-8")
        self.assertIn("Exact WebDriver BiDi socket-peer verification", changelog)
        self.assertIn("IP address and port", changelog)
        self.assertIn("does not authenticate an OS process", changelog)
        self.assertIn("does not negotiate TLS", changelog)


if __name__ == "__main__":
    unittest.main()
