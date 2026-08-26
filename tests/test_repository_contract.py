"""Repository contract tests that remain runnable before Rust compilation."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class RepositoryContractTests(unittest.TestCase):
    """Validate the non-generated repository and governance contract."""

    def test_workspace_declares_all_independently_reusable_crates(self) -> None:
        """The root workspace must expose every reusable policy kernel."""

        data = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(
            set(data["workspace"]["members"]),
            {
                "crates/originweave-core",
                "crates/originweave-policy",
                "crates/originweave-destination",
                "crates/originweave-network",
                "crates/originweave-tls",
                "crates/originweave-resource",
                "crates/originweave-evidence",
            },
        )

    def test_toolchain_is_pinned_to_current_project_baseline(self) -> None:
        """Reproducible builds require an explicit Rust patch version."""

        data = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))
        self.assertEqual(data["toolchain"]["channel"], "1.97.1")

    def test_required_architecture_and_governance_documents_exist(self) -> None:
        """A commercial repository must keep binding decisions discoverable."""

        required_paths = {
            "README.md",
            "ARCHITECTURE.md",
            "AGENTS.md",
            "CLAUDE.md",
            "CHANGELOG.md",
            "CONTRIBUTING.md",
            "SECURITY.md",
            "LICENSE",
            "docs/doctoring.md",
            "docs/product-roadmap.md",
            "docs/quality-gates.md",
            "docs/database-naming.md",
            "docs/registry-maintenance.md",
            "docs/adr/0001-chromium-compatibility-kernel.md",
            "docs/adr/0002-agent-safety-kernel.md",
            "docs/adr/0003-provenance-native-observation.md",
            "docs/adr/0004-resolved-destination-policy.md",
            "docs/adr/0005-direct-socket-binding.md",
            "docs/adr/0006-tls-server-identity.md",
            "docs/adr/0009-hourly-agent-credential-boundary.md",
            "docs/superpowers/specs/2026-08-06-resolved-destination-policy-design.md",
            "docs/superpowers/specs/2026-08-06-direct-socket-binding-design.md",
            "docs/superpowers/specs/2026-08-06-tls-server-identity-design.md",
            "docs/superpowers/plans/2026-08-06-resolved-destination-policy.md",
            "docs/superpowers/plans/2026-08-06-direct-socket-binding.md",
            "docs/superpowers/plans/2026-08-06-tls-server-identity.md",
        }
        missing = sorted(path for path in required_paths if not (ROOT / path).is_file())
        self.assertEqual(missing, [])

    def test_origin_identity_and_destination_safety_remain_distinct(self) -> None:
        """Documentation must never present origin parsing as an SSRF decision."""

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        roadmap = (ROOT / "docs/product-roadmap.md").read_text(encoding="utf-8")
        for relative, text in [
            ("README.md", readme),
            ("ARCHITECTURE.md", architecture),
            ("docs/product-roadmap.md", roadmap),
        ]:
            self.assertIn("origin", text.lower(), relative)
            self.assertIn("destination", text.lower(), relative)
            self.assertIn("SSRF", text, relative)
        self.assertIn("originweave-destination", readme)
        self.assertIn("originweave-destination", architecture)
        self.assertIn("DNS-rebinding", readme)
        self.assertIn("DNS answer expansion", architecture)

    def test_socket_authority_is_documented_as_a_separate_boundary(self) -> None:
        """Destination approval and operating-system peer proof must stay distinct."""

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        self.assertIn("originweave-network", readme)
        self.assertIn("originweave-network", architecture)
        self.assertIn("exact operating-system peer", readme)
        self.assertIn("direct-only", architecture)

    def test_tls_service_identity_is_documented_as_a_separate_boundary(self) -> None:
        """TCP peer equality must not be presented as authenticated HTTPS identity."""

        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        architecture = (ROOT / "ARCHITECTURE.md").read_text(encoding="utf-8")
        self.assertIn("originweave-tls", readme)
        self.assertIn("originweave-tls", architecture)
        self.assertIn("TLS service identity", readme)
        self.assertIn("TCP peer", architecture)
        self.assertIn("RFC 9525", architecture)

    def test_ci_validates_the_exact_pull_request_head(self) -> None:
        """Required PR evidence must validate the head commit, not only a merge ref."""

        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        exact_head = "${{ github.event.pull_request.head.sha || github.sha }}"
        self.assertGreaterEqual(workflow.count(f"ref: {exact_head}"), 2)
        self.assertIn(f"exact-coverage-{exact_head}", workflow)
        self.assertIn("permissions:\n  contents: read", workflow)
        self.assertNotIn("contents: write", workflow)

    def test_hourly_loop_uses_nvidia_nim_and_dedicated_publication_authority(self) -> None:
        """The product loop must use OpenCode/NIM without review or merge credentials."""

        workflow = (ROOT / ".github/workflows/hourly-product-development.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn('cron: "41 * * * *"', workflow)
        self.assertIn("NVIDIA_NIM_API_KEY", workflow)
        self.assertIn("OPENCODE_VERSION", workflow)
        self.assertIn("open_pull_request", workflow)
        self.assertIn("OPENCODE_PR_TOKEN", workflow)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", workflow)
        self.assertNotIn("PR_REVIEW_MERGE_TOKEN", workflow)
        self.assertNotIn("OPENCODE_APPROVE_TOKEN", workflow)
        self.assertNotIn("pull-requests: write", workflow)
        self.assertNotIn("contents: write", workflow)
        self.assertNotIn("gh pr merge", workflow)
        self.assertNotIn("gh pr review", workflow)

    def test_hourly_credential_boundary_has_an_architecture_decision(self) -> None:
        """Secret withholding, broker use, and publication authority must stay explicit."""

        decision = (
            ROOT / "docs/adr/0009-hourly-agent-credential-boundary.md"
        ).read_text(encoding="utf-8")
        for phrase in (
            "NVIDIA_NIM_API_KEY",
            "open_pull_request",
            "release_blocker",
            "dry_run",
            "127.0.0.1:8765",
            "credential broker",
            "egress-policy: block",
            "fingerprint",
            "OPENCODE_PR_TOKEN",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, decision)

    def test_generated_build_outputs_are_ignored(self) -> None:
        """Verification artifacts must never be staged as product source."""

        ignore = (ROOT / ".gitignore").read_text(encoding="utf-8").splitlines()
        self.assertIn("/target/", ignore)
        self.assertIn("__pycache__/", ignore)
        self.assertIn("*.py[cod]", ignore)

    def test_product_name_is_consistent_in_binding_documents(self) -> None:
        """Binding documents must not retain an earlier internal product name."""

        for relative in ["README.md", "ARCHITECTURE.md", "AGENTS.md", "CHANGELOG.md"]:
            text = (ROOT / relative).read_text(encoding="utf-8")
            self.assertIn("OriginWeave", text, relative)
            self.assertNotIn("TraceWeave", text, relative)
            self.assertNotIn("ProofRail", text, relative)

    def test_context_origin_dispatch_is_recorded_in_the_changelog(self) -> None:
        """The public origin-gated dispatch boundary must remain visible in release history."""

        changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        self.assertIn("dispatch_if_context_origin_current", changelog)

    def test_database_contract_requires_two_word_snake_case(self) -> None:
        """Persistent naming policy must include the mandated canonical form."""

        policy = (ROOT / "docs/database-naming.md").read_text(encoding="utf-8")
        self.assertIn("at least two semantic words", policy)
        self.assertIn("snake_case", policy)
        self.assertIn("agent_session", policy)


if __name__ == "__main__":
    unittest.main()
