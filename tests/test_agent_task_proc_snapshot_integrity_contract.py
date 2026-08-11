"""Integrity regressions for sampled Linux process evidence in the controlled browser fixture."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskProcSnapshotIntegrityContractTests(unittest.TestCase):
    """Keep nonresident processes distinct from malformed or ambiguous proc evidence."""

    def test_optional_rss_parser_accepts_absent_or_zero_but_rejects_ambiguity(self) -> None:
        """Only an unambiguous absent/zero VmRSS may mean no resident bytes."""

        namespace = runpy.run_path(str(RUNNER), run_name="agent_task_proc_integrity_contract")
        parser = namespace["_parse_linux_proc_status_optional_rss_bytes"]

        self.assertIsNone(parser("Name:\tchrome\nPid:\t34\nPPid:\t12\n"))
        self.assertIsNone(parser("Name:\tchrome\nVmRSS:\t0 kB\n"))
        self.assertEqual(parser("Name:\tchrome\nVmRSS:\t123 kB\n"), 123 * 1024)

        for malformed in (
            "VmRSS:\t123 kB\nVmRSS:\t124 kB\n",
            "VmRSS:\t0 kB\nVmRSS:\t124 kB\n",
            "VmRSS:\t123 MB\n",
            "VmRSS:\t123 kB extra\n",
            "VmRSS:\tnot-a-number kB\n",
        ):
            with self.subTest(malformed=malformed):
                with self.assertRaises(ValueError):
                    parser(malformed)


if __name__ == "__main__":
    unittest.main()
