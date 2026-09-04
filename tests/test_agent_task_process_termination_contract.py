"""Contract for proving the controlled Agent Task browser process terminates."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


def _proc_stat(process_id: int, command: str, start_time_ticks: int) -> str:
    """Build the bounded `/proc/<pid>/stat` prefix through field 22."""

    fields_three_through_twenty_one = ["S", *[str(value) for value in range(4, 22)]]
    return (
        f"{process_id} ({command}) "
        + " ".join(fields_three_through_twenty_one)
        + f" {start_time_ticks}\n"
    )


class AgentTaskProcessTerminationContractTests(unittest.TestCase):
    """Keep process-cleanup evidence PID-reuse-safe and fail closed."""

    def test_runner_exposes_bounded_process_identity_and_exit_helpers(self) -> None:
        """The runner needs a Linux process identity boundary, not PID-only polling."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_termination")
        for expected in (
            "MAX_PROC_STAT_CHARACTERS",
            "PROCESS_EXIT_TIMEOUT_SECONDS",
            "_parse_linux_proc_stat_process_identity",
            "_read_linux_proc_stat_process_identity",
            "_wait_for_linux_process_identity_exit",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)

    def test_proc_stat_identity_parser_handles_command_text_and_rejects_ambiguity(self) -> None:
        """PID reuse proof must bind a positive PID to the exact Linux start-time field."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_proc_stat_parser")
        parser = namespace["_parse_linux_proc_stat_process_identity"]

        self.assertEqual(parser(_proc_stat(321, "chrome worker", 987654)), (321, 987654))
        self.assertEqual(parser(_proc_stat(322, "chrome ) helper", 987655)), (322, 987655))

        for malformed in (
            "",
            "321 chrome S 1 2 3\n",
            _proc_stat(0, "chrome", 10),
            _proc_stat(321, "chrome", 0),
            _proc_stat(321, "chrome", -1),
            "321 (chrome) S 1 2 3\n",
            "not-a-pid (chrome) S " + " ".join(["1"] * 20) + "\n",
            "321 (chrome) S " + " ".join(["1"] * 19) + " not-a-time\n",
        ):
            with self.subTest(malformed=malformed):
                with self.assertRaises(ValueError):
                    parser(malformed)

    def test_process_exit_waiter_distinguishes_exit_pid_reuse_and_live_identity(self) -> None:
        """A reused PID is not the original browser process and a live identity must fail closed."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_exit_waiter")
        waiter = namespace["_wait_for_linux_process_identity_exit"]
        original_reader = waiter.__globals__["_read_linux_proc_stat_process_identity"]
        try:
            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda _process_id: None
            )
            self.assertTrue(waiter(321, 987654, timeout_seconds=0.0))

            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: (process_id, 987655)
            )
            self.assertTrue(waiter(321, 987654, timeout_seconds=0.0))

            waiter.__globals__["_read_linux_proc_stat_process_identity"] = (
                lambda process_id: (process_id, 987654)
            )
            self.assertFalse(waiter(321, 987654, timeout_seconds=0.0))
        finally:
            waiter.__globals__["_read_linux_proc_stat_process_identity"] = original_reader

        for process_id, start_time_ticks, timeout_seconds in (
            (0, 987654, 0.0),
            (321, 0, 0.0),
            (321, 987654, -0.1),
        ):
            with self.subTest(
                process_id=process_id,
                start_time_ticks=start_time_ticks,
                timeout_seconds=timeout_seconds,
            ):
                with self.assertRaises(ValueError):
                    waiter(process_id, start_time_ticks, timeout_seconds=timeout_seconds)

    def test_successful_agent_task_requires_post_shutdown_process_termination_evidence(self) -> None:
        """A successful task must not be accepted while its original browser root is still live."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "browser_process_start_time_ticks",
            '"browser_process_terminated"',
            'result["browser_process_terminated"]',
            'trial.get("browser_process_terminated") is True',
            "Agent Task browser process did not terminate",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
