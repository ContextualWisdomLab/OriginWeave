"""Contracts for provider-aware retry feasibility in the hourly product loop."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"


def _step_block(workflow: str, step_name: str) -> str:
    """Return one named workflow step without requiring a YAML parser."""

    marker = f"      - name: {step_name}\n"
    _before, separator, remainder = workflow.partition(marker)
    if not separator:
        raise AssertionError(f"missing workflow step: {step_name}")
    return remainder.partition("\n      - name: ")[0]


class ProviderRetryFeasibilityContractTests(unittest.TestCase):
    """Do not confuse a live local broker with a usable upstream provider."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_broker_exposes_generation_safe_nonretryable_provider_outcomes(self) -> None:
        """Runner-visible telemetry must distinguish auth rejection and rate limiting."""

        broker = _step_block(
            self.workflow, "Start loopback-only NVIDIA NIM credential broker"
        )
        for contract in (
            'self.path == "/statusz"',
            '"request_counter"',
            '"last_auth_rejected_request"',
            '"last_rate_limited_request"',
            "status in (401, 403)",
            "status == 429",
        ):
            with self.subTest(contract=contract):
                self.assertIn(contract, broker)

    def test_retry_stops_when_current_attempt_proves_provider_unavailable(self) -> None:
        """Cross-model fallback must not repeat auth or rate-limit failures on one provider."""

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        loop = agent[agent.index("for model in $OPENCODE_MODEL_CANDIDATES; do") :]
        for contract in (
            "provider_before=",
            '"/statusz"',
            "cause=provider_auth_rejected",
            "cause=provider_rate_limited",
            "feasible_retry=false",
        ):
            with self.subTest(contract=contract):
                self.assertIn(contract, loop)

        self.assertLess(loop.index("provider_before="), loop.index("opencode run"))
        self.assertLess(
            loop.index("cause=provider_auth_rejected"),
            loop.index("cause=model_or_tool_failure"),
        )
        self.assertLess(
            loop.index("cause=provider_rate_limited"),
            loop.index("cause=model_or_tool_failure"),
        )

    def test_provider_status_is_fetched_before_local_json_parsing(self) -> None:
        """Status JSON must be data to local code, never a curl-to-interpreter pipe."""

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        loop = agent[agent.index("for model in $OPENCODE_MODEL_CANDIDATES; do") :]
        self.assertIn("provider_before_json=", loop)
        self.assertIn("json.loads(sys.argv[1])", loop)
        self.assertNotIn(
            'curl -fsS "http://${NIM_PROXY_HOST}:${NIM_PROXY_PORT}${provider_status_path}" |',
            loop,
        )


if __name__ == "__main__":
    unittest.main()
