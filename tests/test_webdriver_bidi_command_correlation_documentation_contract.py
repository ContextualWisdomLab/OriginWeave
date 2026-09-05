from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"
SOURCE = ROOT / "crates/originweave-network/src/webdriver_bidi_command_correlation.rs"


def test_command_correlation_release_record_matches_public_boundary() -> None:
    changelog = CHANGELOG.read_text(encoding="utf-8")
    source = SOURCE.read_text(encoding="utf-8")

    assert "Bounded WebDriver BiDi outstanding-command correlation" in changelog
    assert "MAX_WEBDRIVER_BIDI_OUTSTANDING_COMMANDS: usize = 256" in source
    assert "CommandKindMismatch" in source
    assert "UncorrelatableErrorResponse" in source
    assert "protocol ACK" not in changelog
