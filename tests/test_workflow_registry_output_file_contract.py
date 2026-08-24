"""Regression contract for workflow-registry audit output path authority."""

from __future__ import annotations

import json
import pathlib
import runpy
import tempfile
import unittest

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
    """Prevent an audit output path from inheriting ambient symlink authority."""

    def test_symbolic_link_output_cannot_overwrite_its_target(self) -> None:
        """A caller-controlled symlink must never redirect canonical audit evidence output."""

        namespace = runpy.run_path(str(AUDITOR), run_name="workflow_output_file_contract")
        main = namespace["main"]

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


if __name__ == "__main__":
    unittest.main()
