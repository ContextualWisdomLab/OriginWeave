"""Fail-first contract for pinned-Chromium extension update migration evidence."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"


class ManifestV3UpdateMigrationContractTests(unittest.TestCase):
    """Require a real restart across a controlled unpacked-extension version update."""

    def test_runner_uses_trial_local_extension_copy_and_version_update(self) -> None:
        """Update evidence must not rewrite the checked-in fixture or reuse global state."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            "shutil.copytree",
            "extension_dir",
            "INITIAL_EXTENSION_VERSION",
            "UPDATED_EXTENSION_VERSION",
            "_set_fixture_version",
            "update-migration",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)

    def test_service_worker_migrates_versioned_storage_state(self) -> None:
        """The fixture must expose deterministic version-state migration, not update inference."""

        worker = (FIXTURE / "service_worker.js").read_text(encoding="utf-8")
        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        for expected in (
            "chrome.runtime.getManifest().version",
            "originweave_fixture_schema_version",
            "storageMigration",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, worker)
        self.assertIn("originweaveStorageMigration", content)

    def test_runner_requires_updated_version_and_migrated_state(self) -> None:
        """A restart alone must not satisfy update/version-migration compatibility."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            '"extension_version"',
            '"storage_migration"',
            '"update-migration":',
            "UPDATED_EXTENSION_VERSION",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
