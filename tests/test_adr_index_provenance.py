"""Regression contracts for active-PR ADR provenance in the canonical index."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class AdrIndexProvenanceTests(unittest.TestCase):
    """Prevent branch-only ADRs from being presented as protected-main baseline truth."""

    def test_presentation_identity_adr_is_branch_only_until_integration(self) -> None:
        """ADR 0110 must stay in the branch-only provenance subsection on this PR."""
        text = (ROOT / "docs/adr/README.md").read_text(encoding="utf-8")
        baseline = text.split("### Protected-main baseline proposed decisions", 1)[1].split(
            "### Proposed decisions introduced by documentation reconciliation", 1
        )[0]
        branch_only = text.split(
            "### Proposed decisions introduced by documentation reconciliation", 1
        )[1].split("## Index completeness rule", 1)[0]
        adr = "[0110](0110-privacy-preserving-presentation-identity.md)"
        adr_stealth = "[0111](0111-bounded-stealth-normalization-surfaces.md)"
        adr_ua_hints = "[0112](0112-bounded-user-agent-client-hints.md)"
        adr_coherence = "[0113](0113-cross-surface-platform-coherence.md)"

        self.assertNotIn(adr, baseline)
        self.assertIn(adr, branch_only)
        self.assertNotIn(adr_stealth, baseline)
        self.assertIn(adr_stealth, branch_only)
        self.assertNotIn(adr_ua_hints, baseline)
        self.assertIn(adr_ua_hints, branch_only)
        self.assertNotIn(adr_coherence, baseline)
        self.assertIn(adr_coherence, branch_only)


if __name__ == "__main__":
    unittest.main()
