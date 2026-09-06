"""Contracts for the 2026-09-06 volatile delivery checkpoint."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHECKPOINT = ROOT / "docs" / "evidence" / "2026-09-06-delivery-checkpoint.md"


class CurrentDeliveryCheckpointContractTests(unittest.TestCase):
    """Keep source, dependency, hosted and visual evidence boundaries explicit."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.checkpoint = CHECKPOINT.read_text(encoding="utf-8")

    def test_current_source_heads_and_parent_boundary_are_exact(self) -> None:
        for marker in (
            "`b4702cd503fa3f721e0d1f44b355563753dac0a2`",
            "`35555d0f8491d4ee95c2e61d1a2aaa2c02a0635c`",
            "`7fb93e77b800f27187a5c02333298cc31a025bd6`",
            "latest-parent acceptance is not claimed",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.checkpoint)

    def test_red_green_and_hosted_boundaries_are_not_conflated(self) -> None:
        for marker in (
            "RED-only checkpoint",
            "no production repair or GREEN is claimed for #264",
            "1293/1293 functions",
            "13567/13567 lines",
            "17273/17273 regions",
            "1440/1440 branches",
            "CI `34032661775`",
            "MV3 `34032661781`",
            "queued",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.checkpoint)

    def test_visual_and_review_boundaries_remain_open(self) -> None:
        for marker in (
            "Mac remains locked",
            "no stale screenshot substitution",
            "#147",
            "`PRRT_kwDOTulPlM6coZwc`",
            "valid unresolved finding",
        ):
            with self.subTest(marker=marker):
                self.assertIn(marker, self.checkpoint)


if __name__ == "__main__":
    unittest.main()
