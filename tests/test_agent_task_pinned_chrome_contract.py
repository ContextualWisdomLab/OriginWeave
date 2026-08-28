"""Contract for executing the controlled Agent Task fixture on pinned Chrome."""

from __future__ import annotations

import http.client
import inspect
import os
import pathlib
import runpy
import tempfile
import unittest
import unittest.mock
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
                        "result_semantics_verified": True,
                        "structured_value_field": "task_result",
                        "structured_value_sha256": "sha256:" + "0" * 64,
                        "extensions_disabled": True,
                        "profile_cleaned": True,
                        "browser_process_rss_bytes": 1,
                        "chromium_process_count": 1,
                        "chromium_process_set_rss_bytes": 1,
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
        first_server = object()
        first_thread = object()
        second_server = object()
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

    def test_agent_task_locates_controlled_targets_by_exact_role_and_name(self) -> None:
        """The controlled task must discover targets semantically rather than by fixture CSS."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_semantic_locator")
        runner = RUNNER.read_text(encoding="utf-8")
        self.assertIn("_find_element_by_accessible_role_name", namespace)
        for expected in (
            "MAX_SEMANTIC_LOCATOR_CANDIDATES",
            '"/elements"',
            '"css selector"',
            '"*"',
            '"semantic locator returned no exact match"',
            '"semantic locator returned multiple exact matches"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)
        self.assertNotIn('_find_element(driver_port, session_id, "#task-text")', runner)
        self.assertNotIn('_find_element(driver_port, session_id, "#submit-task")', runner)

    def test_semantic_role_name_locator_fails_closed_on_ambiguous_candidates(self) -> None:
        """Exact semantic discovery must reject zero, duplicate, malformed, and oversized sets."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_locator_behavior")
        locate = namespace["_find_element_by_accessible_role_name"]
        element_key = namespace["W3C_ELEMENT_KEY"]
        candidate_limit = namespace["MAX_SEMANTIC_LOCATOR_CANDIDATES"]

        def install_candidates(
            candidate_ids: list[str],
            semantics: dict[str, tuple[str, str]],
        ) -> None:
            locate.__globals__["_json_request"] = lambda *_args, **_kwargs: {
                "value": [{element_key: candidate_id} for candidate_id in candidate_ids]
            }
            locate.__globals__["_get_element_semantics"] = (
                lambda _port, _session, candidate_id: semantics[candidate_id]
            )

        install_candidates(
            ["candidate-a", "candidate-b"],
            {
                "candidate-a": ("button", "Other"),
                "candidate-b": ("button", "Submit task"),
            },
        )
        self.assertEqual(locate(4444, "session-a", "button", "Submit task"), "candidate-b")

        install_candidates(["candidate-a"], {"candidate-a": ("button", "Other")})
        with self.assertRaisesRegex(RuntimeError, "no exact match"):
            locate(4444, "session-a", "button", "Submit task")

        install_candidates(
            ["candidate-a", "candidate-b"],
            {
                "candidate-a": ("button", "Submit task"),
                "candidate-b": ("button", "Submit task"),
            },
        )
        with self.assertRaisesRegex(RuntimeError, "multiple exact matches"):
            locate(4444, "session-a", "button", "Submit task")

        install_candidates(
            [f"candidate-{index}" for index in range(candidate_limit + 1)],
            {},
        )
        with self.assertRaisesRegex(RuntimeError, "bounded candidate limit"):
            locate(4444, "session-a", "button", "Submit task")

        locate.__globals__["_json_request"] = lambda *_args, **_kwargs: {
            "value": [{"not-an-element-id": "candidate-a"}]
        }
        with self.assertRaisesRegex(RuntimeError, "malformed semantic locator candidate"):
            locate(4444, "session-a", "button", "Submit task")

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

    def test_agent_task_records_bounded_chromium_process_tree_rss(self) -> None:
        """The evidence runner must measure one bounded sampled process-set snapshot."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_tree_contract")
        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "MAX_BROWSER_PROCESS_TREE_SIZE",
            "MAX_PROC_PROCESS_SCAN_SIZE",
            "_parse_linux_proc_status_process_identity",
            "_parse_linux_proc_status_optional_rss_bytes",
            "_snapshot_linux_process_evidence",
            "_discover_linux_process_tree_ids",
            "_sample_linux_process_snapshot_rss_bytes",
            "_sample_linux_process_set_rss_bytes",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)
        for expected in (
            '"chromium_process_count"',
            '"chromium_process_set_rss_bytes"',
            '"failure_type"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_process_tree_and_rss_use_one_sampled_process_snapshot(self) -> None:
        """Root and descendant RSS must come from the same bounded status snapshot."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_one_snapshot_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        self.assertEqual(browser_pass_source.count("_snapshot_linux_process_evidence()"), 1)
        self.assertIn("_sample_linux_process_snapshot_rss_bytes", browser_pass_source)
        self.assertNotIn("_sample_linux_process_rss_bytes(browser_process_id)", browser_pass_source)

    def test_process_tree_helpers_are_bounded_and_fail_closed(self) -> None:
        """Sampled lineage must be deterministic and reject malformed membership/evidence."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_helper_contract")
        parse_identity = namespace["_parse_linux_proc_status_process_identity"]
        parse_optional_rss = namespace["_parse_linux_proc_status_optional_rss_bytes"]
        discover = namespace["_discover_linux_process_tree_ids"]
        sample_set = namespace["_sample_linux_process_set_rss_bytes"]

        self.assertEqual(parse_identity("Pid:\t10\nPPid:\t1\n"), (10, 1))
        self.assertIsNone(parse_optional_rss("Name:\tchrome\nPid:\t10\nPPid:\t1\n"))
        self.assertEqual(parse_optional_rss("VmRSS:\t7 kB\n"), 7 * 1024)
        with self.assertRaises(ValueError):
            parse_optional_rss("VmRSS:\t7 kB\nVmRSS:\t8 kB\n")

        evidence = {
            10: (1, 100),
            12: (10, None),
            11: (10, 200),
            13: (11, 300),
        }
        self.assertEqual(discover(10, evidence), (10, 11, 12, 13))
        self.assertEqual(sample_set((10, 11, 12, 13), evidence), 600)
        with self.assertRaises(ValueError):
            sample_set((10, 10), evidence)
        with self.assertRaises(ValueError):
            sample_set((10, 99), evidence)
        with self.assertRaises(RuntimeError):
            discover(99, evidence)

    def test_process_snapshot_ignores_symlinked_proc_entries(self) -> None:
        """The proc snapshot must not follow a symlink presented as a PID entry."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_proc_symlink_contract")
        with tempfile.TemporaryDirectory() as directory:
            temporary_root = pathlib.Path(directory)
            target = temporary_root / "target"
            target.mkdir()
            (target / "status").write_text(
                "Name:\tchrome\nPid:\t123\nPPid:\t1\nVmRSS:\t1 kB\n",
                encoding="utf-8",
            )
            symlinked_entry = temporary_root / "123"
            symlinked_entry.symlink_to(target, target_is_directory=True)
            with unittest.mock.patch.object(
                pathlib.Path, "iterdir", return_value=iter((symlinked_entry,))
            ):
                evidence = namespace["_snapshot_linux_process_evidence"]()

        self.assertEqual(evidence, {})

    def test_fixture_server_does_not_follow_symlinks_outside_fixture_root(self) -> None:
        """The controlled fixture server must not disclose a linked outside file."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_fixture_symlink_contract")
        with tempfile.TemporaryDirectory() as directory:
            temporary_root = pathlib.Path(directory)
            fixture_root = temporary_root / "fixture"
            fixture_root.mkdir()
            (fixture_root / "index.html").write_text("fixture", encoding="utf-8")
            secret_path = temporary_root / "secret.txt"
            secret_path.write_text("not-for-the-fixture", encoding="utf-8")
            (fixture_root / "linked.txt").symlink_to(secret_path)
            server, thread = namespace["_start_fixture_server"](fixture_root)
            try:
                connection = http.client.HTTPConnection(
                    "127.0.0.1", server.server_port, timeout=2
                )
                connection.request("GET", "/linked.txt")
                response = connection.getresponse()
                body = response.read()
                connection.close()
            finally:
                namespace["_stop_fixture_server"](server, thread)

        self.assertIn(response.status, {403, 404})
        self.assertNotIn(b"not-for-the-fixture", body)

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
