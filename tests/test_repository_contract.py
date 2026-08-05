"""Repository contract tests that remain runnable before Rust compilation."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class RepositoryContractTests(unittest.TestCase):
    """Validate the non-generated repository and governance contract."""

    def test_workspace_declares_all_safety_kernel_crates(self) -> None:
        """The root workspace must expose every independently reusable module."""

        data = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(
            set(data["workspace"]["members"]),
            {
                "crates/originweave-core",
                "crates/originweave-policy",
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
            "docs/adr/0001-chromium-compatibility-kernel.md",
            "docs/adr/0002-agent-safety-kernel.md",
            "docs/adr/0003-provenance-native-observation.md",
        }
        missing = sorted(path for path in required_paths if not (ROOT / path).is_file())
        self.assertEqual(missing, [])

    def test_hourly_loop_uses_nvidia_nim_and_never_copilot_token(self) -> None:
        """The product loop must use OpenCode and the approved NIM credential."""

        workflow = (ROOT / ".github/workflows/hourly-product-development.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn('cron: "41 * * * *"', workflow)
        self.assertIn("NVIDIA_NIM_API_KEY", workflow)
        self.assertIn("OPENCODE_VERSION", workflow)
        self.assertIn("open_pull_request", workflow)
        self.assertIn("PR_REVIEW_MERGE_TOKEN", workflow)
        self.assertNotIn("COPILOT_GITHUB_TOKEN", workflow)
        self.assertNotIn("pull-requests: write", workflow)
        self.assertNotIn("contents: write", workflow)

    def test_product_name_is_consistent_in_binding_documents(self) -> None:
        """Binding documents must not retain an earlier internal product name."""

        for relative in ["README.md", "ARCHITECTURE.md", "AGENTS.md", "CHANGELOG.md"]:
            text = (ROOT / relative).read_text(encoding="utf-8")
            self.assertIn("OriginWeave", text, relative)
            self.assertNotIn("TraceWeave", text, relative)
            self.assertNotIn("ProofRail", text, relative)

    def test_database_contract_requires_two_word_snake_case(self) -> None:
        """Persistent naming policy must include the mandated canonical form."""

        policy = (ROOT / "docs/database-naming.md").read_text(encoding="utf-8")
        self.assertIn("at least two semantic words", policy)
        self.assertIn("snake_case", policy)
        self.assertIn("agent_session", policy)


if __name__ == "__main__":
    unittest.main()
