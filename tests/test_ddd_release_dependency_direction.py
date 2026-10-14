"""Architectural fitness for release-context dependency direction."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ReleaseDependencyDirectionTests(unittest.TestCase):
    """Keep the shared kernel independent of commercial release policy."""

    def test_shared_core_does_not_depend_outward_on_release_context(self) -> None:
        core_manifest = tomllib.loads(
            (ROOT / "crates/originweave-core/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertNotIn("originweave-release", core_manifest.get("dependencies", {}))

        core_root = (ROOT / "crates/originweave-core/src/root.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("originweave_release", core_root)
        self.assertNotIn("Temporary compatibility path", core_root)

    def test_release_context_does_not_depend_on_shared_core_in_reverse(self) -> None:
        release_manifest = tomllib.loads(
            (ROOT / "crates/originweave-release/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertNotIn("originweave-core", release_manifest.get("dependencies", {}))


if __name__ == "__main__":
    unittest.main()
