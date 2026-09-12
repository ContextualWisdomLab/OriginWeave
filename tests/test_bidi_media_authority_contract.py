from pathlib import Path


SOURCE = Path("crates/originweave-bidi/src/presentation_capabilities.rs")


def test_reduced_motion_capability_does_not_mint_unowned_command() -> None:
    source = SOURCE.read_text(encoding="utf-8")
    command_enum = source.split("pub enum WebDriverBidiPresentationCommand {", 1)[1].split(
        "/// Plan the reversible standard-BiDi presentation commands", 1
    )[0]

    assert "PresentationSurface::ReducedMotion" in source
    assert "SetReducedMotion" not in command_enum
