"""Repository contracts for Browser Session presentation authority."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class BrowserSessionLifecycleContractTests(unittest.TestCase):
    """Keep presentation mutation authority in an explicit Browser Session domain."""

    def test_browser_session_is_an_independent_workspace_boundary(self) -> None:
        """Browser Session authority must not be hidden in a driver adapter."""

        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn(
            "crates/originweave-browser-session",
            workspace["workspace"]["members"],
        )
        self.assertTrue(
            (ROOT / "crates/originweave-browser-session/src/lib.rs").is_file()
        )


if __name__ == "__main__":
    unittest.main()
