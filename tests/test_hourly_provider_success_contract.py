"""Fail-closed proof that a green model attempt reached a successful provider response."""

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


class HourlyProviderSuccessContractTests(unittest.TestCase):
    """Do not accept process exit zero as upstream model success evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_green_attempt_requires_a_new_successful_provider_response(self) -> None:
        """A 2xx provider response after attempt start is required before success=true."""

        broker = _step_block(
            self.workflow, "Start loopback-only NVIDIA NIM credential broker"
        )
        for contract in (
            "LAST_SUCCESSFUL_REQUEST = 0",
            "global LAST_SUCCESSFUL_REQUEST",
            "if 200 <= status < 300:",
            '"last_successful_request": LAST_SUCCESSFUL_REQUEST',
        ):
            self.assertIn(contract, broker)

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        for contract in (
            '"last_successful_request",',
            "provider_success_before",
            "provider_successful",
            'if [ "$provider_successful" -le "$provider_success_before" ]; then',
            "cause=provider_success_not_observed",
        ):
            self.assertIn(contract, agent)

        success_branch = agent[
            agent.index('if [ "$status" -eq 0 ]; then') : agent.index(
                'if ! provider_status="$(read_provider_status)"; then'
            )
        ]
        self.assertLess(
            success_branch.index(
                'if [ "$provider_successful" -le "$provider_success_before" ]; then'
            ),
            success_branch.index("success=true"),
        )


if __name__ == "__main__":
    unittest.main()
