"""Regression contract for the sensitive-handle reservation decision API."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SENSITIVE_DATA = ROOT / "crates" / "originweave-policy" / "src" / "sensitive_data.rs"


class SensitiveHandleUseMustUseContractTests(unittest.TestCase):
    """Keep the state-mutating reservation decision impossible to ignore silently."""

    def test_reserve_use_result_is_must_use(self) -> None:
        """The reservation API must warn callers that discard its authorization decision."""
        source = SENSITIVE_DATA.read_text(encoding="utf-8")
        declaration = re.compile(r"#\[must_use\]\s+pub fn reserve_use\(")
        self.assertRegex(source, declaration)


if __name__ == "__main__":
    unittest.main()
