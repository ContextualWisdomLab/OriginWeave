"""Keep the bounded command-correlation release contract in native CI discovery."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"
SOURCE = ROOT / "crates/originweave-network/src/webdriver_bidi_command_correlation.rs"


class CommandCorrelationDocumentationTests(unittest.TestCase):
    """Enforce the correlation-owned release record without constraining other entries."""

    def test_command_correlation_release_record_matches_public_boundary(self) -> None:
        """The release record must retain its resource, provenance and authority bounds."""
        changelog = CHANGELOG.read_text(encoding="utf-8")
        source = SOURCE.read_text(encoding="utf-8")

        release_records = [
            line
            for line in changelog.splitlines()
            if line.startswith("- Bounded WebDriver BiDi outstanding-command correlation")
        ]
        self.assertEqual(len(release_records), 1)
        release_record = release_records[0]
        self.assertIn("at most 256 local ids", release_record)
        self.assertIn("exact typed command-family provenance", release_record)
        self.assertIn("events, null-id errors, and kind mismatches", release_record)
        self.assertIn(
            "performs no transport I/O or browser, policy, secret, or Agent authority grant",
            release_record,
        )

        self.assertIn("MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256", source)
        self.assertIn("CommandKindMismatch", source)
        self.assertIn("UncorrelatableErrorResponse", source)
