"""Regression contracts for the authoritative OriginWeave documentation graph."""

from pathlib import Path
import re
import unittest


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DOCS_ROOT = REPOSITORY_ROOT / "docs"
ADR_ROOT = DOCS_ROOT / "adr"
UML_ROOT = DOCS_ROOT / "uml"

ADR_STATUSES = {"Proposed", "Accepted", "Superseded", "Deprecated", "Rejected"}


def _adr_files() -> set[str]:
    """Return every numbered ADR Markdown file currently tracked by the repository."""
    return {
        path.name
        for path in ADR_ROOT.glob("[0-9][0-9][0-9][0-9]-*.md")
        if path.is_file()
    }


def _adr_file_status(path: Path) -> str:
    """Read one ADR's explicit lifecycle status from its metadata header."""
    text = path.read_text(encoding="utf-8")
    match = re.search(
        r"(?im)^-\s+(?:\*\*Status:\*\*|\*\*Status\*\*:|Status:)\s*(\w+)(?:[;\s].*)?$",
        text,
    )
    if match is None:
        raise AssertionError(f"ADR has no parseable status: {path.name}")
    status = match.group(1)
    if status not in ADR_STATUSES:
        raise AssertionError(f"ADR has unsupported status {status!r}: {path.name}")
    return status


def _insert_unique(mapping: dict[str, str], path: str, status: str, source: str) -> None:
    """Insert one index target while rejecting duplicate or conflicting entries."""
    if path in mapping:
        raise AssertionError(f"duplicate ADR index target {path!r} in {source}")
    mapping[path] = status


def _parse_docs_index(text: str) -> dict[str, str]:
    """Parse ADR links from the product documentation index by lifecycle section."""
    mapping: dict[str, str] = {}
    current_status: str | None = None
    for line in text.splitlines():
        if line.startswith("## "):
            current_status = next(
                (status for status in ADR_STATUSES if line.startswith(f"## {status}")),
                None,
            )
            continue
        target = re.search(r"\(adr/(\d{4}[-\w]*\.md)\)", line)
        if target is not None:
            if current_status is None:
                raise AssertionError(
                    f"ADR link {target.group(1)!r} is outside a lifecycle-status section"
                )
            _insert_unique(mapping, target.group(1), current_status, "docs/README.md")
    return mapping


def _parse_adr_index(text: str) -> dict[str, str]:
    """Parse the dedicated ADR table into an exact target-to-status mapping."""
    mapping: dict[str, str] = {}
    pattern = re.compile(
        r"^\|\s*\[\d{4}\]\((\d{4}[-\w]*\.md)\)\s*\|[^|]*\|\s*"
        r"(Proposed|Accepted|Superseded|Deprecated|Rejected)\s*\|",
        re.MULTILINE,
    )
    for path, status in pattern.findall(text):
        _insert_unique(mapping, path, status, "docs/adr/README.md")
    return mapping


