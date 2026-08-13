"""Contract for one total post-shutdown teardown deadline in the forced-close lane."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskForcedCloseSharedTeardownDeadlineContractTests(unittest.TestCase):
    """Prevent root and process-set teardown polling from multiplying the budget."""

    def test_runner_exposes_one_combined_teardown_waiter(self) -> None:
        """Root and sampled-set evidence must be observed under one timeout authority."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_shared_teardown_deadline"
        )
        self.assertIn("_wait_for_linux_process_teardown", namespace)

    def test_combined_waiter_preserves_partial_evidence_at_one_deadline(self) -> None:
        """A root may exit while a descendant remains live when the one deadline expires."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_shared_teardown_behavior"
        )
        waiter = namespace["_wait_for_linux_process_teardown"]

        class FakeTime:
            def __init__(self) -> None:
                self.now = 0.0

            def monotonic(self) -> float:
                return self.now

            def sleep(self, seconds: float) -> None:
                self.now += seconds

        fake_time = FakeTime()
        root_identity = (101, 1_001)
        child_identity = (202, 2_002)

        def fake_read(process_id: int) -> tuple[int, int] | None:
            if process_id == root_identity[0]:
                return None if fake_time.now >= 0.05 else root_identity
            if process_id == child_identity[0]:
                return child_identity
            raise AssertionError(f"unexpected process id: {process_id}")

        waiter.__globals__["time"] = fake_time
        waiter.__globals__["_read_linux_proc_stat_process_identity"] = fake_read

        root_terminated, process_set_terminated = waiter(
            root_identity[0],
            root_identity[1],
            (root_identity, child_identity),
            timeout_seconds=0.10,
        )
        self.assertIs(root_terminated, True)
        self.assertIs(process_set_terminated, False)
        self.assertLessEqual(fake_time.now, 0.1000001)

    def test_combined_waiter_requires_root_identity_in_the_sampled_set(self) -> None:
        """A separate root identity may not be paired with an unrelated process set."""

        namespace = runpy.run_path(
            str(RUNNER), run_name="forced_close_shared_teardown_identity"
        )
        waiter = namespace["_wait_for_linux_process_teardown"]
        with self.assertRaises(ValueError):
            waiter(101, 1_001, ((202, 2_002),), timeout_seconds=0)

    def test_forced_close_browser_pass_uses_only_the_combined_waiter(self) -> None:
        """The forced-close pass must not run independent root and set timeout windows."""

        runner = RUNNER.read_text(encoding="utf-8")
        start = runner.index("def _run_agent_task_forced_close_browser_pass(")
        end = runner.index("\ndef _run_agent_task_forced_close_trial(", start)
        browser_pass = runner[start:end]

        self.assertIn("_wait_for_linux_process_teardown(", browser_pass)
        self.assertNotIn("_wait_for_linux_process_identity_exit(", browser_pass)
        self.assertNotIn("_wait_for_linux_process_identity_set_exit(", browser_pass)


if __name__ == "__main__":
    unittest.main()
