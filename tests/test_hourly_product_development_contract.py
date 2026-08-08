"""Regression contracts for the fail-closed hourly product-development workflow."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"


def _step_block(workflow: str, step_name: str) -> str:
    """Return one named workflow step without depending on a YAML parser."""

    marker = f"      - name: {step_name}\n"
    _before, separator, remainder = workflow.partition(marker)
    if not separator:
        raise AssertionError(f"missing workflow step: {step_name}")
    return remainder.partition("\n      - name: ")[0]


def _allowed_endpoints(hardening_step: str) -> set[str]:
    """Return exact endpoint entries from the Harden Runner allowlist."""

    marker = "          allowed-endpoints: >-\n"
    _before, separator, remainder = hardening_step.partition(marker)
    if not separator:
        raise AssertionError("missing Harden Runner allowed-endpoints block")
    return {
        line.strip()
        for line in remainder.splitlines()
        if line.startswith("            ") and line.strip()
    }


class HourlyProductDevelopmentContractTests(unittest.TestCase):
    """Keep deterministic governance independent from optional model credentials."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW.read_text(encoding="utf-8")

    def test_gitHub_api_gate_keeps_exact_fail_closed_endpoint_contract(self) -> None:
        """GitHub API access must stay explicit without broadening fail-closed egress."""

        hardening = _step_block(
            self.workflow, "Harden runner and block undeclared egress"
        )
        endpoints = _allowed_endpoints(hardening)
        self.assertIn("egress-policy: block", hardening)
        self.assertIn("api.github.com:443", endpoints)
        self.assertNotIn("*.github.com:443", endpoints)

        gate = _step_block(
            self.workflow, "Enforce NVIDIA NIM and pull-request-first gates"
        )
        self.assertIn(
            'gh api "repos/${GITHUB_REPOSITORY}/pulls?state=open&per_page=1"', gate
        )
        self.assertIn(
            'gh api "repos/${GITHUB_REPOSITORY}/issues?state=open&labels=release-blocker&per_page=1"',
            gate,
        )

    def test_deterministic_stop_gates_precede_optional_nvidia_credential(self) -> None:
        """Open PR, release blocker, and dry run decisions must not require NIM."""

        gate = _step_block(
            self.workflow, "Enforce NVIDIA NIM and pull-request-first gates"
        )
        credential_gate = gate.index("reason=nim_api_key_unavailable")
        for reason in ("open_pull_request", "release_blocker", "dry_run"):
            with self.subTest(reason=reason):
                self.assertLess(gate.index(f"reason={reason}"), credential_gate)


if __name__ == "__main__":
    unittest.main()