class DocumentationFitnessContractTests(unittest.TestCase):
    """Keep architecture discovery and implementation-maturity metadata coherent."""

    def test_documentation_index_links_fitness_assessment(self) -> None:
        """The semantic fitness audit must remain discoverable from the docs index."""
        index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("[Documentation fitness assessment](DOCUMENTATION_FITNESS.md)", index)
        self.assertTrue((DOCS_ROOT / "DOCUMENTATION_FITNESS.md").is_file())

    def test_documentation_fitness_distinguishes_design_from_protected_main(self) -> None:
        """A broad design pack must not be mislabeled as protected-main closure."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        self.assertIn("DESIGN-SUFFICIENT", assessment)
        self.assertIn("PROTECTED-MAIN-PARTIAL", assessment)
        self.assertIn("File existence alone is never sufficient", assessment)
        self.assertIn("HTTP lineage", assessment)
        self.assertIn("Manifest V3 compatibility", assessment)
        self.assertIn("Browser identifier authority", assessment)
        self.assertIn("integration before any of these branch repairs become protected-main truth", assessment)

    def test_every_adr_is_indexed_once_with_its_file_status(self) -> None:
        """Both canonical indexes must exactly cover ADR files and their lifecycle status."""
        actual_files = _adr_files()
        file_status = {
            path: _adr_file_status(ADR_ROOT / path)
            for path in sorted(actual_files)
        }
        docs_index = _parse_docs_index((DOCS_ROOT / "README.md").read_text(encoding="utf-8"))
        adr_index = _parse_adr_index((ADR_ROOT / "README.md").read_text(encoding="utf-8"))

        self.assertEqual(set(docs_index), actual_files)
        self.assertEqual(set(adr_index), actual_files)
        self.assertEqual(docs_index, file_status)
        self.assertEqual(adr_index, file_status)

    def test_adr_index_does_not_use_change_local_language_as_timeless_authority(self) -> None:
        """The protected-main ADR index must not describe its ADRs as only `this change`."""
        adr_index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertNotIn("Proposed target-architecture decisions in this change", adr_index)
        self.assertIn("Index completeness rule", adr_index)

    def test_proposed_adr_provenance_does_not_promote_branch_to_protected_main(self) -> None:
        """Branch-only ADR presence must remain distinct from lifecycle and protected-main truth."""
        docs_index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        adr_index = (ADR_ROOT / "README.md").read_text(encoding="utf-8")

        for text in (docs_index, adr_index):
            with self.subTest(index="docs" if text is docs_index else "adr"):
                self.assertIn("## Proposed architecture decisions", text)
                self.assertIn("Protected-main baseline proposed decisions", text)
                self.assertNotIn("## Proposed decisions retained on protected main", text)

        docs_branch = docs_index.split(
            "### Proposed decisions introduced by this documentation reconciliation", 1
        )[1].split("\n## ", 1)[0]
        adr_branch = adr_index.split(
            "### Proposed decisions introduced by documentation reconciliation", 1
        )[1].split("\n## ", 1)[0]
        for adr_path in (
            "0013-manifest-v3-extension-authority.md",
            "0014-architecture-decision-governance.md",
        ):
            with self.subTest(adr=adr_path):
                self.assertIn(adr_path, docs_branch)
                self.assertIn(adr_path, adr_branch)
        self.assertIn("exist only on this documentation branch until it integrates", adr_index)

    def test_current_replacement_lanes_are_not_promoted_to_protected_main(self) -> None:
        """Canonical docs must distinguish active implementation from shipped implementation."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        traceability = (DOCS_ROOT / "traceability" / "README.md").read_text(encoding="utf-8")
        for marker in ("PR #37", "PR #40", "PR #43", "issue #10", "issue #27", "issue #28"):
            with self.subTest(marker=marker):
                self.assertTrue(marker in assessment or marker in traceability)
        self.assertIn("IMPLEMENTED_ON_ACTIVE_PR", traceability)
        self.assertIn("IMPLEMENTED_ON_PROTECTED_MAIN", traceability)
        self.assertIn("Active-PR behavior is never protected-main truth", traceability)

    def test_prd_does_not_restore_superseded_active_pr_claims(self) -> None:
        """Historical feature branches must not reappear as the current implementation lane."""
        prd = (DOCS_ROOT / "PRD.md").read_text(encoding="utf-8")
        self.assertNotIn("Active PR #11", prd)
        self.assertNotIn("Active replacement PR #33", prd)
        self.assertIn("active replacement PR #37", prd)
        self.assertIn("Protected-main purpose-bound sensitive-data policy kernel", prd)
        self.assertIn("active PR #43 adds", prd)

    def test_trd_uses_single_status_with_separate_active_pr_evidence(self) -> None:
        """Implementation status must not be collapsed with active-development annotations."""
        trd = (DOCS_ROOT / "TRD.md").read_text(encoding="utf-8")
        self.assertNotIn("**Planned / active development**", trd)
        self.assertNotIn("**Accepted architecture; active development.**", trd)
        self.assertIn("Protected-main status", trd)
        self.assertIn("Active/non-shipped evidence", trd)
        self.assertIn("Active replacement PR #37", trd)
        self.assertIn("purpose-bound sensitive-data authority", trd)

    def test_extension_authority_uml_separates_compatibility_from_agent_authority(self) -> None:
        """A Chrome permission must never be documented as an Agent capability."""
        diagram = (UML_ROOT / "extension-authority.md").read_text(encoding="utf-8")
        self.assertIn("A Chromium extension permission is not an OriginWeave Agent capability", diagram)
        self.assertIn("Compatibility evidence is separate from authority evidence", diagram)
        self.assertIn("sequenceDiagram", diagram)
        self.assertIn("OriginWeave Extension Grant Policy", diagram)
        self.assertIn("cannot approve", diagram)
        self.assertIn("cannot resolve", diagram)

    def test_fitness_audit_does_not_duplicate_existing_resource_or_hourly_uml(self) -> None:
        """The audit must recognize existing product-wide resource and automation diagrams."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        uml_index = (UML_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("resource-pressure/GPU fallback", assessment)
        self.assertIn("hourly automation flows", assessment)
        self.assertIn("## 9. Resource-pressure and fallback flow", uml_index)
        self.assertIn("## 10. Hourly product-development gate-to-model flow", uml_index)


if __name__ == "__main__":
    unittest.main()
