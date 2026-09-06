"""Behavioral regressions for reviewed browser cleanup evidence boundaries."""

from __future__ import annotations

import ast
import http.client
import pathlib
import runpy
import unittest
from unittest import mock

RUNNER = pathlib.Path(__file__).resolve().parents[1] / "scripts/ci/run_mv3_compatibility.py"


class AgentTaskReviewEvidenceContractTests(unittest.TestCase):
    """Keep failure evidence complete without relaxing success or retry gates."""

    def test_actual_success_predicate_rejects_invalid_pre_shutdown_counts(self) -> None:
        """Evaluate the production gate with valid evidence except for the disputed count."""

        namespace = runpy.run_path(str(RUNNER))
        module = ast.parse(RUNNER.read_text(encoding="utf-8"))
        assignment = next(
            node for node in ast.walk(module)
            if isinstance(node, ast.Assign)
            and any(isinstance(target, ast.Name) and target.id == "agent_task_surfaces_complete"
                    for target in node.targets)
        )
        predicate = compile(ast.Expression(assignment.value), str(RUNNER), "eval")
        trial = dict.fromkeys((
            "passed", "post_condition", "input_echo_verified", "url_unchanged",
            "input_semantics_verified", "submit_semantics_verified",
            "result_semantics_verified", "extensions_disabled", "profile_cleaned",
            "browser_process_terminated", "chromium_process_set_terminated",
        ), True)
        trial.update(
            structured_value_field="task_result", structured_value_sha256="sha256:" + "a" * 64,
            browser_process_rss_bytes=1024, chromium_process_count=3,
            chromium_process_set_rss_bytes=2048, semantic_observation_bytes=128,
            action_latency_ms=1, task_duration_ms=2,
        )
        namespace["agent_task_trials"] = [trial]
        for count in (0, 1, 2, None, True, False, -1, 3, 4, "1", 1.5):
            with self.subTest(count=count):
                trial["chromium_process_pre_shutdown_exit_count"] = count
                self.assertIs(eval(predicate, namespace), type(count) is int and 0 <= count < 3)

    def test_ordinary_failure_retains_only_valid_known_cleanup_fields(self) -> None:
        """Preserve driver and cleanup outcomes, but reject malformed returned evidence."""

        namespace = runpy.run_path(str(RUNNER))
        trial = namespace["_run_agent_task_trial"]
        evidence = {
            "failure_type": "RuntimeError", "browser_process_terminated": False,
            "driver_process_terminated": False, "driver_kill_fallback_used": True,
            "cleanup_failure_type": "TimeoutExpired", "session_cleanup_failure_type": "OSError",
            "untrusted_detail": "private marker",
        }
        with mock.patch.dict(trial.__globals__, {"_run_agent_task_browser_pass": lambda *_: evidence}):
            result = trial(pathlib.Path("unused-chrome"), pathlib.Path("unused-driver"), "http://127.0.0.1/fixture", 1)
            self.assertIs(result["passed"], False)
            self.assertIs(result["profile_cleaned"], True)
            self.assertNotIn("private marker", repr(result))
            for key in evidence.keys() - {"untrusted_detail"}:
                self.assertEqual(result.get(key), evidence[key], key)
            for key, invalid in (
                ("driver_process_terminated", 1), ("driver_kill_fallback_used", "true"),
                ("cleanup_failure_type", ""), ("session_cleanup_failure_type", None),
            ):
                with self.subTest(key=key), mock.patch.dict(evidence, {key: invalid}):
                    with self.assertRaises(RuntimeError):
                        trial(pathlib.Path("unused-chrome"), pathlib.Path("unused-driver"), "http://127.0.0.1/fixture", 2)

    def test_protocol_faults_keep_real_browser_cleanup_evidence(self) -> None:
        """Mid-pass protocol faults stay terminal and preserve observed root cleanup."""

        for lane, waiter_name, observed_exit, expected_args in (
            (
                "agent_task", "_wait_for_linux_process_teardown",
                (False, False), (321, 654, ((321, 654),)),
            ),
            (
                "agent_task_forced_close", "_wait_for_linux_process_teardown",
                (False, False), (321, 654, ((321, 654),)),
            ),
        ):
            for error in (http.client.BadStatusLine("private marker"), http.client.IncompleteRead(b"private marker")):
                with self.subTest(lane=lane, error=type(error).__name__):
                    namespace = runpy.run_path(str(RUNNER))
                    trial = namespace[f"_run_{lane}_trial"]
                    driver = mock.Mock()
                    exit_wait = mock.Mock(return_value=observed_exit)

                    def request(_port, method, target, *_args):
                        if target == "/session":
                            return {"value": {"sessionId": "controlled-session", "capabilities": {
                                "browserVersion": namespace["PINNED_CHROME_VERSION"], "goog:processID": 321,
                            }}}
                        if method == "DELETE":
                            return {}
                        raise error

                    replacements = {
                        "_free_loopback_port": lambda: 12345, "_wait_for_driver": lambda *_: None,
                        "_json_request": request,
                        "_read_linux_proc_stat_process_identity": lambda *_: (321, 654),
                        waiter_name: exit_wait,
                    }
                    with mock.patch.dict(trial.__globals__, replacements), mock.patch.object(namespace["subprocess"], "Popen", return_value=driver):
                        result = trial(pathlib.Path("unused-chrome"), pathlib.Path("unused-driver"), "http://127.0.0.1/fixture", 3)
                    self.assertIs(result["passed"], False)
                    self.assertIs(result["profile_cleaned"], True)
                    self.assertIs(result["browser_process_terminated"], False)
                    self.assertIs(result["driver_process_terminated"], True)
                    self.assertEqual(result["failure_type"], type(error).__name__)
                    self.assertNotIn("private marker", repr(result))
                    exit_wait.assert_called_once_with(*expected_args)
                    driver.terminate.assert_called_once_with()

    def test_all_trial_boundaries_redact_protocol_failures_before_identity_capture(self) -> None:
        """Each outer trial must clean its profile and report only a terminal error type."""

        for trial_name, pass_name in (
            ("_run_restart_trial", "_run_browser_pass"),
            ("_run_agent_task_trial", "_run_agent_task_browser_pass"),
            ("_run_agent_task_forced_close_trial", "_run_agent_task_forced_close_browser_pass"),
        ):
            with self.subTest(trial=trial_name):
                namespace = runpy.run_path(str(RUNNER))
                trial = namespace[trial_name]
                browser_pass = mock.Mock(side_effect=http.client.BadStatusLine("private marker"))
                with mock.patch.dict(trial.__globals__, {pass_name: browser_pass}):
                    result = trial(pathlib.Path("unused-chrome"), pathlib.Path("unused-driver"), "http://127.0.0.1/fixture", 4)
                self.assertIs(result["passed"], False)
                self.assertIs(result["profile_cleaned"], True)
                self.assertEqual(result["failure_type"], "BadStatusLine")
                self.assertNotIn("private marker", repr(result))
                self.assertNotIn("browser_process_terminated", result)
                browser_pass.assert_called_once()

    def test_main_records_protocol_faults_without_exposing_remote_diagnostics(self) -> None:
        """Final evidence preserves all failed trials and still rejects acceptance."""

        namespace = runpy.run_path(str(RUNNER))
        main = namespace["main"]
        failed_trial = mock.Mock(side_effect=http.client.BadStatusLine("private marker"))
        stop_server = mock.Mock()
        replacements = dict.fromkeys((
            "_run_restart_trial", "_run_agent_task_trial", "_run_agent_task_forced_close_trial",
        ), failed_trial)
        replacements.update(
            _start_fixture_server=lambda *_: (mock.Mock(server_port=12345), mock.Mock()),
            _stop_fixture_server=stop_server,
        )
        with mock.patch.dict(main.__globals__, replacements), mock.patch.object(pathlib.Path, "is_file", return_value=True), mock.patch("builtins.print") as output:
            with self.assertRaisesRegex(RuntimeError, "profile cleanup gate failed"):
                main()
        evidence = namespace["json"].loads(output.call_args.args[0])
        self.assertNotIn("private marker", repr(evidence))
        for lane in (evidence, evidence["agent_task"], evidence["agent_task"]["forced_close"]):
            self.assertEqual(len(lane["trial_results"]), lane["repeatability_trials"])
            self.assertEqual(lane["successful_trials"], 0)
            for trial in lane["trial_results"]:
                self.assertIs(trial["passed"], False)
                self.assertEqual(trial["failure_type"], "BadStatusLine")
        self.assertEqual(stop_server.call_count, 2)

    def test_obsolete_http_body_is_not_accepted_as_closed_context_evidence(self) -> None:
        """The redacted request producer no longer emits an HTTP-JSON diagnostic format."""

        namespace = runpy.run_path(str(RUNNER))
        recognize = namespace["_is_no_such_window_runtime_error"]
        self.assertFalse(recognize(RuntimeError('WebDriver HTTP 404: {"value":{"error":"no such window"}}')))
        self.assertTrue(recognize(RuntimeError("WebDriver error: no such window: response details redacted")))


if __name__ == "__main__":
    unittest.main()
