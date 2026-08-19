"""Regression contract for bounded ephemeral Chromium profile lifecycle."""

from __future__ import annotations

import pathlib
import runpy
import signal
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3EphemeralProfileContractTests(unittest.TestCase):
    """Prove each Chromium trial creates, reuses, and deletes one isolated profile."""

    def test_restart_trial_uses_empty_profile_then_deletes_it(self) -> None:
        """A trial must start empty, reuse only its own profile, then remove it."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_ephemeral_profile_contract")
        run_restart_trial = namespace["_run_restart_trial"]
        globals_ = run_restart_trial.__globals__
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
                self.assertTrue(
                    profile_path.joinpath("profile-created-by-browser").is_file()
                )
            observed_profiles.append(profile_path)
            call_count += 1
            return {
                "browser_version": namespace["PINNED_CHROME_VERSION"],
                "worker_start_count": call_count,
                "storage_persistence": expected_storage_persistence,
                "surfaces": {"fixture": True},
            }

        with unittest.mock.patch.dict(
            globals_, {"_run_browser_pass": fake_browser_pass}
        ):
            result = run_restart_trial(
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

    def test_browser_pass_owns_and_terminates_the_chromedriver_process_group(self) -> None:
        """Failure cleanup must signal the isolated driver group, not only its leader."""

        source = RUNNER.read_text(encoding="utf-8")
        self.assertIn("start_new_session=True", source)

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_process_group_contract")
        run_browser_pass = namespace["_run_browser_pass"]
        globals_ = run_browser_pass.__globals__
        driver = unittest.mock.Mock()
        driver.pid = 4242
        driver.wait.return_value = 0

        with tempfile.TemporaryDirectory(prefix="originweave-mv3-cleanup-") as profile_dir:
            with (
                unittest.mock.patch.dict(
                    globals_,
                    {
                        "_start_chromedriver": lambda _binary: (driver, 43123),
                        "_wait_for_driver": unittest.mock.Mock(
                            side_effect=RuntimeError("controlled startup failure")
                        ),
                    },
                ),
                unittest.mock.patch.object(globals_["os"], "killpg") as kill_process_group,
            ):
                with self.assertRaisesRegex(RuntimeError, "controlled startup failure"):
                    run_browser_pass(
                        pathlib.Path("/unused/chrome"),
                        pathlib.Path("/unused/chromedriver"),
                        "http://127.0.0.1/fixture",
                        profile_dir,
                        "initialized",
                    )

        kill_process_group.assert_called_once_with(driver.pid, signal.SIGTERM)
        driver.wait.assert_called_once_with(timeout=5)
        driver.terminate.assert_not_called()
        driver.kill.assert_not_called()


if __name__ == "__main__":
    unittest.main()
