"""Regression contract for bounded workspace file comparison."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"


def _step_block(workflow: str, step_name: str) -> str:
    marker = f"      - name: {step_name}\n"
    _before, separator, remainder = workflow.partition(marker)
    if not separator:
        raise AssertionError(f"missing workflow step: {step_name}")
    return remainder.partition("\n      - name: ")[0]


class WorkspaceBundleSizeContractTests(unittest.TestCase):
    """Reject oversized untrusted files before any full comparison read."""

    def test_workspace_files_are_size_bounded_before_full_read(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        bundle = _step_block(
            workflow, "Validate and seal the credential-free change bundle"
        )
        constant = "MAX_CHANGED_FILE_BYTES = 1_048_576"
        stat = "path.stat().st_size > MAX_CHANGED_FILE_BYTES"
        reject = 'raise SystemExit(f"oversized workspace files rejected: {oversized}")'
        compare_read = "new_files[name].read_bytes()"
        for contract in (constant, stat, reject, compare_read):
            self.assertIn(contract, bundle)
        self.assertLess(bundle.index(constant), bundle.index(stat))
        self.assertLess(bundle.index(stat), bundle.index(reject))
        self.assertLess(bundle.index(reject), bundle.index(compare_read))


if __name__ == "__main__":
    unittest.main()
