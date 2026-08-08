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


def _scalar_value(workflow: str, name: str, indentation: int) -> str:
    """Return one quoted or unquoted YAML scalar from the expected indentation."""

    marker = f"{' ' * indentation}{name}: "
    for line in workflow.splitlines():
        if line.startswith(marker):
            return line.removeprefix(marker).strip().strip('"')
    raise AssertionError(f"missing scalar: {name}")


def _folded_environment_values(workflow: str, name: str) -> list[str]:
    """Return nonempty values from one top-level folded environment scalar."""

    marker = f"  {name}: >-\n"
    _before, separator, remainder = workflow.partition(marker)
    if not separator:
        raise AssertionError(f"missing folded environment value: {name}")
    values = []
    for line in remainder.splitlines():
        if not line.startswith("    "):
            break
        if line.strip():
            values.append(line.strip())
    return values


class HourlyProductDevelopmentContractTests(unittest.TestCase):
    """Keep deterministic governance and recovery bounded, isolated, and realistic."""

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
            self.workflow, "Evaluate deterministic pull-request-first gates"
        )
        self.assertIn(
            'gh api "repos/${GITHUB_REPOSITORY}/pulls?state=open&per_page=1"', gate
        )
        self.assertIn(
            'gh api "repos/${GITHUB_REPOSITORY}/issues?state=open&labels=release-blocker&per_page=100"',
            gate,
        )

    def test_release_blocker_checks_filter_pulls_after_a_full_page(self) -> None:
        """A labeled PR must not hide a real release-blocker issue behind pagination."""

        query = (
            'gh api "repos/${GITHUB_REPOSITORY}/issues?state=open&labels='
            'release-blocker&per_page=100"'
        )
        for step_name in (
            "Evaluate deterministic pull-request-first gates",
            "Recheck repository state and publish the verified PR",
        ):
            with self.subTest(step_name=step_name):
                step = _step_block(self.workflow, step_name)
                self.assertIn(query, step)
                self.assertIn("select(.pull_request == null)", step)
                self.assertNotIn("labels=release-blocker&per_page=1", step)

    def test_nvidia_secret_is_materialized_only_after_deterministic_gates(self) -> None:
        """Stopped runs must never receive the optional live-model credential."""

        gate = _step_block(
            self.workflow, "Evaluate deterministic pull-request-first gates"
        )
        self.assertNotIn("NIM_UPSTREAM_API_KEY", gate)
        self.assertNotIn("secrets.NVIDIA_NIM_API_KEY", gate)
        for reason in ("open_pull_request", "release_blocker", "dry_run"):
            with self.subTest(reason=reason):
                self.assertIn(f"reason={reason}", gate)

        credential = _step_block(
            self.workflow, "Require NVIDIA NIM credential for model-backed path"
        )
        self.assertIn("if: steps.gate.outputs.develop == 'true'", credential)
        self.assertIn(
            "NIM_UPSTREAM_API_KEY: ${{ secrets.NVIDIA_NIM_API_KEY }}", credential
        )
        self.assertIn("reason=nim_api_key_unavailable", credential)
        self.assertIn("ready=true", credential)

        checkout = _step_block(
            self.workflow,
            "Check out the protected default branch without persisted credentials",
        )
        self.assertIn("if: steps.credential.outputs.ready == 'true'", checkout)

    def test_missing_model_credential_is_not_a_green_noop(self) -> None:
        """A model-backed run must fail closed when its required NIM key is absent."""

        credential = _step_block(
            self.workflow, "Require NVIDIA NIM credential for model-backed path"
        )
        marker = 'if [ -z "${NIM_UPSTREAM_API_KEY:-}" ]; then'
        else_marker = "          else\n"
        self.assertIn(marker, credential)
        self.assertIn(else_marker, credential)
        missing_key_branch = credential[
            credential.index(marker) : credential.index(else_marker, credential.index(marker))
        ]
        self.assertIn("reason=nim_api_key_unavailable", missing_key_branch)
        self.assertIn("exit 1", missing_key_branch)
        self.assertNotIn("exit 0", missing_key_branch)

    def test_bundle_leak_scan_uses_a_fingerprint_without_rematerializing_secret(
        self,
    ) -> None:
        """Post-model validation must detect the key without receiving the raw key."""

        credential = _step_block(
            self.workflow, "Require NVIDIA NIM credential for model-backed path"
        )
        for contract in (
            "originweave-nim-secret-fingerprint.json",
            '"secret_length"',
            '"secret_sha256"',
            '"rolling_hash"',
            'echo "fingerprint_file=$fingerprint_file"',
        ):
            self.assertIn(contract, credential)

        bundle = _step_block(
            self.workflow, "Validate and seal the credential-free change bundle"
        )
        self.assertNotIn("FORBIDDEN_SECRET:", bundle)
        self.assertNotIn("secrets.NVIDIA_NIM_API_KEY", bundle)
        self.assertIn("FORBIDDEN_SECRET_FINGERPRINT_FILE:", bundle)
        self.assertIn("steps.credential.outputs.fingerprint_file", bundle)
        self.assertIn("contains_forbidden_secret", bundle)
        self.assertIn("fingerprint_path.unlink", bundle)

        permitted_steps = (
            "Require NVIDIA NIM credential for model-backed path",
            "Start loopback-only NVIDIA NIM credential broker",
        )
        permitted_uses = sum(
            _step_block(self.workflow, name).count("secrets.NVIDIA_NIM_API_KEY")
            for name in permitted_steps
        )
        self.assertEqual(permitted_uses, 2)
        self.assertEqual(
            self.workflow.count("secrets.NVIDIA_NIM_API_KEY"), permitted_uses
        )

    def test_pr_message_is_bounded_before_secret_scan(self) -> None:
        """Untrusted PR prose must be size-bounded before byte-wise leak scanning."""

        bundle = _step_block(
            self.workflow, "Validate and seal the credential-free change bundle"
        )
        read = 'message_bytes = message_path.read_bytes()'
        bound = 'if len(message_bytes) > 65_536:'
        scan = 'if contains_forbidden_secret(message_bytes):'
        for contract in (read, bound, 'raise SystemExit("PR_MESSAGE.md is too large")', scan):
            self.assertIn(contract, bundle)
        self.assertLess(bundle.index(read), bundle.index(bound))
        self.assertLess(bundle.index(bound), bundle.index(scan))

    def test_declared_runtime_can_execute_every_model_and_verification_reserve(self) -> None:
        """The job timeout must cover every advertised model plus final verification."""

        candidates = _folded_environment_values(
            self.workflow, "OPENCODE_MODEL_CANDIDATES"
        )
        per_model_seconds = int(
            _scalar_value(self.workflow, "OPENCODE_RUN_TIMEOUT_SECONDS", 2)
        )
        job_timeout_minutes = int(
            _scalar_value(self.workflow, "timeout-minutes", 4)
        )
        model_minutes = (len(candidates) * per_model_seconds + 59) // 60
        self.assertGreaterEqual(job_timeout_minutes, model_minutes + 30)
        self.assertIn("cancel-in-progress: false", self.workflow)

    def test_agent_prompt_requires_rca_feasibility_action_and_revalidation(self) -> None:
        """A failed command must lead to evidence-based feasible remediation, not surrender."""

        prepare = _step_block(
            self.workflow, "Prepare immutable baseline and disposable workspace"
        )
        required_sequence = (
            "Perform root-cause analysis",
            "Verify that the corrective action is feasible",
            "materially distinct",
            "Rerun the exact failed command",
        )
        positions = []
        for phrase in required_sequence:
            self.assertIn(phrase, prepare)
            positions.append(prepare.index(phrase))
        self.assertEqual(positions, sorted(positions))

    def test_each_model_retry_starts_from_the_exact_pristine_source_tree(self) -> None:
        """Fallback models must not inherit partial source edits from failed attempts."""

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        loop = agent[agent.index("for model in $OPENCODE_MODEL_CANDIDATES; do") :]
        reset = "reset_agent_workspace"
        invocation = "opencode run"
        self.assertIn('rm -rf "$AGENT_WORKSPACE"', agent)
        self.assertIn('git archive HEAD | tar -x -C "$AGENT_WORKSPACE"', agent)
        self.assertIn(reset, loop)
        self.assertLess(loop.index(reset), loop.index(invocation))

    def test_success_cleanup_handles_unprivileged_workspace_ownership(self) -> None:
        """A successful model must not fail while removing its uid-65532 config."""

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        self.assertIn('sudo rm -f "${AGENT_WORKSPACE}/opencode.json"', agent)

    def test_retry_feasibility_distinguishes_model_failure_from_broker_failure(self) -> None:
        """The scheduler may retry a model only while its local credential broker is healthy."""

        agent = _step_block(
            self.workflow, "Run OpenCode in an unprivileged no-Git workspace"
        )
        for contract in (
            "cause=model_timeout",
            "cause=model_or_tool_failure",
            "cause=credential_broker_unavailable",
            "feasible_retry=true",
            "feasible_retry=false",
        ):
            self.assertIn(contract, agent)
        self.assertIn(
            'tail -n 50 "${RUNNER_TEMP}/originweave-nim-broker.log" >&2 || true',
            agent,
        )

    def test_missing_publication_authority_is_not_a_green_noop(self) -> None:
        """A verified change must fail closed when its dedicated publisher is absent."""

        publish = _step_block(
            self.workflow, "Recheck repository state and publish the verified PR"
        )
        marker = 'if [ -z "${AUTOMATION_TOKEN:-}" ]; then'
        next_probe = 'live_default_sha="$(GH_TOKEN="$AUTOMATION_TOKEN" gh api'
        self.assertIn(marker, publish)
        self.assertIn(next_probe, publish)
        missing_token_branch = publish[
            publish.index(marker) : publish.index(next_probe)
        ]
        self.assertIn("Dedicated OpenCode PR token unavailable", missing_token_branch)
        self.assertIn("exit 1", missing_token_branch)
        self.assertNotIn("exit 0", missing_token_branch)


if __name__ == "__main__":
    unittest.main()
