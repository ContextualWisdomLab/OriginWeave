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
        """A bounded 2xx provider response after attempt start is required before success."""

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

        provider_exchange = broker[
            broker.index("response = connection.getresponse()") : broker.index(
                "self.send_body(\n                          response.status"
            )
        ]
        response_read = "response_body = response.read(MAX_RESPONSE + 1)"
        response_bound = "if len(response_body) > MAX_RESPONSE:"
        status_record = "record_provider_status(request_id, response.status)"
        for contract in (response_read, response_bound, status_record):
            self.assertIn(contract, provider_exchange)
        self.assertLess(provider_exchange.index(response_read), provider_exchange.index(response_bound))
        self.assertLess(provider_exchange.index(response_bound), provider_exchange.index(status_record))

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
