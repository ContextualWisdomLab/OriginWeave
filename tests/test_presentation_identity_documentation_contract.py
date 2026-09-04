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

    def test_changelog_records_kernel_without_claiming_chromium_application(self) -> None:
        """The changelog must retain the kernel-versus-browser adapter boundary."""
        changelog = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
        self.assertIn(
            "bounded Rust presentation-identity kernel for explicit browser-visible "
            "profiles and credential-free replay digests",
            changelog,
        )
        self.assertIn(
            "applying those profiles to Chromium and proving page-observed effects "
            "remain separate adapter and browser-E2E work",
            changelog,
        )


if __name__ == "__main__":
    unittest.main()
