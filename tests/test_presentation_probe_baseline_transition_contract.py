"""Contract for causal presentation-probe baseline evidence."""

from __future__ import annotations

import pathlib
import runpy
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class PresentationProbeBaselineTransitionContractTests(unittest.TestCase):
    """Require every claimed presentation surface to transition from ambient state."""

    def test_baseline_rejects_any_surface_already_matching_the_fixed_target(self) -> None:
        """A matching ambient surface cannot prove that its override took effect."""

        namespace = runpy.run_path(str(RUNNER), run_name="presentation_baseline_contract")
        validate = namespace["_validate_presentation_probe_baseline"]
        target = namespace["_presentation_probe_target"]()
        ambient = {
            "viewport": "800x600",
            "device_pixel_ratio": "1",
            "timezone": "UTC",
        }

        validate(ambient)
        for key, target_value in target.items():
            with self.subTest(surface=key), self.assertRaisesRegex(
                RuntimeError,
                r"^presentation probe baseline already matched target$",
            ):
                matching = dict(ambient)
                matching[key] = target_value
                validate(matching)


if __name__ == "__main__":
    unittest.main()
