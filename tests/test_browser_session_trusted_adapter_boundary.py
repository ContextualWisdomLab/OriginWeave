import pathlib
import re
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BROWSER_SESSION_CARGO = ROOT / "crates/originweave-browser-session/Cargo.toml"
THREAT_MODEL = ROOT / "docs/THREAT_MODEL.md"
DOSSIER = ROOT / "docs/traceability/browser-session-trusted-adapter-boundary.md"

# Production adapter implementations are explicit review surfaces. Test doubles under
# crate `tests/` are intentionally outside this scan.
APPROVED_PRODUCTION_PORT_IMPLEMENTATIONS = {
    "crates/originweave-bidi/src/lifecycle_acl.rs",
}

PORT_IMPL = re.compile(r"impl(?:<[^{}]*>)?\s+DisposableContextPort\s+for\s+")


class BrowserSessionTrustedAdapterBoundaryTests(unittest.TestCase):
    """Keep the privileged lifecycle adapter inside the reviewed product TCB."""

    def test_browser_session_crate_is_internal_and_zone_c_is_trusted(self) -> None:
        cargo = BROWSER_SESSION_CARGO.read_text(encoding="utf-8")
        threat_model = THREAT_MODEL.read_text(encoding="utf-8")

        self.assertRegex(cargo, r"(?m)^publish\s*=\s*false\s*$")
        self.assertIn(
            "Zone C — Chromium browser process and privileged adapters",
            threat_model,
        )
        self.assertIn("trusted browser integration code", threat_model)

    def test_trusted_adapter_dossier_states_the_supported_security_boundary(self) -> None:
        dossier = DOSSIER.read_text(encoding="utf-8")

        for required in (
            "trusted computing base",
            "DisposableContextPort",
            "publish = false",
            "supply-chain compromise",
            "caller-selected production adapter",
            "request/completion correlation is not adapter authentication",
        ):
            self.assertIn(required, dossier)

    def test_production_disposable_context_port_implementations_are_allowlisted(self) -> None:
        discovered = set()
        for path in ROOT.glob("crates/*/src/**/*.rs"):
            text = path.read_text(encoding="utf-8")
            if PORT_IMPL.search(text):
                discovered.add(path.relative_to(ROOT).as_posix())

        unexpected = discovered - APPROVED_PRODUCTION_PORT_IMPLEMENTATIONS
        self.assertEqual(unexpected, set(), f"unreviewed production port implementations: {sorted(unexpected)}")

    def test_product_sources_do_not_bind_a_caller_selected_lifecycle_port(self) -> None:
        callers = set()
        for path in ROOT.glob("crates/*/src/**/*.rs"):
            if path == ROOT / "crates/originweave-browser-session/src/browser_session.rs":
                continue
            text = path.read_text(encoding="utf-8")
            if ".bind_lifecycle_port(" in text:
                callers.add(path.relative_to(ROOT).as_posix())

        self.assertEqual(
            callers,
            set(),
            "product composition must gain an explicit reviewed owner before binding a lifecycle port",
        )


if __name__ == "__main__":
    unittest.main()
