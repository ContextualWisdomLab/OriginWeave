"""Regression contract for the sensitive-handle reservation decision API."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
POLICY_LIB = ROOT / "crates" / "originweave-policy" / "src" / "lib.rs"
SENSITIVE_DATA = ROOT / "crates" / "originweave-policy" / "src" / "sensitive_data.rs"


class SensitiveHandleUseMustUseContractTests(unittest.TestCase):
    """Keep sensitive-handle authorization behind authoritative reservation state."""

    def test_reserve_use_result_is_must_use(self) -> None:
        """The reservation API must warn callers that discard its authorization decision."""
        source = SENSITIVE_DATA.read_text(encoding="utf-8")
        declaration = re.compile(r"#\[must_use\]\s+pub fn reserve_use\(")
        self.assertRegex(source, declaration)

    def test_caller_supplied_use_count_is_not_a_public_authorization_surface(self) -> None:
        """Only reservation state may expose an Authorized handle-use decision to callers."""
        policy_lib = POLICY_LIB.read_text(encoding="utf-8")
        sensitive_data = SENSITIVE_DATA.read_text(encoding="utf-8")

        export_block = re.search(
            r"pub use sensitive_data::\{(?P<body>.*?)\};",
            policy_lib,
            flags=re.DOTALL,
        )
        self.assertIsNotNone(export_block)
        exports = export_block.group("body")
        self.assertNotIn("HandleUseRequest", exports)
        self.assertNotIn("evaluate_handle_use", exports)
        self.assertNotRegex(sensitive_data, r"\bpub struct HandleUseRequest\b")
        self.assertNotRegex(sensitive_data, r"\bpub fn evaluate_handle_use\b")


if __name__ == "__main__":
    unittest.main()
