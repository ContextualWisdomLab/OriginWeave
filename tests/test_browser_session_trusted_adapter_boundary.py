import pathlib
import re
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
BROWSER_SESSION_CARGO = ROOT / "crates/originweave-browser-session/Cargo.toml"
THREAT_MODEL = ROOT / "docs/THREAT_MODEL.md"
DOSSIER = ROOT / "docs/traceability/browser-session-trusted-adapter-boundary.md"
BROWSER_SESSION_SOURCE_ROOT = "crates/originweave-browser-session/src/"

# Any production reference to the lifecycle SPI outside the Browser Session owner is an explicit
# review surface. The reserved BiDi path is the only currently approved external production owner.
APPROVED_PRODUCTION_PORT_REFERENCES = {
    "crates/originweave-bidi/src/lifecycle_acl.rs",
}
APPROVED_BROWSER_SESSION_DEPENDENCIES = {
    "crates/originweave-bidi/Cargo.toml",
}

PORT_REFERENCE = re.compile(r"\bDisposableContextPort\b")
LIFECYCLE_BINDING = re.compile(r"\bbind_lifecycle_port\b")


def _has_port_reference(text: str) -> bool:
    return PORT_REFERENCE.search(text) is not None


def _has_lifecycle_binding(text: str) -> bool:
    return LIFECYCLE_BINDING.search(text) is not None


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

    def test_production_disposable_context_port_references_are_allowlisted(self) -> None:
        discovered = set()
        for path in ROOT.glob("crates/*/src/**/*.rs"):
            relative = path.relative_to(ROOT).as_posix()
            if relative.startswith(BROWSER_SESSION_SOURCE_ROOT):
                continue
            text = path.read_text(encoding="utf-8")
            if _has_port_reference(text):
                discovered.add(relative)

        unexpected = discovered - APPROVED_PRODUCTION_PORT_REFERENCES
        self.assertEqual(
            unexpected,
            set(),
            f"unreviewed production lifecycle-port references: {sorted(unexpected)}",
        )

    def test_product_sources_do_not_bind_a_caller_selected_lifecycle_port(self) -> None:
        callers = set()
        for path in ROOT.glob("crates/*/src/**/*.rs"):
            if path == ROOT / "crates/originweave-browser-session/src/browser_session.rs":
                continue
            text = path.read_text(encoding="utf-8")
            if _has_lifecycle_binding(text):
                callers.add(path.relative_to(ROOT).as_posix())

        self.assertEqual(
            callers,
            set(),
            "product composition must gain an explicit reviewed owner before binding a lifecycle port",
        )

    def test_scanners_cover_qualified_alias_and_ufcs_spellings(self) -> None:
        port_spellings = (
            "impl originweave_browser_session::DisposableContextPort for CandidatePort {}",
            (
                "use originweave_browser_session::DisposableContextPort as LifecyclePort;\n"
                "impl LifecyclePort for CandidatePort {}"
            ),
        )
        for source in port_spellings:
            self.assertTrue(
                _has_port_reference(source),
                f"production port spelling escaped review scanner: {source!r}",
            )

        binding_spellings = (
            "session.bind_lifecycle_port(port);",
            "BrowserSession::bind_lifecycle_port(session, port);",
            "session.bind_lifecycle_port (port);",
        )
        for source in binding_spellings:
            self.assertTrue(
                _has_lifecycle_binding(source),
                f"production lifecycle binding escaped review scanner: {source!r}",
            )

    def test_cross_file_alias_cannot_escape_dependency_review_surface(self) -> None:
        alias_only_source = "impl LifecyclePort for CandidatePort {}"
        dependency_manifest = (
            "[dependencies]\n"
            'originweave-browser-session = { path = "../originweave-browser-session" }\n'
        )

        self.assertFalse(_has_port_reference(alias_only_source))
        self.assertTrue(_has_browser_session_dependency(dependency_manifest))


if __name__ == "__main__":
    unittest.main()
