"""Contract for PID-safe browser-process crash evidence in the Agent Task lane."""

from __future__ import annotations

import pathlib
import runpy
import signal
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskBrowserCrashRecoveryContractTests(unittest.TestCase):
    """Require controlled browser-process interruption without PID-reuse races."""

    def test_runner_exposes_pidfd_crash_boundary(self) -> None:
        """The Linux browser crash probe must use a PID-safe signalling boundary."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_contract")
        for expected in (
            "_signal_linux_process_identity",
            "_run_agent_task_browser_crash_browser_pass",
            "_run_agent_task_browser_crash_trial",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)

    def test_signal_boundary_rejects_pid_reuse_before_open(self) -> None:
        """A reused PID must never receive the crash signal."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_reuse")
        signal_identity = namespace["_signal_linux_process_identity"]
        opened: list[int] = []
        signalled: list[tuple[int, int]] = []

        signal_identity.__globals__["_read_linux_proc_stat_process_identity"] = (
            lambda _process_id: (777, 99)
        )
        signal_identity.__globals__["os"].pidfd_open = lambda process_id, _flags=0: opened.append(process_id) or 12
        signal_identity.__globals__["signal"].pidfd_send_signal = (
            lambda pidfd, sig, *_args, **_kwargs: signalled.append((pidfd, sig))
        )
        signal_identity.__globals__["os"].close = lambda _fd: None

        self.assertFalse(signal_identity((777, 42), signal.SIGKILL))
        self.assertEqual(opened, [])
        self.assertEqual(signalled, [])

    def test_signal_boundary_rechecks_identity_after_pidfd_open(self) -> None:
        """The process identity must still match after the race-free handle is opened."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_post_open")
        signal_identity = namespace["_signal_linux_process_identity"]
        identities = iter(((777, 42), (777, 99)))
        signalled: list[tuple[int, int]] = []
        closed: list[int] = []

        signal_identity.__globals__["_read_linux_proc_stat_process_identity"] = (
            lambda _process_id: next(identities)
        )
        signal_identity.__globals__["os"].pidfd_open = lambda _process_id, _flags=0: 12
        signal_identity.__globals__["signal"].pidfd_send_signal = (
            lambda pidfd, sig, *_args, **_kwargs: signalled.append((pidfd, sig))
        )
        signal_identity.__globals__["os"].close = closed.append

        self.assertFalse(signal_identity((777, 42), signal.SIGKILL))
        self.assertEqual(signalled, [])
        self.assertEqual(closed, [12])

    def test_signal_boundary_targets_only_exact_open_identity(self) -> None:
        """Exact identity proof must send one signal through the opened pidfd and close it."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_exact")
        signal_identity = namespace["_signal_linux_process_identity"]
        signalled: list[tuple[int, int]] = []
        closed: list[int] = []

        signal_identity.__globals__["_read_linux_proc_stat_process_identity"] = (
            lambda _process_id: (777, 42)
        )
        signal_identity.__globals__["os"].pidfd_open = lambda _process_id, _flags=0: 12
        signal_identity.__globals__["signal"].pidfd_send_signal = (
            lambda pidfd, sig, *_args, **_kwargs: signalled.append((pidfd, sig))
        )
        signal_identity.__globals__["os"].close = closed.append

        self.assertTrue(signal_identity((777, 42), signal.SIGKILL))
        self.assertEqual(signalled, [(12, signal.SIGKILL)])
        self.assertEqual(closed, [12])

    def test_signal_boundary_handles_exit_before_pidfd_signal(self) -> None:
        """A target that exits after pidfd open must become a bounded not-signalled result."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_esrch")
        signal_identity = namespace["_signal_linux_process_identity"]
        closed: list[int] = []

        signal_identity.__globals__["_read_linux_proc_stat_process_identity"] = (
            lambda _process_id: (777, 42)
        )
        signal_identity.__globals__["os"].pidfd_open = lambda _process_id, _flags=0: 12

        def exited_before_signal(*_args: object, **_kwargs: object) -> None:
            raise ProcessLookupError("process exited before pidfd signal")

        signal_identity.__globals__["signal"].pidfd_send_signal = exited_before_signal
        signal_identity.__globals__["os"].close = closed.append

        self.assertFalse(signal_identity((777, 42), signal.SIGKILL))
        self.assertEqual(closed, [12])

    def test_proc_stat_identity_treats_read_time_esrch_as_process_exit(self) -> None:
        """A process disappearing while procfs is read must become bounded absence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_proc_esrch")
        read_identity = namespace["_read_linux_proc_stat_process_identity"]

        class VanishedStat:
            def __enter__(self) -> "VanishedStat":
                return self

            def __exit__(self, *_args: object) -> None:
                return None

            def read(self, _limit: int) -> str:
                raise ProcessLookupError("process exited during proc stat read")

        with mock.patch.object(pathlib.Path, "open", return_value=VanishedStat()):
            self.assertIsNone(read_identity(777))

    def test_crash_driver_cleanup_is_idempotent_after_driver_exit(self) -> None:
        """A browser crash may end ChromeDriver before cleanup without a second signal."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_cleanup")
        cleanup = namespace["_stop_crashed_driver"]
        events: list[object] = []

        class ExitedDriver:
            def poll(self) -> int:
                events.append("poll")
                return 0

            def terminate(self) -> None:
                raise AssertionError("already-exited ChromeDriver must not be re-signalled")

            def wait(self, *, timeout: int) -> int:
                events.append(("wait", timeout))
                return 0

            def kill(self) -> None:
                raise AssertionError("already-exited ChromeDriver must not be killed")

        cleanup(ExitedDriver())
        self.assertEqual(events, ["poll", ("wait", 5)])

    def test_crash_session_cleanup_ignores_only_reviewed_transport_failures(self) -> None:
        """Expected post-crash transport loss is bounded, while programming failures propagate."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_session_cleanup")
        self.assertIn("_cleanup_crashed_browser_session", namespace)
        cleanup_session = namespace["_cleanup_crashed_browser_session"]
        calls: list[tuple[object, ...]] = []

        def expected_transport_failure(*args: object, **_kwargs: object) -> object:
            calls.append(args)
            raise OSError("browser transport is already gone")

        cleanup_session.__globals__["_json_request"] = expected_transport_failure
        cleanup_session(9222, "session-1")
        self.assertEqual(len(calls), 1)

        def unexpected_programming_failure(*_args: object, **_kwargs: object) -> object:
            raise AssertionError("unexpected cleanup defect")

        cleanup_session.__globals__["_json_request"] = unexpected_programming_failure
        with self.assertRaisesRegex(AssertionError, "unexpected cleanup defect"):
            cleanup_session(9222, "session-1")

    def test_crash_session_cleanup_without_session_is_a_noop(self) -> None:
        """No session identifier means cleanup has no remote operation to attempt."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_browser_crash_no_session")
        self.assertIn("_cleanup_crashed_browser_session", namespace)
        cleanup_session = namespace["_cleanup_crashed_browser_session"]

        def unexpected_request(*_args: object, **_kwargs: object) -> object:
            raise AssertionError("cleanup must not call WebDriver without a session")

        cleanup_session.__globals__["_json_request"] = unexpected_request
        cleanup_session(9222, None)

    def test_crash_lane_is_required_for_success_evidence(self) -> None:
        """The real-browser evidence must retain deterministic crash and teardown proof."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            '"browser_crash"',
            '"browser_process_crash_detected"',
            '"browser_process_terminated"',
            '"chromium_process_set_terminated"',
            "Agent Task browser-crash recovery gate failed",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
