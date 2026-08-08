"""Regression contracts for the fail-closed hourly product-development workflow."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"
EXPECTED_ENDPOINTS = {
    "*.actions.githubusercontent.com:443",
    "*.blob.core.windows.net:443",
    "api.github.com:443",
    "codeload.github.com:443",
    "crates.io:443",
    "github.com:443",
    "index.crates.io:443",
    "integrate.api.nvidia.com:443",
    "objects.githubusercontent.com:443",
    "registry.npmjs.org:443",
    "release-assets.githubusercontent.com:443",
    "results-receiver.actions.githubusercontent.com:443",
    "static.crates.io:443",
    "static.rust-lang.org:443",
}


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

    def test_github_api_gate_keeps_exact_fail_closed_endpoint_contract(self) -> None:
        """GitHub API access must stay explicit without unreviewed egress expansion."""

        hardening = _step_block(
            self.workflow, "Harden runner and block undeclared egress"
        )
        self.assertIn("egress-policy: block", hardening)
        self.assertEqual(_allowed_endpoints(hardening), EXPECTED_ENDPOINTS)

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
        """Open PR, release blocker, and dry run decisions must stop before NIM."""

        gate = _step_block(
            self.workflow, "Enforce NVIDIA NIM and pull-request-first gates"
        )
        credential_gate = gate.index("reason=nim_api_key_unavailable")
        for reason in ("open_pull_request", "release_blocker", "dry_run"):
            with self.subTest(reason=reason):
                stop = f"develop=false\n            reason={reason}"
                self.assertIn(stop, gate)
                self.assertLess(gate.index(stop), credential_gate)


if __name__ == "__main__":
    unittest.main()
