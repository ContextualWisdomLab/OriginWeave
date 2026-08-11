"""Contract for executing the controlled Agent Task fixture on pinned Chrome."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "agent_task_basic" / "index.html"
WORKFLOW = ROOT / ".github" / "workflows" / "mv3-compatibility.yml"


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

    def test_agent_task_records_bounded_chromium_process_tree_rss(self) -> None:
        """The evidence runner must measure one bounded sampled process-set snapshot."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_tree_contract")
        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "MAX_BROWSER_PROCESS_TREE_SIZE",
            "MAX_PROC_PROCESS_SCAN_SIZE",
            "_parse_linux_proc_status_process_identity",
            "_snapshot_linux_process_evidence",
            "_discover_linux_process_tree_ids",
            "_sample_linux_process_set_rss_bytes",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, namespace)
        self.assertNotIn("_parse_linux_children_process_ids", namespace)
        self.assertNotIn("_snapshot_linux_process_parent_ids", namespace)
        for expected in (
            '"chromium_process_count"',
            '"chromium_process_set_rss_bytes"',
            '"failure_type"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_process_tree_and_rss_use_one_sampled_process_snapshot(self) -> None:
        """Descendant RSS must come from the same bounded status snapshot as lineage."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_process_snapshot")
        discover = namespace["_discover_linux_process_tree_ids"]
        sample = namespace["_sample_linux_process_set_rss_bytes"]
        evidence = {
            10: (1, 100),
            20: (10, 200),
            30: (20, 300),
            40: (999, 400),
        }
        process_ids = discover(10, evidence)
        self.assertEqual(process_ids, (10, 20, 30))
        self.assertEqual(sample(process_ids, evidence), 600)
        with self.assertRaises(ValueError):
            sample((10, 50), evidence)

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

    def test_linux_status_identity_parser_is_strict_and_positive(self) -> None:
        """Process snapshot discovery must parse one unambiguous identity per status."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_parent_map_contract")
        parser = namespace["_parse_linux_proc_status_process_identity"]
        self.assertEqual(parser("Name:\tchrome\nPid:\t34\nPPid:\t12\n"), (34, 12))
        self.assertEqual(parser("Name:\tinit\nPid:\t1\nPPid:\t0\n"), (1, 0))
        for malformed in (
            "Name:\tchrome\nPid:\t34\n",
            "Name:\tchrome\nPPid:\t12\n",
            "Pid:\t0\nPPid:\t12\n",
            "Pid:\t34\nPPid:\t-1\n",
            "Pid:\tchild\nPPid:\t12\n",
            "Pid:\t34\nPid:\t35\nPPid:\t12\n",
            "Pid:\t34\nPPid:\t12\nPPid:\t13\n",
        ):
            with self.subTest(malformed=malformed):
                with self.assertRaises(ValueError):
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


if __name__ == "__main__":
    unittest.main()
