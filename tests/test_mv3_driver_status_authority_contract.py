"""Regression contracts for bounded ChromeDriver startup/status authority."""

from __future__ import annotations

import pathlib
import runpy
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3DriverStatusAuthorityTests(unittest.TestCase):
    """Startup evidence must come from the pinned ChromeDriver status shape."""

    def test_ready_status_rejects_a_different_chromedriver_build(self) -> None:
        """A ready loopback endpoint cannot impersonate the pinned driver build."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_driver_status_authority")
        wait_for_driver = namespace["_wait_for_driver"]
        globals_ = wait_for_driver.__globals__

        def mismatched_status(
            _driver_port: int,
            method: str,
            path: str,
            _payload=None,
            *,
            timeout: float = 5.0,
        ) -> dict[str, object]:
            self.assertEqual(method, "GET")
            self.assertEqual(path, "/status")
            self.assertGreater(timeout, 0)
            return {
                "value": {
                    "ready": True,
                    "build": {"version": "149.0.0.0 (controlled-mismatch)"},
                }
            }

        with unittest.mock.patch.dict(globals_, {"_json_request": mismatched_status}):
            with self.assertRaisesRegex(RuntimeError, "ChromeDriver status identity mismatch"):
                wait_for_driver(43123)

    def test_malformed_status_value_is_bounded_and_retried(self) -> None:
        """Malformed external status JSON must not escape as AttributeError."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_driver_status_protocol")
        wait_for_driver = namespace["_wait_for_driver"]
        globals_ = wait_for_driver.__globals__
        monotonic_values = iter((0.0, 0.0, 21.0))
        status_calls = 0

        def malformed_status(
            _driver_port: int,
            method: str,
            path: str,
            _payload=None,
            *,
            timeout: float = 5.0,
        ) -> dict[str, object]:
            nonlocal status_calls
            status_calls += 1
            self.assertEqual(method, "GET")
            self.assertEqual(path, "/status")
            self.assertGreater(timeout, 0)
            return {"value": []}

        with (
            unittest.mock.patch.dict(globals_, {"_json_request": malformed_status}),
            unittest.mock.patch.object(
                globals_["time"], "monotonic", side_effect=lambda: next(monotonic_values)
            ),
            unittest.mock.patch.object(globals_["time"], "sleep", return_value=None),
        ):
            with self.assertRaisesRegex(
                RuntimeError,
                r"ChromeDriver did not become ready \(status_protocol_error\)",
            ):
                wait_for_driver(43123)

        self.assertEqual(status_calls, 1)


if __name__ == "__main__":
    unittest.main()
