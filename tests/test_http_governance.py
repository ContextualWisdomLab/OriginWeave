"""Static safety, dependency, and documentation contracts for bounded HTTP authority."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/originweave-http"
SOURCE = CRATE / "src"


class HttpGovernanceTests(unittest.TestCase):
    """Keep HTTP parsing independent from transport, browser, and persistence authority."""

    def test_http_crate_contains_every_reviewed_module(self) -> None:
        """The reviewed HTTP boundary must remain decomposed into focused modules."""

        required = {
            "lib.rs",
            "error.rs",
            "policy.rs",
            "named_policy.rs",
            "target.rs",
            "field.rs",
            "request.rs",
            "response_head.rs",
            "response_head_checked.rs",
            "framing.rs",
            "chunked.rs",
            "content.rs",
            "integrity.rs",
            "mime.rs",
            "disposition.rs",
            "evidence.rs",
            "exchange.rs",
        }
        self.assertEqual({path.name for path in SOURCE.glob("*.rs")}, required)

    def test_http_manifest_has_only_reviewed_dependencies(self) -> None:
        """The authority kernel must not inherit a convenience HTTP client stack."""

        manifest = tomllib.loads((CRATE / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(
            set(manifest["dependencies"]),
            {
                "base64",
                "flate2",
                "originweave-core",
                "originweave-tls",
                "sha2",
            },
        )
        flate2 = manifest["dependencies"]["flate2"]
        self.assertEqual(flate2["version"], "=1.1.9")
        self.assertFalse(flate2["default-features"])
        self.assertEqual(flate2["features"], ["rust_backend"])
        self.assertEqual(
            set(manifest["dev-dependencies"]),
            {
                "originweave-destination",
                "originweave-network",
                "rcgen",
                "rustls",
            },
        )
        serialized = (CRATE / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn("reqwest", serialized)
        self.assertNotIn("hyper", serialized)

    def test_production_http_source_forbids_alternate_authority_paths(self) -> None:
        """HTTP semantics must consume authenticated TLS rather than create authority."""

        forbidden = {
            "TcpStream::connect": "socket connection",
            "connect_timeout": "socket connection",
            "ToSocketAddrs": "DNS or address selection",
            "lookup_host": "DNS lookup",
            "HTTP_PROXY": "ambient proxy",
            "HTTPS_PROXY": "ambient proxy",
            "ALL_PROXY": "ambient proxy",
            "reqwest": "general HTTP client",
            "hyper::client": "general HTTP connector",
            "File::create": "file persistence",
            "std::fs::write": "file persistence",
            "Command::new": "process execution",
            "COPILOT_GITHUB_TOKEN": "forbidden model credential",
        }
        combined = "\n".join(
            path.read_text(encoding="utf-8") for path in sorted(SOURCE.rglob("*.rs"))
        )
        for token, authority in forbidden.items():
            self.assertNotIn(token, combined, authority)

    def test_dependency_lock_contains_reviewed_decoder_family(self) -> None:
        """The lockfile must pin the reviewed portable decoder graph."""

        lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
        packages = {(item["name"], item["version"]): item for item in lock["package"]}
        self.assertIn(("originweave-http", "0.1.0"), packages)
        self.assertIn(("flate2", "1.1.9"), packages)
        self.assertIn("flate2", packages[("originweave-http", "0.1.0")]["dependencies"])
        self.assertTrue(packages[("flate2", "1.1.9")].get("checksum"))

    def test_http_crate_forbids_unsafe_and_requires_public_docs(self) -> None:
        """The parsing boundary must remain safe Rust with mandatory rustdoc."""

        lib = (SOURCE / "lib.rs").read_text(encoding="utf-8")
        self.assertIn("#![forbid(unsafe_code)]", lib)
        self.assertIn("#![deny(missing_docs)]", lib)
        combined = "\n".join(
            path.read_text(encoding="utf-8") for path in sorted(SOURCE.rglob("*.rs"))
        )
        self.assertNotIn("unsafe {", combined)
        self.assertNotIn("unsafe fn", combined)

    def test_http_coverage_scaffolding_is_explicit(self) -> None:
        """Production coverage must exclude assertion-only inline test scaffolding."""

        crate_root = (SOURCE / "lib.rs").read_text(encoding="utf-8")
        self.assertIn(
            "#![cfg_attr(coverage_nightly, feature(coverage_attribute))]",
            crate_root,
        )
        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        unexpected_cfgs = workspace["workspace"]["lints"]["rust"]["unexpected_cfgs"]
        self.assertEqual(unexpected_cfgs["level"], "warn")
        self.assertIn("cfg(coverage_nightly)", unexpected_cfgs["check-cfg"])

        for module_name in [
            "exchange_error_contract",
            "chunked_wire_budget_contract",
            "coverage_contract",
            "integrity_padding_contract",
            "mime_contract",
            "reachability_contract",
            "reason_phrase_contract",
            "region_contract",
            "security_contract",
            "trailer_error_contract",
        ]:
            self.assertIn(f'mod {module_name};', crate_root)

    def test_current_lineage_http_decisions_are_status_bearing_and_collision_free(self) -> None:
        """Rebuilt HTTP decisions must use current ADR numbers rather than stale collisions."""

        bounded = (ROOT / "docs/adr/0011-bounded-http11-semantics.md").read_text(
            encoding="utf-8"
        )
        reason = (ROOT / "docs/adr/0012-http-reason-phrase-diagnostics.md").read_text(
            encoding="utf-8"
        )
        for document in (bounded, reason):
            self.assertTrue(
                "- Status: Proposed" in document or "- Status: Accepted" in document
            )
            self.assertNotIn("TBD", document)
            self.assertNotIn("TODO", document)
        self.assertFalse((ROOT / "docs/adr/0007-bounded-http11-semantics.md").exists())
        self.assertFalse((ROOT / "docs/adr/0008-http-reason-phrase-diagnostics.md").exists())

    def test_http_design_and_doctoring_reference_current_lineage(self) -> None:
        """HTTP design evidence must remain present and point at current ADR authority."""

        design = (
            ROOT / "docs/superpowers/specs/2026-08-07-http11-semantics-design.md"
        ).read_text(encoding="utf-8")
        plan = (
            ROOT / "docs/superpowers/plans/2026-08-07-http11-semantics.md"
        ).read_text(encoding="utf-8")
        chunked = (ROOT / "docs/doctoring/http-chunked-message-boundary.md").read_text(
            encoding="utf-8"
        )
        reason = (ROOT / "docs/doctoring/http-reason-phrase-diagnostics.md").read_text(
            encoding="utf-8"
        )
        security = (ROOT / "docs/doctoring/http11-security-evidence.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("**Status:** Approved", design)
        self.assertIn("RFC 9110", design)
        self.assertIn("RFC 9112", design)
        self.assertIn("RFC 9530", design)
        self.assertIn("WHATWG", design)
        self.assertIn("exactly 100%", plan)
        self.assertIn("0011-bounded-http11-semantics.md", chunked)
        self.assertIn("ADR 0011", security)
        self.assertIn("0012-http-reason-phrase-diagnostics.md", reason)

    def test_root_doctoring_records_http_structured_fields_contract(self) -> None:
        """Material HTTP standards choices must remain in canonical doctoring."""

        doctoring = (ROOT / "docs/doctoring.md").read_text(encoding="utf-8")
        for marker in (
            "### Bounded HTTP semantics and digest-field interoperability",
            "RFC 9112",
            "RFC 9530",
            "RFC 8941",
            "RFC 9651",
            "obsoletes RFC 8941",
        ):
            self.assertIn(marker, doctoring)


if __name__ == "__main__":
    unittest.main()
