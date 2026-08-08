"""Regression contract for bounded PR-message handling."""

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


class PrMessageSizeContractTests(unittest.TestCase):
    """Keep PR prose bounded before a full file read."""

    def test_pr_message_size_is_checked_before_full_read(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        bundle = _step_block(
            workflow, "Validate and seal the credential-free change bundle"
        )
        stat = "message_size = message_path.stat().st_size"
        bound = "if message_size > 65_536:"
        read = "message_bytes = message_path.read_bytes()"
        for contract in (
            stat,
            bound,
            'raise SystemExit("PR_MESSAGE.md is too large")',
            read,
        ):
            self.assertIn(contract, bundle)
        self.assertLess(bundle.index(stat), bundle.index(bound))
        self.assertLess(bundle.index(bound), bundle.index(read))


if __name__ == "__main__":
    unittest.main()
