"""Coverage-shape contract for the bounded BiDi locateNodes exchange boundary.

The production coverage gate is exact across functions, lines, regions, and branches.
Keeping the caller-supplied Pong entropy callback generic monomorphizes the whole
exchange function per closure type, which creates synthetic per-instantiation
coverage holes despite exercising the protocol paths. The callback is therefore a
borrowed trait object at this boundary: its behavior remains stateful and caller
owned without multiplying production coverage regions.

Likewise, error conversion at the public binding wrapper must use a named production
function rather than an inline closure. The library is linked into both unit and
integration-test harnesses; an inline closure can therefore acquire a second uncovered
instantiation even when the real fail-closed binding path is exercised. Keeping that
conversion named makes exact coverage represent product behavior rather than linker
instantiation shape.
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
    """Prevent monomorphization artifacts from invalidating exact coverage evidence."""

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

    def test_node_binding_error_conversion_is_named_not_inline_closure(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("fn map_node_binding_error(", source)
        self.assertIn(".map_err(map_node_binding_error)?;", source)
        self.assertNotIn(".map_err(|error| {", source)


if __name__ == "__main__":
    unittest.main()
