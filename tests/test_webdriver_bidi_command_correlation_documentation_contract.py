from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"
SOURCE = ROOT / "crates/originweave-network/src/webdriver_bidi_command_correlation.rs"


def test_command_correlation_release_record_matches_public_boundary() -> None:
    changelog = CHANGELOG.read_text(encoding="utf-8")
    source = SOURCE.read_text(encoding="utf-8")

    release_records = [
        line
        for line in changelog.splitlines()
        if line.startswith("- Bounded WebDriver BiDi outstanding-command correlation")
    ]
    assert len(release_records) == 1
    release_record = release_records[0]
    assert "at most 256 local ids" in release_record
    assert "exact typed command-family provenance" in release_record
    assert "events, null-id errors, and kind mismatches" in release_record
    assert "performs no transport I/O or browser, policy, secret, or Agent authority grant" in release_record

    assert "MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256" in source
    assert "CommandKindMismatch" in source
    assert "UncorrelatableErrorResponse" in source
