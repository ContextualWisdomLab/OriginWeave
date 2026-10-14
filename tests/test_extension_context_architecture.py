"""Architectural fitness for the Extension Policy bounded context."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class ExtensionContextArchitectureTest(unittest.TestCase):
    """Keep Chromium extension adapter vocabulary out of stable browser authority contracts."""

    def test_native_messaging_adapter_has_dedicated_context(self) -> None:
        """Native-messaging integration belongs to the Extension Policy context, not core."""
        workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('"crates/originweave-extension"', workspace)

        extension_root = ROOT / "crates" / "originweave-extension"
        self.assertTrue((extension_root / "Cargo.toml").is_file())
        self.assertTrue((extension_root / "src" / "lib.rs").is_file())
        self.assertTrue((extension_root / "src" / "native_messaging.rs").is_file())

        core_root = ROOT / "crates" / "originweave-core" / "src"
        self.assertFalse((core_root / "native_messaging.rs").exists())
        core_entry = (core_root / "root.rs").read_text(encoding="utf-8")
        self.assertNotIn("mod native_messaging;", core_entry)

        extension_manifest = (extension_root / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('originweave-core = { path = "../originweave-core" }', extension_manifest)
        core_manifest = (ROOT / "crates" / "originweave-core" / "Cargo.toml").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("originweave-extension", core_manifest)


if __name__ == "__main__":
    unittest.main()
