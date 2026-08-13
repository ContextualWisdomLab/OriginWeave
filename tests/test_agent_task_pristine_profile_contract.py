"""Contract for pristine, credential-free Agent Task browser profile admission."""

from __future__ import annotations

import pathlib
import runpy
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class AgentTaskPristineProfileContractTests(unittest.TestCase):
    """Require explicit isolation proof before the pinned browser task can pass."""

    def _namespace(self, name: str) -> dict[str, object]:
        return runpy.run_path(str(RUNNER), run_name=name)

    def test_runner_exposes_pristine_profile_and_ambient_state_boundaries(self) -> None:
        """Profile admission and ambient-state inspection must be explicit boundaries."""

        namespace = self._namespace("agent_task_pristine_profile_contract")
        self.assertIn("_require_pristine_agent_task_profile", namespace)
        self.assertIn("_probe_agent_task_ambient_state", namespace)

    def test_profile_admission_rejects_preexisting_state(self) -> None:
        """Any pre-existing profile entry must fail closed before Chromium starts."""

        namespace = self._namespace("agent_task_pristine_profile_behavior")
        require_pristine = namespace["_require_pristine_agent_task_profile"]
        with tempfile.TemporaryDirectory(prefix="originweave-pristine-profile-") as profile_dir:
            require_pristine(profile_dir)
            pathlib.Path(profile_dir, "Cookies").write_text("preexisting", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "not pristine"):
                require_pristine(profile_dir)

    def test_ambient_state_probe_rejects_cookies_or_web_storage(self) -> None:
        """Browser-visible cookie or Web Storage state must not be normalized as isolated."""

        namespace = self._namespace("agent_task_ambient_state_behavior")
        probe = namespace["_probe_agent_task_ambient_state"]

        def no_cookies_request(
            _driver_port: int,
            method: str,
            path: str,
            _payload: object | None = None,
            **_kwargs: object,
        ) -> dict[str, object]:
            self.assertEqual(method, "GET")
            self.assertTrue(path.endswith("/cookie"))
            return {"value": []}

        probe.__globals__["_json_request"] = no_cookies_request
        probe.__globals__["_execute"] = lambda *_args, **_kwargs: {
            "localStorageLength": 0,
            "sessionStorageLength": 0,
        }
        self.assertEqual(
            probe(4444, "session-a"),
            {
                "ambient_cookies_absent": True,
                "ambient_web_storage_absent": True,
            },
        )

        def ambient_cookie_request(
            _driver_port: int,
            _method: str,
            _path: str,
            _payload: object | None = None,
            **_kwargs: object,
        ) -> dict[str, object]:
            return {"value": [{"name": "ambient", "value": "redacted-test"}]}

        probe.__globals__["_json_request"] = ambient_cookie_request
        with self.assertRaisesRegex(RuntimeError, "ambient cookies"):
            probe(4444, "session-a")

        probe = self._namespace("agent_task_ambient_storage_behavior")[
            "_probe_agent_task_ambient_state"
        ]
        probe.__globals__["_json_request"] = no_cookies_request
        probe.__globals__["_execute"] = lambda *_args, **_kwargs: {
            "localStorageLength": 1,
            "sessionStorageLength": 0,
        }
        with self.assertRaisesRegex(RuntimeError, "ambient Web Storage"):
            probe(4444, "session-a")

    def test_agent_task_disables_saved_credential_services_and_gates_evidence(self) -> None:
        """The acceptance runner must configure and require credential-free isolation evidence."""

        runner = RUNNER.read_text(encoding="utf-8")
        for expected in (
            '"credentials_enable_service": False',
            '"profile.password_manager_enabled": False',
            '"profile_pristine_before_launch"',
            '"ambient_cookies_absent"',
            '"ambient_web_storage_absent"',
            '"saved_credential_services_disabled"',
            "Agent Task isolation gate failed",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, runner)


if __name__ == "__main__":
    unittest.main()
