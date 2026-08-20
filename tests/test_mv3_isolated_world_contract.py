"""Fail-first contract for real Manifest V3 content-script isolated-world evidence."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "tests" / "fixtures" / "mv3_basic"


class ManifestV3IsolatedWorldContractTests(unittest.TestCase):
    """Require content-script compatibility to prove main/isolated world separation."""

    def test_page_continuously_exposes_its_main_world_sentinel(self) -> None:
        """The page must re-read its own window after the content script runs."""

        page = (FIXTURE / "page.html").read_text(encoding="utf-8")
        for expected in (
            'window.originweaveWorldSentinel = "page"',
            "originweavePageWorld",
            "setInterval",
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, page)

    def test_content_ready_depends_on_isolated_world_separation(self) -> None:
        """The existing content-script gate must fail if page and extension worlds collapse."""

        content = (FIXTURE / "content_script.js").read_text(encoding="utf-8")
        for expected in (
            'window.originweaveWorldSentinel = "extension"',
            "originweavePageWorld",
            'window.originweaveWorldSentinel === "extension"',
            'pageWorldSentinel === "page"',
        ):
            with self.subTest(expected=expected):
                self.assertIn(expected, content)
        self.assertIn("setTimeout", content)
        self.assertIn("originweaveContentScript", content)


if __name__ == "__main__":
    unittest.main()
