from pathlib import Path


ARCHITECTURE = Path("ARCHITECTURE.md")


def _browser_session_section() -> str:
    text = ARCHITECTURE.read_text(encoding="utf-8")
    return text.split("### `originweave-browser-session` (active PR)", 1)[1].split("## 6. Planned modules", 1)[0]


def _security_boundaries_section() -> str:
    text = ARCHITECTURE.read_text(encoding="utf-8")
    return text.split("## 12. Security boundaries", 1)[1].split("## 13. Deployment topology", 1)[0]


def test_browser_session_architecture_names_incarnation_as_aba_discriminator() -> None:
    section = _browser_session_section()

    assert "`BrowserSessionIncarnation`" in section
    assert "sequential ABA" in section
    assert "participates in authorization validation" in section
    assert "The isolation identity prevents distinct aggregate incarnations" not in section


def test_security_boundary_binds_browser_session_incarnation() -> None:
    section = _security_boundaries_section()
    browser_session_boundary = next(
        line
        for line in section.splitlines()
        if line.startswith("- Disposable Browser Session mutation and destruction authority")
    )

    assert "`BrowserSessionIncarnation`" in browser_session_boundary
    assert "session, context, and epoch" in browser_session_boundary
