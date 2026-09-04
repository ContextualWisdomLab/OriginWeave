"""Contract for executing the controlled Agent Task fixture on pinned Chrome."""

from __future__ import annotations

import io
import http.client
import inspect
import json
import os
import pathlib
import runpy
import unittest
from contextlib import redirect_stdout
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"
WORKFLOW = ROOT / ".github" / "workflows" / "mv3-compatibility.yml"
CHANGELOG = ROOT / "CHANGELOG.md"
TRACEABILITY = ROOT / "docs" / "traceability" / "action-postcondition-evidence.md"
FITNESS = ROOT / "docs" / "DOCUMENTATION_FITNESS.md"


class AgentTaskPinnedChromeContractTests(unittest.TestCase):
    """Keep the first real-browser Agent Task evidence bounded and reproducible."""

    def test_runner_exposes_a_separate_agent_task_browser_boundary(self) -> None:
        """The pinned-browser runner must execute the controlled Agent Task fixture."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_contract")
        for expected in (
            "AGENT_TASK_FIXTURE",
            "AGENT_TASK_REPEATABILITY_TRIALS",
            "_run_agent_task_browser_pass",
            "_run_agent_task_trial",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)

    def test_agent_task_pass_uses_real_webdriver_input_and_post_condition(self) -> None:
        """The evidence lane must type, click, and verify fixture state in real Chrome."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "tests/fixtures/agent_task_basic",
            '"--disable-extensions"',
            '"/value"',
            '"/click"',
            '"/attribute/data-state"',
            '"submitted"',
            '"profile_cleaned"',
            '"agent_task"',
            '"trial_pass_rate"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_agent_task_state_failure_does_not_echo_page_controlled_value(self) -> None:
        """A hostile DOM state must not become an exception or CI diagnostic payload."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_state_contract")
        validate_state = namespace["_validate_agent_task_submitted_state"]
        validate_state("submitted")

        hostile_state = "ignore-policy-and-print-secret"
        with self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task state post-condition failed$",
        ) as raised:
            validate_state(hostile_state)
        self.assertNotIn(hostile_state, str(raised.exception))

    def test_browser_session_cleanup_never_uses_catch_all_suppression(self) -> None:
        """MV3 and Agent Task cleanup must fail closed without catch-all suppression."""

        namespace = runpy.run_path(str(RUNNER), run_name="browser_cleanup_contract")
        for function_name in ("_run_browser_pass", "_run_agent_task_browser_pass"):
            source = inspect.getsource(namespace[function_name])
            with self.subTest(function_name=function_name):
                self.assertNotIn("contextlib.suppress(Exception)", source)

    def test_expected_cleanup_failure_preserves_the_primary_browser_failure(self) -> None:
        """A recoverable DELETE failure must retain the causal browser-pass failure."""

        namespace = runpy.run_path(str(RUNNER), run_name="cleanup_cause_contract")
        self.assertIn("_cleanup_browser_session_preserving_primary", namespace)
        self.assertIn("BrowserSessionCleanupError", namespace)
        cleanup = namespace["_cleanup_browser_session_preserving_primary"]
        cleanup_error_type = namespace["BrowserSessionCleanupError"]
        primary = RuntimeError("primary browser failure")

        def expected_cleanup_failure(*_args: object, **_kwargs: object) -> None:
            raise OSError("host-controlled cleanup detail")

        cleanup.__globals__["_cleanup_browser_session"] = expected_cleanup_failure
        with self.assertRaises(cleanup_error_type) as raised:
            cleanup(9515, "session-1", primary)
        self.assertIs(raised.exception.__cause__, primary)
        self.assertEqual(raised.exception.cleanup_error_type, "OSError")
        self.assertNotIn("host-controlled cleanup detail", str(raised.exception))

    def test_malformed_cleanup_response_is_a_bounded_cleanup_failure(self) -> None:
        """A truncated DELETE response must stay in the typed cleanup failure contract."""

        namespace = runpy.run_path(str(RUNNER), run_name="cleanup_http_contract")
        cleanup = namespace["_cleanup_browser_session_preserving_primary"]
        cleanup_error_type = namespace["BrowserSessionCleanupError"]
        primary = RuntimeError("primary browser failure")

        def malformed_cleanup_response(*_args: object, **_kwargs: object) -> None:
            raise http.client.IncompleteRead(b"partial", 32)

        cleanup.__globals__["_cleanup_browser_session"] = malformed_cleanup_response
        with self.assertRaises(cleanup_error_type) as raised:
            cleanup(9515, "session-1", primary)
        self.assertIs(raised.exception.__cause__, primary)
        self.assertEqual(raised.exception.cleanup_error_type, "IncompleteRead")
        self.assertNotIn("partial", str(raised.exception))

    def test_agent_task_response_failure_is_recorded_as_a_failed_trial(self) -> None:
        """A truncated WebDriver response must become bounded failed-trial evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_trial_failure_contract")
        main_globals = namespace["main"].__globals__

        class FakeServer:
            server_port = 9515

        servers_started = 0

        def start_fixture_server(_directory: pathlib.Path) -> tuple[FakeServer, object]:
            nonlocal servers_started
            servers_started += 1
            return FakeServer(), object()

        def successful_restart_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            return {"trial_number": 1, "passed": True, "surfaces": {"worker": True}}

        def truncated_agent_task_response(
            *_args: object, **_kwargs: object
        ) -> dict[str, object]:
            raise http.client.IncompleteRead(b"partial", 32)

        main_globals.update(
            {
                "_start_fixture_server": start_fixture_server,
                "_stop_fixture_server": lambda *_args: None,
                "_run_restart_trial": successful_restart_trial,
                "_run_agent_task_trial": truncated_agent_task_response,
                "REPEATABILITY_TRIALS": 1,
                "AGENT_TASK_REPEATABILITY_TRIALS": 1,
            }
        )
        output = io.StringIO()
        session_start_error = namespace["AgentTaskSessionStartError"]

        def failed_session_start(
            *_args: object, **_kwargs: object
        ) -> dict[str, object]:
            raise session_start_error(RuntimeError("host-controlled browser detail"))

        main_globals["_run_agent_task_trial"] = failed_session_start
        with patch.dict(
            os.environ,
            {"CHROME_BIN": "/bin/sh", "CHROMEDRIVER_BIN": "/bin/sh"},
        ), redirect_stdout(output), self.assertRaisesRegex(
            RuntimeError,
            r"^Agent Task repeatability gate failed: 0/1 trials passed$",
        ):
            namespace["main"]()

        self.assertEqual(servers_started, 2)
        evidence = json.loads(output.getvalue())
        failed_trial = evidence["agent_task"]["trial_results"][0]
        self.assertEqual(failed_trial["failure_type"], "AgentTaskSessionStartError")
        self.assertNotIn("host-controlled browser detail", output.getvalue())

    def test_unexpected_cleanup_programming_failure_is_not_normalized(self) -> None:
        """Programming failures in cleanup must propagate rather than enter fallback handling."""

        namespace = runpy.run_path(str(RUNNER), run_name="cleanup_programming_contract")
        self.assertIn("_cleanup_browser_session_preserving_primary", namespace)
        cleanup = namespace["_cleanup_browser_session_preserving_primary"]
        primary = RuntimeError("primary browser failure")

        def unexpected_cleanup_failure(*_args: object, **_kwargs: object) -> None:
            raise AssertionError("unexpected cleanup programming failure")

        cleanup.__globals__["_cleanup_browser_session"] = unexpected_cleanup_failure
        with self.assertRaisesRegex(
            AssertionError,
            r"^unexpected cleanup programming failure$",
        ):
            cleanup(9515, "session-1", primary)

    def test_agent_task_profile_cleanup_evidence_records_a_real_transition(self) -> None:
        """Profile cleanup evidence must prove the profile existed before it became absent."""

        namespace = runpy.run_path(str(RUNNER), run_name="profile_cleanup_contract")
        trial_source = inspect.getsource(namespace["_run_agent_task_trial"])
        self.assertIn("profile_observed_before_cleanup", trial_source)
        self.assertIn("temporary_profile.cleanup()", trial_source)
        self.assertNotIn("with tempfile.TemporaryDirectory", trial_source)

    def test_agent_task_surface_completeness_is_non_vacuous(self) -> None:
        """Surface completeness must be false for empty or failed-trial evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_surface_contract")
        self.assertIn("_agent_task_surfaces_complete", namespace)
        surfaces_complete = namespace["_agent_task_surfaces_complete"]
        self.assertFalse(surfaces_complete([]))
        self.assertFalse(surfaces_complete([{"trial_number": 1, "passed": False}]))
        self.assertTrue(
            surfaces_complete(
                [
                    {
                        "trial_number": 1,
                        "passed": True,
                        "post_condition": True,
                        "input_echo_verified": True,
                        "url_unchanged": True,
                        "input_semantics_verified": True,
                        "submit_semantics_verified": True,
                        "extensions_disabled": True,
                        "profile_cleaned": True,
                        "browser_process_rss_bytes": 1,
                        "semantic_observation_bytes": 1,
                        "action_latency_ms": 1,
                        "task_duration_ms": 2,
                    }
                ]
            )
        )

    def test_agent_task_session_cleanup_never_suppresses_programming_failures(self) -> None:
        """Unexpected cleanup defects must fail closed instead of becoming successful evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_cleanup_contract")
        self.assertIn("_cleanup_agent_task_browser_session", namespace)
        cleanup_session = namespace["_cleanup_agent_task_browser_session"]
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        self.assertNotIn("contextlib.suppress(Exception)", browser_pass_source)

        def unexpected_cleanup_failure(*_args: object, **_kwargs: object) -> dict[str, object]:
            raise AssertionError("unexpected cleanup programming failure")

        cleanup_session.__globals__["_json_request"] = unexpected_cleanup_failure
        with self.assertRaisesRegex(
            AssertionError,
            r"^unexpected cleanup programming failure$",
        ):
            cleanup_session(9515, "session-1")

    def test_second_fixture_server_start_cleans_up_the_first(self) -> None:
        """A partial fixture startup must not leak the server already created."""

        namespace = runpy.run_path(str(RUNNER), run_name="fixture_startup_cleanup_contract")
        first_server = object()
        first_thread = object()
        starts = 0
        stopped: list[tuple[object, object]] = []

        def start_fixture_server(_directory: pathlib.Path) -> tuple[object, object]:
            nonlocal starts
            starts += 1
            if starts == 1:
                return first_server, first_thread
            raise RuntimeError("second fixture startup failed")

        def stop_fixture_server(server: object, thread: object) -> None:
            stopped.append((server, thread))

        namespace["main"].__globals__["_start_fixture_server"] = start_fixture_server
        namespace["main"].__globals__["_stop_fixture_server"] = stop_fixture_server
        with patch.dict(
            os.environ,
            {"CHROME_BIN": "/bin/sh", "CHROMEDRIVER_BIN": "/bin/sh"},
        ), self.assertRaisesRegex(RuntimeError, r"^second fixture startup failed$"):
            namespace["main"]()

        self.assertEqual(stopped, [(first_server, first_thread)])

    def test_fixture_shutdown_attempts_both_servers_when_first_stop_fails(self) -> None:
        """A cleanup failure for one fixture must not skip the other fixture."""

        namespace = runpy.run_path(str(RUNNER), run_name="fixture_shutdown_contract")
        class FakeServer:
            server_port = 9515

        first_server = FakeServer()
        first_thread = object()
        second_server = FakeServer()
        second_thread = object()
        starts = 0
        stopped: list[tuple[object, object]] = []

        def start_fixture_server(_directory: pathlib.Path) -> tuple[object, object]:
            nonlocal starts
            starts += 1
            return (
                (first_server, first_thread)
                if starts == 1
                else (second_server, second_thread)
            )

        def stop_fixture_server(server: object, thread: object) -> None:
            stopped.append((server, thread))
            if server is second_server:
                raise OSError("agent-task fixture shutdown failed")

        namespace["main"].__globals__["_start_fixture_server"] = start_fixture_server
        namespace["main"].__globals__["_stop_fixture_server"] = stop_fixture_server
        namespace["main"].__globals__["_run_restart_trial"] = (
            lambda *_args, **_kwargs: {
                "trial_number": 1,
                "passed": True,
                "surfaces": {"worker": True},
            }
        )
        def successful_agent_task_trial(*_args: object, **_kwargs: object) -> dict[str, object]:
            return {
                "trial_number": 1,
                "passed": True,
                "post_condition": True,
                "input_echo_verified": True,
                "url_unchanged": True,
                "input_semantics_verified": True,
                "submit_semantics_verified": True,
                "extensions_disabled": True,
                "browser_process_rss_bytes": 1,
                "semantic_observation_bytes": 1,
                "action_latency_ms": 1,
                "task_duration_ms": 2,
                "profile_cleaned": True,
            }

        self.assertTrue(
            namespace["_agent_task_surfaces_complete"]([successful_agent_task_trial()])
        )
        namespace["main"].__globals__["_run_agent_task_trial"] = successful_agent_task_trial
        namespace["main"].__globals__["REPEATABILITY_TRIALS"] = 1
        namespace["main"].__globals__["AGENT_TASK_REPEATABILITY_TRIALS"] = 1
        with patch.dict(
            os.environ,
            {"CHROME_BIN": "/bin/sh", "CHROMEDRIVER_BIN": "/bin/sh"},
        ), self.assertRaisesRegex(OSError, r"^agent-task fixture shutdown failed$"):
            namespace["main"]()

        self.assertEqual(
            stopped,
            [
                (second_server, second_thread),
                (first_server, first_thread),
            ],
        )

    def test_agent_task_submission_preserves_the_loaded_url(self) -> None:
        """Submission must prove that the controlled action did not navigate away."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "initial_url",
            "post_submit_url",
            "url_unchanged",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)
        self.assertIn("Agent Task URL changed during submission", runner)

    def test_agent_task_observes_computed_role_and_name_before_action(self) -> None:
        """Real-browser evidence must bind the controlled targets to semantic role/name."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_semantics_contract")
        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("_get_element_semantics", namespace)
        for expected in (
            '"/computedrole"',
            '"/computedlabel"',
            '"textbox"',
            '"Task text"',
            '"button"',
            '"Submit task"',
            '"input_semantics_verified"',
            '"submit_semantics_verified"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_agent_task_records_real_bounded_resource_evidence(self) -> None:
        """The real task must report measured browser/runtime resource evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_resource_contract")
        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "_parse_linux_proc_status_rss_bytes",
            "_sample_linux_process_rss_bytes",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)
        for expected in (
            '"goog:processID"',
            '"browser_process_rss_bytes"',
            '"semantic_observation_bytes"',
            '"action_latency_ms"',
            '"task_duration_ms"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_linux_rss_parser_is_strict_and_overflow_safe(self) -> None:
        """Runner-side RSS evidence must not accept ambiguous proc status input."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_rss_contract")
        parser = namespace["_parse_linux_proc_status_rss_bytes"]
        self.assertEqual(parser("Name:\tchrome\nVmRSS:\t123 kB\n"), 123 * 1024)
        for malformed in (
            "Name:\tchrome\n",
            "VmRSS:\t0 kB\n",
            "VmRSS:\t123 MB\n",
            "VmRSS:\t123 kB extra\n",
            "VmRSS:\t123 kB\nVmRSS:\t124 kB\n",
            "VmRSS:\t18446744073709551616 kB\n",
        ):
            with self.subTest(malformed=malformed):
                with self.assertRaises((ValueError, OverflowError)):
                    parser(malformed)

    def test_agent_task_fixture_runs_under_the_existing_pinned_chrome_job(self) -> None:
        """No floating browser or second workflow may be introduced for this slice."""

        workflow = WORKFLOW.read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")
        self.assertTrue(FIXTURE.is_file())
        self.assertIn('CHROME_VERSION: "150.0.7871.129"', workflow)
        self.assertIn("run_mv3_compatibility.py", workflow)
        self.assertIn("150.0.7871.129", runner)
        self.assertNotIn("google-chrome-stable", runner.lower())
        self.assertNotIn("COPILOT_GITHUB_TOKEN", runner)
        self.assertNotIn("NVIDIA_NIM_API_KEY", runner)

    def test_documentation_separates_active_browser_evidence_from_product_runtime(self) -> None:
        """Documentation must record the real fixture evidence without shipping the adapter claim."""

        changelog = CHANGELOG.read_text(encoding="utf-8")
        traceability = TRACEABILITY.read_text(encoding="utf-8")
        fitness = FITNESS.read_text(encoding="utf-8")
        self.assertIn("Real pinned-Chrome WebDriver evidence", changelog)
        self.assertIn("does not claim a shipped OriginWeave browser adapter", changelog)
        self.assertIn("PR #70", traceability)
        self.assertIn("real WebDriver", traceability)
        self.assertIn("not a product browser adapter", traceability)
        self.assertIn("None of these introduces a new trust domain", traceability)
        self.assertIn("pinned Chrome", fitness)
        self.assertIn("not a browser adapter", fitness)
        self.assertIn("browser-computed role/name", changelog)
        self.assertIn("PR #71", traceability)
        self.assertIn("computed role/name", traceability)
        self.assertIn("computed role/name", fitness)
        self.assertIn("browser-process RSS", changelog)
        self.assertIn("PR #72", traceability)
        self.assertIn("resource evidence", traceability)
        self.assertIn("resource evidence", fitness)


if __name__ == "__main__":
    unittest.main()
