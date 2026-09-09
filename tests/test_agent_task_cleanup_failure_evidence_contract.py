"""Contract for bounded Agent Task cleanup-failure provenance in emitted evidence."""

from __future__ import annotations

import io
import json
import os
import pathlib
import runpy
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskCleanupFailureEvidenceContractTests(unittest.TestCase):
    """Keep the first causal failure distinguishable from secondary cleanup failure."""

    def _assert_cleanup_failure_evidence(
        self,
        cleanup_kind: str,
        *,
        expected_failure_type: str,
        expected_cleanup_type: str,
    ) -> None:
        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_cleanup_evidence")
        main_globals = namespace["main"].__globals__
        primary = RuntimeError("buyer-secret-primary-detail")

        if cleanup_kind == "session":
            cleanup = namespace["_cleanup_browser_session_preserving_primary"]
            cleanup_error_type = namespace["BrowserSessionCleanupError"]

            def failed_cleanup(*_args: object, **_kwargs: object) -> None:
                raise OSError("buyer-secret-cleanup-detail")

            cleanup.__globals__["_cleanup_browser_session"] = failed_cleanup
            try:
                cleanup(9515, "session-1", primary)
            except cleanup_error_type as raised:
                cleanup_failure = raised
            else:
                self.fail("session cleanup double did not fail")
        elif cleanup_kind == "profile":
            cleanup_error_type = namespace["BrowserProfileCleanupError"]
            cleanup_failure = cleanup_error_type(
                OSError("buyer-secret-cleanup-detail"),
                primary,
            )
        else:
            self.fail("unsupported cleanup evidence test kind")

        class FakeServer:
            server_port = 9515

        def start_fixture_server(_directory: pathlib.Path) -> tuple[FakeServer, object]:
            return FakeServer(), object()

        def successful_restart_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            return {"trial_number": 1, "passed": True, "surfaces": {"worker": True}}

        def failed_agent_task_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise cleanup_failure

        main_globals.update(
            {
                "_start_fixture_server": start_fixture_server,
                "_stop_fixture_server": lambda *_args: None,
                "_run_restart_trial": successful_restart_trial,
                "_run_agent_task_trial": failed_agent_task_trial,
                "REPEATABILITY_TRIALS": 1,
                "AGENT_TASK_REPEATABILITY_TRIALS": 1,
            }
        )

        output = io.StringIO()
        with patch.dict(
            os.environ,
            {"CHROME_BIN": "/bin/sh", "CHROMEDRIVER_BIN": "/bin/sh"},
        ), redirect_stdout(output), self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task repeatability gate failed: 0/1 trials passed$",
        ):
            namespace["main"]()

        evidence = json.loads(output.getvalue())
        failed_trial = evidence["agent_task"]["trial_results"][0]
        self.assertEqual(failed_trial["failure_type"], expected_failure_type)
        self.assertEqual(failed_trial["failure_cause_type"], "RuntimeError")
        self.assertEqual(failed_trial["cleanup_error_type"], expected_cleanup_type)
        self.assertNotIn("buyer-secret-primary-detail", output.getvalue())
        self.assertNotIn("buyer-secret-cleanup-detail", output.getvalue())

    def test_session_cleanup_failure_retains_bounded_primary_and_cleanup_types(self) -> None:
        """Durable evidence must retain the primary browser type across DELETE failure."""

        self._assert_cleanup_failure_evidence(
            "session",
            expected_failure_type="BrowserSessionCleanupError",
            expected_cleanup_type="OSError",
        )

    def test_profile_cleanup_failure_retains_bounded_primary_and_cleanup_types(self) -> None:
        """Durable evidence must retain the primary browser type across profile cleanup."""

        self._assert_cleanup_failure_evidence(
            "profile",
            expected_failure_type="BrowserProfileCleanupError",
            expected_cleanup_type="OSError",
        )

    def test_profile_cleanup_after_session_cleanup_retains_bounded_chain(self) -> None:
        """Durable evidence must retain root, session-cleanup, and profile-cleanup types."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_nested_cleanup_evidence")
        main_globals = namespace["main"].__globals__
        session_cleanup_error_type = namespace["BrowserSessionCleanupError"]
        profile_cleanup_error_type = namespace["BrowserProfileCleanupError"]
        primary = RuntimeError("buyer-secret-primary-detail")
        session_cleanup_failure = session_cleanup_error_type(
            OSError("buyer-secret-session-cleanup-detail"),
            primary,
        )
        profile_cleanup_failure = profile_cleanup_error_type(
            PermissionError("buyer-secret-profile-cleanup-detail"),
            session_cleanup_failure,
        )

        class FakeServer:
            server_port = 9515

        def start_fixture_server(_directory: pathlib.Path) -> tuple[FakeServer, object]:
            return FakeServer(), object()

        def successful_restart_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            return {"trial_number": 1, "passed": True, "surfaces": {"worker": True}}

        def failed_agent_task_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise profile_cleanup_failure

        main_globals.update(
            {
                "_start_fixture_server": start_fixture_server,
                "_stop_fixture_server": lambda *_args: None,
                "_run_restart_trial": successful_restart_trial,
                "_run_agent_task_trial": failed_agent_task_trial,
                "REPEATABILITY_TRIALS": 1,
                "AGENT_TASK_REPEATABILITY_TRIALS": 1,
            }
        )

        output = io.StringIO()
        with patch.dict(
            os.environ,
            {"CHROME_BIN": "/bin/sh", "CHROMEDRIVER_BIN": "/bin/sh"},
        ), redirect_stdout(output), self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task repeatability gate failed: 0/1 trials passed$",
        ):
            namespace["main"]()

        failed_trial = json.loads(output.getvalue())["agent_task"]["trial_results"][0]
        self.assertEqual(failed_trial["failure_type"], "BrowserProfileCleanupError")
        self.assertEqual(failed_trial["failure_cause_type"], "RuntimeError")
        self.assertEqual(failed_trial["session_cleanup_error_type"], "OSError")
        self.assertEqual(failed_trial["cleanup_error_type"], "PermissionError")
        self.assertNotIn("buyer-secret-primary-detail", output.getvalue())
        self.assertNotIn("buyer-secret-session-cleanup-detail", output.getvalue())
        self.assertNotIn("buyer-secret-profile-cleanup-detail", output.getvalue())


if __name__ == "__main__":
    unittest.main()
