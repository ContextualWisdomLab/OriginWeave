"""Contract for proving loopback fixture-server shutdown actually completes."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class FixtureServerShutdownPostconditionContractTests(unittest.TestCase):
    """Reject cleanup evidence that relies on bounded join acknowledgement alone."""

    def test_fixture_server_shutdown_fails_closed_if_thread_is_still_alive(self) -> None:
        """A timed join must be followed by an explicit thread-termination observation."""

        namespace = runpy.run_path(str(RUNNER), run_name="fixture_shutdown_postcondition_contract")
        stop_fixture_server = namespace["_stop_fixture_server"]

        class FakeServer:
            def __init__(self) -> None:
                self.shutdown_called = False
                self.close_called = False

            def shutdown(self) -> None:
                self.shutdown_called = True

            def server_close(self) -> None:
                self.close_called = True

        class StalledThread:
            def __init__(self) -> None:
                self.join_timeout: float | None = None

            def join(self, timeout: float | None = None) -> None:
                self.join_timeout = timeout

            def is_alive(self) -> bool:
                return True

        server = FakeServer()
        thread = StalledThread()
        with self.assertRaisesRegex(
            RuntimeError,
            r"^fixture server thread did not stop$",
        ):
            stop_fixture_server(server, thread)

        self.assertTrue(server.shutdown_called)
        self.assertTrue(server.close_called)
        self.assertEqual(thread.join_timeout, 5)

    def test_fixture_server_shutdown_accepts_observed_thread_termination(self) -> None:
        """Successful cleanup requires the helper thread to be observed stopped."""

        namespace = runpy.run_path(str(RUNNER), run_name="fixture_shutdown_success_contract")
        stop_fixture_server = namespace["_stop_fixture_server"]

        class FakeServer:
            def shutdown(self) -> None:
                return None

            def server_close(self) -> None:
                return None

        class StoppedThread:
            def join(self, timeout: float | None = None) -> None:
                self.join_timeout = timeout

            def is_alive(self) -> bool:
                return False

        thread = StoppedThread()
        stop_fixture_server(FakeServer(), thread)
        self.assertEqual(thread.join_timeout, 5)


if __name__ == "__main__":
    unittest.main()
