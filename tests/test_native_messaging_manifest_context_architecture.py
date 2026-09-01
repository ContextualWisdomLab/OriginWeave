"""Architectural fitness for native-messaging manifest ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class NativeMessagingManifestContextArchitectureTest(unittest.TestCase):
    """Keep Chrome native-host manifest behavior inside the Extension Policy context."""

    def test_manifest_behavior_belongs_to_extension_context(self) -> None:
        """Host-manifest parsing must not leak back into stable core contracts."""
        workspace = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('"crates/originweave-extension"', workspace)

        extension_source = ROOT / "crates" / "originweave-extension" / "src"
        self.assertTrue((extension_source / "native_messaging_manifest.rs").is_file())
        self.assertTrue((extension_source / "native_messaging_manifest_document.rs").is_file())

        core_source = ROOT / "crates" / "originweave-core" / "src"
        self.assertFalse((core_source / "native_messaging_manifest.rs").exists())
        self.assertFalse((core_source / "native_messaging_manifest_document.rs").exists())
        core_entry = (core_source / "root.rs").read_text(encoding="utf-8")
        self.assertNotIn("native_messaging_manifest", core_entry)


if __name__ == "__main__":
    unittest.main()
