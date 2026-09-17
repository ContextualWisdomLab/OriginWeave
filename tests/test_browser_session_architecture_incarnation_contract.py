from pathlib import Path


ARCHITECTURE = Path("ARCHITECTURE.md")


def _browser_session_section() -> str:
    text = ARCHITECTURE.read_text(encoding="utf-8")
    return text.split("### `originweave-browser-session` (active PR)", 1)[1].split("## 6. Planned modules", 1)[0]


def test_browser_session_architecture_names_incarnation_as_aba_discriminator() -> None:
    section = _browser_session_section()

    assert "`BrowserSessionIncarnation`" in section
    assert "sequential ABA" in section
    assert "participates in authorization validation" in section
    assert "The isolation identity prevents distinct aggregate incarnations" not in section
