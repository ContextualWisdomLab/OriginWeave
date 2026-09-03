"""Security contract for pinned Manifest V3 browser executable authority."""

from __future__ import annotations

import os
import pathlib
import runpy
import tempfile
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3BinaryAuthorityContractTests(unittest.TestCase):
    """Prevent environment variables from selecting arbitrary executable code."""

    def setUp(self) -> None:
        """Load the production runner without executing its command-line entrypoint."""

        self.namespace = runpy.run_path(str(RUNNER), run_name="mv3_binary_authority")
        self.validate = self.namespace["_pinned_workspace_binary"]

    @staticmethod
    def _make_executable(path: pathlib.Path) -> None:
        """Create one inert executable fixture without ever executing it."""

        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        path.chmod(0o755)

    def test_untrusted_environment_override_is_rejected_before_execution(self) -> None:
        """An existing executable outside the pinned workspace path must fail closed."""

        with tempfile.TemporaryDirectory(prefix="originweave-binary-authority-") as temp_dir:
            root = pathlib.Path(temp_dir)
            expected = root / ".mv3-browser" / "chromedriver-linux64" / "chromedriver"
            attacker = root / "attacker-controlled" / "chromedriver"
            self._make_executable(expected)
            self._make_executable(attacker)

            with unittest.mock.patch.dict(
                os.environ,
                {"CHROMEDRIVER_BIN": str(attacker)},
                clear=False,
            ):
                with self.assertRaisesRegex(SystemExit, "pinned workspace executable"):
                    self.validate(
                        "CHROMEDRIVER_BIN",
                        pathlib.PurePosixPath(
                            ".mv3-browser/chromedriver-linux64/chromedriver"
                        ),
                        "ChromeDriver",
                        root=root,
                    )

    def test_exact_pinned_workspace_executable_is_accepted(self) -> None:
        """The exact executable provisioned by the pinned workflow remains usable."""

        with tempfile.TemporaryDirectory(prefix="originweave-binary-authority-") as temp_dir:
            root = pathlib.Path(temp_dir)
            expected = root / ".mv3-browser" / "chrome-linux64" / "chrome"
            self._make_executable(expected)

            with unittest.mock.patch.dict(
                os.environ,
                {"CHROME_BIN": str(expected)},
                clear=False,
            ):
                actual = self.validate(
                    "CHROME_BIN",
                    pathlib.PurePosixPath(".mv3-browser/chrome-linux64/chrome"),
                    "Chrome for Testing",
                    root=root,
                )

            self.assertEqual(actual, expected)

    def test_symlink_at_pinned_executable_path_is_rejected(self) -> None:
        """A matching pathname must not authorize a symlink to foreign executable code."""

        with tempfile.TemporaryDirectory(prefix="originweave-binary-authority-") as temp_dir:
            root = pathlib.Path(temp_dir)
            expected = root / ".mv3-browser" / "chromedriver-linux64" / "chromedriver"
            attacker = root / "attacker-controlled" / "chromedriver"
            self._make_executable(attacker)
            expected.parent.mkdir(parents=True, exist_ok=True)
            expected.symlink_to(attacker)

            with unittest.mock.patch.dict(
                os.environ,
                {"CHROMEDRIVER_BIN": str(expected)},
                clear=False,
            ):
                with self.assertRaisesRegex(SystemExit, "symlink"):
                    self.validate(
                        "CHROMEDRIVER_BIN",
                        pathlib.PurePosixPath(
                            ".mv3-browser/chromedriver-linux64/chromedriver"
                        ),
                        "ChromeDriver",
                        root=root,
                    )


if __name__ == "__main__":
    unittest.main()
