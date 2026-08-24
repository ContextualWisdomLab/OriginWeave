"""Contract for proving the controlled Agent Task Chromium process set terminates."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskProcessSetTerminationContractTests(unittest.TestCase):
    """Keep descendant cleanup evidence bounded, PID-reuse-safe, and fail closed."""

    def test_runner_exposes_bounded_process_set_identity_and_exit_helpers(self) -> None:
        """A sampled Chromium tree needs exact PID/start-time identities before shutdown."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_set_termination")
        for expected in (
            "_read_linux_process_identity_set",
            "_wait_for_linux_process_identity_set_exit",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)

    def test_process_identity_set_reader_preserves_root_and_tolerates_exited_children(self) -> None:
        """Short-lived descendants may exit after the snapshot, but root identity stays exact."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_set_reader")
        reader = namespace["_read_linux_process_identity_set"]
        original_reader = reader.__globals__["_read_linux_proc_stat_process_identity"]
        identities = {10: (10, 101), 20: (20, 202), 30: (30, 303)}
        try:
            reader.__globals__["_read_linux_proc_stat_process_identity"] = identities.get
            self.assertEqual(
                reader((10, 20, 30), required_root_identity=(10, 101)),
                (((10, 101), (20, 202), (30, 303)), 0),
            )

            reader.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: None if process_id == 20 else identities[process_id]
            )
            self.assertEqual(
                reader((10, 20, 30), required_root_identity=(10, 101)),
                (((10, 101), (30, 303)), 1),
            )

            reader.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: None if process_id == 10 else identities[process_id]
            )
            with self.assertRaisesRegex(RuntimeError, "root process identity disappeared"):
                reader((10, 20, 30), required_root_identity=(10, 101))

            reader.__globals__["_read_linux_proc_stat_process_identity"] = identities.get
            with self.assertRaisesRegex(RuntimeError, "root process identity changed"):
                reader((10, 20, 30), required_root_identity=(10, 999))
        finally:
            reader.__globals__["_read_linux_proc_stat_process_identity"] = original_reader

        for process_ids, root_identity in (
            ((), (10, 101)),
            ((10, 10), (10, 101)),
            ((10, 20), (20, 202)),
            ((10, 20), (10, 0)),
        ):
            with self.subTest(process_ids=process_ids, root_identity=root_identity):
                with self.assertRaises(ValueError):
                    reader(process_ids, required_root_identity=root_identity)

    def test_process_set_exit_waiter_uses_one_deadline_and_detects_any_live_identity(self) -> None:
        """A reused PID is exited evidence, but any exact surviving identity fails closed."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_set_exit_waiter")
        waiter = namespace["_wait_for_linux_process_identity_set_exit"]
        original_reader = waiter.__globals__["_read_linux_proc_stat_process_identity"]
        identities = ((10, 101), (20, 202), (30, 303))
        try:
            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda _process_id: None
            )
            self.assertTrue(waiter(identities, timeout_seconds=0.0))

            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: (process_id, {10: 111, 20: 222, 30: 333}[process_id])
            )
            self.assertTrue(waiter(identities, timeout_seconds=0.0))

            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: (20, 202) if process_id == 20 else None
            )
            self.assertFalse(waiter(identities, timeout_seconds=0.0))
        finally:
            waiter.__globals__["_read_linux_proc_stat_process_identity"] = original_reader

        for process_identities, timeout_seconds in (
            ((), 0.0),
            (((10, 101), (10, 102)), 0.0),
            (((10, 101),), -0.1),
        ):
            with self.subTest(
                process_identities=process_identities,
                timeout_seconds=timeout_seconds,
            ):
                with self.assertRaises(ValueError):
                    waiter(process_identities, timeout_seconds=timeout_seconds)

    def test_successful_agent_task_requires_entire_sampled_process_set_to_terminate(self) -> None:
        """Successful acceptance must preserve already-exited descendants as explicit evidence."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "chromium_process_identities",
            "chromium_process_pre_shutdown_exit_count",
            '"chromium_process_set_terminated"',
            'result["chromium_process_set_terminated"]',
            'trial.get("chromium_process_set_terminated") is True',
            "Agent Task Chromium process set did not terminate",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
