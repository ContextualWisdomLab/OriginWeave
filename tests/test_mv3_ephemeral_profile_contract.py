"""Regression contract for bounded ephemeral Chromium profile lifecycle."""

from __future__ import annotations

import importlib.util
import pathlib
import unittest
from unittest import mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER_PATH = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


def _load_runner():
    """Load the compatibility runner without executing its command-line entry point."""

    spec = importlib.util.spec_from_file_location("originweave_mv3_runner", RUNNER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load MV3 compatibility runner")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class ManifestV3EphemeralProfileContractTests(unittest.TestCase):
    """Prove each Chromium trial creates, reuses, and deletes one isolated profile."""

    def test_restart_trial_uses_empty_profile_then_deletes_it(self) -> None:
        """A trial must start empty, reuse only its own profile, then remove it."""

        runner = _load_runner()
        observed_profiles: list[pathlib.Path] = []
        call_count = 0

        def fake_browser_pass(
            _chrome_bin: pathlib.Path,
            _chromedriver_bin: pathlib.Path,
            _fixture_url: str,
            profile_dir: str,
            expected_storage_persistence: str,
        ) -> dict[str, object]:
            nonlocal call_count
            profile_path = pathlib.Path(profile_dir)
            if call_count == 0:
                self.assertTrue(profile_path.is_dir())
                self.assertEqual(list(profile_path.iterdir()), [])
                profile_path.joinpath("profile-created-by-browser").write_text(
                    "fixture", encoding="utf-8"
                )
            else:
                self.assertEqual(profile_path, observed_profiles[0])
                self.assertTrue(profile_path.joinpath("profile-created-by-browser").is_file())
            observed_profiles.append(profile_path)
            call_count += 1
            return {
                "browser_version": runner.PINNED_CHROME_VERSION,
                "worker_start_count": call_count,
                "storage_persistence": expected_storage_persistence,
                "surfaces": {"fixture": True},
            }

        with mock.patch.object(runner, "_run_browser_pass", side_effect=fake_browser_pass):
            result = runner._run_restart_trial(
                pathlib.Path("/unused/chrome"),
                pathlib.Path("/unused/chromedriver"),
                "http://127.0.0.1/fixture",
                1,
            )

        self.assertEqual(len(observed_profiles), 2)
        self.assertEqual(observed_profiles[0], observed_profiles[1])
        self.assertFalse(observed_profiles[0].exists())
        self.assertNotIn(str(observed_profiles[0]), repr(result))
        self.assertIs(result.get("passed"), True)


if __name__ == "__main__":
    unittest.main()
