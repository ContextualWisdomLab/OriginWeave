"""Repository contracts for WireGuard and IKEv2 profile support."""

from __future__ import annotations

import pathlib
import re
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates" / "originweave-vpn-profile"


class VpnProfileContractTests(unittest.TestCase):
    """Keep the VPN profile authority bounded, secret-safe, and discoverable."""

    def test_workspace_exposes_the_reusable_vpn_profile_crate(self) -> None:
        """Profile normalization must remain a reusable Rust control-plane crate."""

        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn(
            "crates/originweave-vpn-profile",
            workspace["workspace"]["members"],
        )
        self.assertTrue((CRATE / "Cargo.toml").is_file())
        self.assertTrue((CRATE / "src" / "lib.rs").is_file())

    def test_profile_boundary_never_executes_hooks_or_retains_raw_secrets(self) -> None:
        """Untrusted profile text may describe connectivity but not grant host authority."""

        source = (CRATE / "src" / "lib.rs").read_text(encoding="utf-8")
        for required in (
            "import_wireguard_profile",
            "parse_ikev2_profile",
            "parse_vpn_profile",
            "VpnSecretImporter",
            "SecretReference",
            "WireGuardPrivateKey",
            "WireGuardPresharedKey",
            "PreUp",
            "PostUp",
            "PreDown",
            "PostDown",
            "SaveConfig",
            "Table",
        ):
            with self.subTest(required=required):
                self.assertIn(required, source)
        for forbidden in (
            "std::process::Command",
            "Command::new",
            "private_key: String",
            "preshared_key: String",
        ):
            with self.subTest(forbidden=forbidden):
                self.assertNotIn(forbidden, source)

    def test_raw_secret_vocabulary_is_not_automatically_debuggable(self) -> None:
        """A derived Debug implementation must not copy raw credentials into diagnostics."""

        source = (CRATE / "src" / "lib.rs").read_text(encoding="utf-8")
        declaration = re.search(
            r"#\[derive\((?P<traits>[^)]*)\)\]\s*(?:///[^\n]*\n\s*)*pub enum VpnSecret",
            source,
            re.DOTALL,
        )
        self.assertIsNotNone(declaration)
        assert declaration is not None
        traits = {trait.strip() for trait in declaration.group("traits").split(",")}
        self.assertNotIn("Debug", traits)

    def test_secret_cleanup_authority_is_documented_consistently(self) -> None:
        """ADR and doctoring must describe the implemented reverse-order rollback hook."""

        source = (CRATE / "src" / "lib.rs").read_text(encoding="utf-8")
        adr = (
            ROOT / "docs" / "adr" / "0015-vpn-profile-authority.md"
        ).read_text(encoding="utf-8")
        doctoring = (
            ROOT / "docs" / "doctoring" / "vpn-profile-support.md"
        ).read_text(encoding="utf-8")

        self.assertIn("fn discard_secret", source)
        self.assertIn("rollback_imports", source)
        for stale_claim in (
            "Until disposal is represented in the trait",
            "trait does not expose rollback/disposal",
            "must currently own cleanup of partial second-pass secret imports",
            "directly to the trusted secret importer",
        ):
            with self.subTest(stale_claim=stale_claim):
                self.assertNotIn(stale_claim, adr + doctoring)
        for required_claim in (
            "discard_secret",
            "reverse order",
            "SecretCleanupFailed",
        ):
            with self.subTest(required_claim=required_claim):
                self.assertIn(required_claim, adr)
        self.assertIn("side-effect-free validation pass", doctoring)
        self.assertIn("reverse-order rollback", doctoring)

    def test_vpn_profile_decision_and_primary_source_doctoring_are_present(self) -> None:
        """The route and credential authority change requires an ADR and APA evidence."""

        self.assertTrue(
            (ROOT / "docs" / "adr" / "0015-vpn-profile-authority.md").is_file()
        )
        self.assertTrue(
            (ROOT / "docs" / "doctoring" / "vpn-profile-support.md").is_file()
        )


if __name__ == "__main__":
    unittest.main()
