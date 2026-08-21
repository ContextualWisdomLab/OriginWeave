"""Coverage-shape contract for the bounded BiDi locateNodes exchange boundary.

The production coverage gate is exact across functions, lines, regions, and branches.
Keeping the caller-supplied Pong entropy callback generic monomorphizes the whole
exchange function per closure type, which creates synthetic per-instantiation
coverage holes despite exercising the protocol paths. The callback is therefore a
borrowed trait object at this boundary: its behavior remains stateful and caller
owned without multiplying production coverage regions.
"""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = (
    ROOT
    / "crates"
    / "originweave-network"
    / "src"
    / "webdriver_bidi_locate_nodes_exchange.rs"
)


class WebDriverBiDiLocateNodesCoverageShapeTests(unittest.TestCase):
    """Prevent callback monomorphization from invalidating exact coverage evidence."""

    def test_pong_entropy_callback_is_non_generic_at_exchange_boundary(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "next_pong_key: &mut dyn FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>",
            source,
        )
        self.assertNotIn(
            "next_pong_key: impl FnMut() -> Option<WebDriverBiDiWebSocketMaskKey>",
            source,
        )


if __name__ == "__main__":
    unittest.main()
