from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
DOCS_INDEX = ROOT / "docs" / "README.md"
ADR_INDEX = ROOT / "docs" / "adr" / "README.md"
ADR = ROOT / "docs" / "adr" / "0016-bap-task-lifecycle-authority.md"
WORKSPACE = ROOT / "Cargo.toml"
BAP = ROOT / "crates" / "originweave-bap" / "src" / "lib.rs"


class BapAdrProvenanceContract(unittest.TestCase):
    def bounded_section(self, text: str, start: str, end: str) -> str:
        self.assertIn(start, text)
        tail = text.split(start, 1)[1]
        self.assertIn(end, tail)
        return tail.split(end, 1)[0]

    def test_bounded_section_requires_end_marker_after_start_marker(self) -> None:
        with self.assertRaises(AssertionError):
            self.bounded_section(
                "END\nSTART\npayload without a trailing end marker",
                "START",
                "END",
            )

    def test_proposed_adr_tracks_protected_main_implementation_without_false_active_pr_provenance(self) -> None:
        docs_index = DOCS_INDEX.read_text(encoding="utf-8")
        adr_index = ADR_INDEX.read_text(encoding="utf-8")
        adr = ADR.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        bap = BAP.read_text(encoding="utf-8")

        self.assertIn('"crates/originweave-bap"', workspace)
        self.assertIn("pub enum BapTaskState", bap)
        self.assertIn("pub enum BapTaskEvent", bap)
        self.assertIn("- **Status:** Proposed", adr)

        protected_docs = self.bounded_section(
            docs_index,
            "### Protected-main baseline proposed decisions",
            "### Proposed decisions introduced by",
        )
        introduced_docs = self.bounded_section(
            docs_index,
            "### Proposed decisions introduced by",
            "See the [ADR index]",
        )
        protected_adr = self.bounded_section(
            adr_index,
            "### Protected-main baseline proposed decisions",
            "### Proposed decisions introduced by documentation reconciliation",
        )
        introduced_adr = self.bounded_section(
            adr_index,
            "### Proposed decisions introduced by documentation reconciliation",
            "## Index completeness rule",
        )

        self.assertIn("ADR 0016", protected_docs)
        self.assertNotIn("ADR 0016", introduced_docs)
        self.assertIn("[0016]", protected_adr)
        self.assertNotIn("[0016]", introduced_adr)

        context = self.bounded_section(adr, "## Context", "## Decision drivers")
        self.assertIn("merged PR #208", context)
        self.assertIn("protected `main`", context)
        self.assertIn("Proposed", context)


if __name__ == "__main__":
    unittest.main()
