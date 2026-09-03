"""Documentation contract for the presentation-identity bounded context."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class PresentationIdentityDocumentationContractTests(unittest.TestCase):
    """Keep branch-local presentation maturity separate from Chromium shipment."""

    def test_presentation_identity_status_separates_kernel_evidence_from_adapter(self) -> None:
        """The TRD must not promote the planned Chromium adapter to shipped behavior."""
        trd = (ROOT / "docs/TRD.md").read_text(encoding="utf-8")
        section = trd.split("### 6.8 Presentation identity", 1)[1].split(
            "## 7. Observation architecture", 1
        )[0]
        self.assertIn("**Active-PR kernel evidence; Chromium adapter planned.**", section)
        self.assertNotIn("**Proposed.**", section)
        self.assertNotIn("**Implemented kernel contract; adapter planned.**", section)


if __name__ == "__main__":
    unittest.main()
