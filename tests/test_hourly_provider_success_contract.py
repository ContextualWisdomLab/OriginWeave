"""Fail-closed proof that model traffic remains successful and resource-bounded."""

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
    """Do not accept process exit zero or unbounded broker work as model success."""

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

    def test_local_credential_broker_has_a_hard_request_worker_bound(self) -> None:
        """Untrusted agent traffic must not create an unbounded runner thread pool."""

        broker = _step_block(
            self.workflow, "Start loopback-only NVIDIA NIM credential broker"
        )
        for contract in (
            "MAX_BROKER_REQUESTS = 4",
            "class BoundedThreadingHTTPServer(ThreadingHTTPServer):",
            "threading.BoundedSemaphore(MAX_BROKER_REQUESTS)",
            "if not self.request_slots.acquire(blocking=False):",
            "request.close()",
            "self.request_slots.release()",
            "server = BoundedThreadingHTTPServer((\"127.0.0.1\", 8765), Handler)",
        ):
            self.assertIn(contract, broker)

        server_class = broker[
            broker.index("class BoundedThreadingHTTPServer") : broker.index(
                "class Handler"
            )
        ]
        self.assertLess(
            server_class.index("self.request_slots.acquire(blocking=False)"),
            server_class.index("super().process_request(request, client_address)"),
        )
        self.assertLess(
            server_class.index("super().process_request_thread(request, client_address)"),
            server_class.index("self.request_slots.release()"),
        )


if __name__ == "__main__":
    unittest.main()
