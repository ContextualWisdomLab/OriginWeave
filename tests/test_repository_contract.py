"""Repository contract tests that remain runnable before Rust compilation."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class RepositoryContractTests(unittest.TestCase):
    """Validate the non-generated repository contract."""

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


if __name__ == "__main__":
    unittest.main()
