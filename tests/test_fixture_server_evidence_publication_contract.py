"""Contract for publishing browser evidence only after owned fixture teardown."""

from __future__ import annotations

import os
import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class FixtureServerEvidencePublicationContractTests(unittest.TestCase):
    """Keep success-shaped evidence behind both fixture-server cleanup post-conditions."""

    def _run_main_with_controlled_servers(
        self,
        *,
        fail_agent_cleanup: bool = False,
        fail_trials: bool = False,
    ) -> tuple[list[str], Exception | None]:
        namespace = runpy.run_path(
            str(RUNNER),
            run_name="fixture_evidence_publication_contract",
        )
        events: list[str] = []

        class FakeServer:
            def __init__(self, label: str, port: int) -> None:
                self.label = label
                self.server_port = port

        class FakeThread:
            pass

        def start_fixture_server(directory: pathlib.Path) -> tuple[FakeServer, FakeThread]:
            if directory == namespace["FIXTURE"]:
                return FakeServer("mv3", 19001), FakeThread()
            if directory == namespace["AGENT_TASK_FIXTURE"]:
                return FakeServer("agent", 19002), FakeThread()
            raise AssertionError("unexpected fixture directory")

        def stop_fixture_server(server: FakeServer, _thread: FakeThread) -> None:
            events.append(f"stopped:{server.label}")
            if fail_agent_cleanup and server.label == "agent":
                raise RuntimeError("fixture server thread did not stop")

        def run_restart_trial(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            trial_number: int,
        ) -> dict[str, object]:
            return {
                "trial_number": trial_number,
                "passed": not fail_trials,
                "browser_version": namespace["PINNED_CHROME_VERSION"],
                "surfaces": {"controlled": not fail_trials},
                "browser_passes": [],
            }

        def run_agent_task_trial(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            trial_number: int,
        ) -> dict[str, object]:
            return {
                "trial_number": trial_number,
                "passed": not fail_trials,
            }

        namespace["_start_fixture_server"] = start_fixture_server
        namespace["_stop_fixture_server"] = stop_fixture_server
        namespace["_run_restart_trial"] = run_restart_trial
        namespace["_run_agent_task_trial"] = run_agent_task_trial
        namespace["_agent_task_surfaces_complete"] = lambda _trials: not fail_trials
        namespace["print"] = lambda *_args, **_kwargs: events.append("evidence")

        error: Exception | None = None
        with tempfile.TemporaryDirectory() as temp_dir:
            chrome_bin = pathlib.Path(temp_dir) / "chrome"
            chromedriver_bin = pathlib.Path(temp_dir) / "chromedriver"
            chrome_bin.touch()
            chromedriver_bin.touch()
            with unittest.mock.patch.dict(
                os.environ,
                {
                    "CHROME_BIN": str(chrome_bin),
                    "CHROMEDRIVER_BIN": str(chromedriver_bin),
                },
            ):
                try:
                    self.assertEqual(namespace["main"](), 0)
                except Exception as caught:
                    error = caught
        return events, error

    def test_success_evidence_is_emitted_after_both_fixture_servers_stop(self) -> None:
        """A success artifact must follow, not precede, the owned teardown witnesses."""

        events, error = self._run_main_with_controlled_servers()

        self.assertIsNone(error)
        self.assertEqual(events, ["stopped:agent", "stopped:mv3", "evidence"])

    def test_cleanup_failure_cannot_leave_success_shaped_evidence(self) -> None:
        """A teardown failure must fail closed before successful evidence publication."""

        events, error = self._run_main_with_controlled_servers(fail_agent_cleanup=True)

        self.assertIsInstance(error, RuntimeError)
        self.assertEqual(str(error), "fixture server thread did not stop")
        self.assertEqual(events, ["stopped:agent", "stopped:mv3"])

    def test_failed_trial_gate_keeps_bounded_evidence_before_raising(self) -> None:
        """Moving success publication must not suppress bounded failure diagnostics."""

        events, error = self._run_main_with_controlled_servers(fail_trials=True)

        self.assertIsInstance(error, RuntimeError)
        self.assertEqual(
            str(error),
            "Manifest V3 repeatability gate failed: 0/3 trials passed",
        )
        self.assertEqual(events, ["evidence", "stopped:agent", "stopped:mv3"])


if __name__ == "__main__":
    unittest.main()
