from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
DOCS_INDEX = ROOT / "docs" / "README.md"
ADR_INDEX = ROOT / "docs" / "adr" / "README.md"
ADR = ROOT / "docs" / "adr" / "0016-bap-task-lifecycle-authority.md"
WORKSPACE = ROOT / "Cargo.toml"
BAP = ROOT / "crates" / "originweave-bap" / "src" / "lib.rs"


class BapAdrProvenanceContract(unittest.TestCase):
    def test_proposed_adr_tracks_protected_main_implementation_without_false_active_pr_provenance(self) -> None:
        docs_index = DOCS_INDEX.read_text(encoding="utf-8")
        adr_index = ADR_INDEX.read_text(encoding="utf-8")
        adr = ADR.read_text(encoding="utf-8")
        workspace = WORKSPACE.read_text(encoding="utf-8")
        bap = BAP.read_text(encoding="utf-8")

        self.assertIn('"crates/originweave-bap"', workspace)
        self.assertIn("Stable internal Browser Agent Protocol lifecycle contracts", bap)
        self.assertIn("- **Status:** Proposed", adr)

        protected_docs = docs_index.split(
            "### Protected-main baseline proposed decisions", 1
        )[1].split("### Proposed decisions introduced by", 1)[0]
        protected_adr = adr_index.split(
            "### Protected-main baseline proposed decisions", 1
        )[1].split("### Proposed decisions introduced by", 1)[0]
        self.assertIn("ADR 0016", protected_docs)
        self.assertIn("[0016]", protected_adr)

        self.assertNotIn(
            "ADR 0016 is owned by this active BAP lifecycle feature branch",
            docs_index,
        )
        self.assertNotIn(
            "ADR 0016 belongs to the active BAP lifecycle feature branch",
            adr_index,
        )
        self.assertNotIn("removal of the active BAP lifecycle branch", adr)
        self.assertIn("Proposed lifecycle does not erase protected-main implementation evidence", adr)


if __name__ == "__main__":
    unittest.main()
