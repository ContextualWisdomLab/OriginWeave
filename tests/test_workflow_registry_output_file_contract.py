"""Regression contract for workflow-registry audit output path authority."""

from __future__ import annotations

import json
import os
import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUDITOR = ROOT / "scripts" / "ci" / "audit_workflow_registry.py"
SHA = "a" * 40


def _payload() -> dict:
    """Return one valid single-page read-only workflow-registry fixture."""

    return {
        "schema_version": 1,
        "expected_default_branch_sha": SHA,
        "observed_default_branch_sha": SHA,
        "observed_at": "2026-08-24T04:00:00Z",
        "reported_total_count": 1,
        "protected_workflow_paths": [".github/workflows/ci.yml"],
        "active_pr_workflow_paths": [],
        "registry_pages": [
            {
                "page": 1,
                "status_code": 200,
                "has_next": False,
                "workflows": [
                    {
                        "id": 1,
                        "name": "CI",
                        "path": ".github/workflows/ci.yml",
                        "state": "active",
                    }
                ],
            }
        ],
    }


class WorkflowRegistryOutputFileContractTests(unittest.TestCase):
    """Prevent an audit output path from inheriting ambient link authority."""

    def _main(self):
        """Load the audit CLI entrypoint without executing its process wrapper."""

        namespace = runpy.run_path(str(AUDITOR), run_name="workflow_output_file_contract")
        return namespace["main"]

    def test_symbolic_link_output_cannot_overwrite_its_target(self) -> None:
        """A caller-controlled symlink must never redirect canonical audit evidence output."""

        main = self._main()
        with tempfile.TemporaryDirectory(prefix="originweave-workflow-output-") as directory:
            root = pathlib.Path(directory)
            source = root / "registry.json"
            source.write_text(json.dumps(_payload()), encoding="utf-8")
            target = root / "operator-owned.txt"
            target.write_text("sentinel\n", encoding="utf-8")
            output = root / "audit.json"
            try:
                output.symlink_to(target)
            except OSError as error:
                self.skipTest(f"symbolic links are unavailable on this platform: {error}")

            self.assertEqual(main([str(source), "--output", str(output)]), 1)
            self.assertEqual(target.read_text(encoding="utf-8"), "sentinel\n")

    def test_hard_link_output_cannot_overwrite_its_peer(self) -> None:
        """A caller-controlled hard link must not grant write authority to a peer path."""

        main = self._main()
        with tempfile.TemporaryDirectory(prefix="originweave-workflow-output-") as directory:
            root = pathlib.Path(directory)
            source = root / "registry.json"
            source.write_text(json.dumps(_payload()), encoding="utf-8")
            target = root / "operator-owned.txt"
            target.write_text("sentinel\n", encoding="utf-8")
            output = root / "audit.json"
            try:
                output.hardlink_to(target)
            except OSError as error:
                self.skipTest(f"hard links are unavailable on this platform: {error}")

            self.assertEqual(main([str(source), "--output", str(output)]), 1)
            self.assertEqual(target.read_text(encoding="utf-8"), "sentinel\n")

    def test_hard_link_race_after_descriptor_check_cannot_alias_output(self) -> None:
        """A hard link added after descriptor inspection must not gain write authority."""

        main = self._main()
        with tempfile.TemporaryDirectory(prefix="originweave-workflow-output-") as directory:
            root = pathlib.Path(directory)
            source = root / "registry.json"
            source.write_text(json.dumps(_payload()), encoding="utf-8")
            output = root / "audit.json"
            output.write_text("sentinel\n", encoding="utf-8")
            peer = root / "raced-peer.txt"
            real_fstat = os.fstat
            fstat_calls = 0

            def fstat_with_hard_link_race(descriptor: int) -> os.stat_result:
                nonlocal fstat_calls
                result = real_fstat(descriptor)
                fstat_calls += 1
                if fstat_calls == 2:
                    try:
                        peer.hardlink_to(output)
                    except OSError as error:
                        self.skipTest(
                            f"hard links are unavailable on this platform: {error}"
                        )
                return result

            with unittest.mock.patch("os.fstat", side_effect=fstat_with_hard_link_race):
                self.assertEqual(main([str(source), "--output", str(output)]), 1)

            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")
            self.assertEqual(peer.read_text(encoding="utf-8"), "sentinel\n")


if __name__ == "__main__":
    unittest.main()
