"""Coverage-shape contract for the bounded BiDi locateNodes exchange boundary.

The production coverage gate is exact across functions, lines, regions, and branches.
Keeping the caller-supplied Pong entropy callback generic monomorphizes the whole
exchange function per closure type, which creates synthetic per-instantiation
coverage holes despite exercising the protocol paths. The callback is therefore a
borrowed trait object at this boundary: its behavior remains stateful and caller
owned without multiplying production coverage regions.

The final node-binding error conversion must also avoid generating a separate closure
or helper function. The library is linked into both unit and integration-test
harnesses, and either form can acquire an uncovered duplicate instantiation even when
the real fail-closed binding path is exercised. A direct match inside the already
exercised public wrapper keeps the typed error conversion at the causal boundary
without adding another production symbol for coverage to duplicate.
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

    def test_node_binding_error_conversion_stays_inside_exercised_wrapper(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn("let handles = match result.bind_current_nodes(", source)
        self.assertNotIn("fn map_node_binding_error(", source)
        self.assertNotIn(".map_err(map_node_binding_error)?;", source)
        self.assertNotIn(".map_err(|error| {", source)


if __name__ == "__main__":
    unittest.main()
