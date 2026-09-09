"""Protocol contract for bounded WebDriver session cleanup."""

from __future__ import annotations

import inspect
import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class WebDriverSessionCleanupContractTests(unittest.TestCase):
    """Keep Delete Session aligned with the body-free WebDriver command."""

    def test_delete_session_omits_request_body(self) -> None:
        """DELETE /session/{id} must not manufacture an empty JSON payload."""

        namespace = runpy.run_path(str(RUNNER), run_name="webdriver_cleanup_body_contract")
        cleanup = namespace["_cleanup_browser_session"]
        calls: list[tuple[tuple[object, ...], dict[str, object]]] = []

        def record_request(*args: object, **kwargs: object) -> dict[str, object]:
            calls.append((args, kwargs))
            return {"value": None}

        cleanup.__globals__["_json_request"] = record_request
        cleanup(9515, "session-1")

        self.assertEqual(
            calls,
            [((9515, "DELETE", "/session/session-1"), {})],
        )

    def test_agent_task_cleanup_uses_shared_production_path_only(self) -> None:
        """The executed Agent Task finalizer must use the shared cleanup path directly."""

        namespace = runpy.run_path(str(RUNNER), run_name="webdriver_cleanup_path_contract")
        browser_pass_source = inspect.getsource(namespace["_run_agent_task_browser_pass"])
        self.assertIn("_cleanup_browser_session_preserving_primary(", browser_pass_source)
        self.assertNotIn("_cleanup_agent_task_browser_session(", browser_pass_source)
        self.assertIn("_cleanup_browser_session_preserving_primary", namespace)


if __name__ == "__main__":
    unittest.main()
