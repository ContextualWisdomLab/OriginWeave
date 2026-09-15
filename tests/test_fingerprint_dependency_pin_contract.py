from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FINGERPRINT_MANIFEST = ROOT / "crates" / "originweave-fingerprint" / "Cargo.toml"
TLS_MANIFEST = ROOT / "crates" / "originweave-tls" / "Cargo.toml"


def _sha2_requirement(manifest: Path) -> str:
    text = manifest.read_text(encoding="utf-8")
    match = re.search(r'^sha2\s*=\s*"([^"]+)"\s*$', text, flags=re.MULTILINE)
    if match is None:
        raise AssertionError(f"sha2 dependency is missing from {manifest.relative_to(ROOT)}")
    return match.group(1)


class FingerprintDependencyPinContractTests(unittest.TestCase):
    def test_sha2_uses_the_existing_exact_workspace_resolution(self) -> None:
        fingerprint_requirement = _sha2_requirement(FINGERPRINT_MANIFEST)
        tls_requirement = _sha2_requirement(TLS_MANIFEST)

        self.assertRegex(tls_requirement, r"^=\d+\.\d+\.\d+$")
        self.assertEqual(fingerprint_requirement, tls_requirement)


if __name__ == "__main__":
    unittest.main()
