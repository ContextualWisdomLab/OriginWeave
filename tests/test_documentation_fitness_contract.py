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
        r"(?im)^-\s+(?:\*\*Status:\*\*|\*\*Status\*\*:|Status:)\s*(\w+)(?:\s+.+)?$",
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
    """Keep architecture discovery and ADR lifecycle metadata coherent."""

    def test_documentation_index_links_fitness_assessment(self) -> None:
        """The semantic fitness audit must remain discoverable from the docs index."""
        index = (DOCS_ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("[Documentation fitness assessment](DOCUMENTATION_FITNESS.md)", index)
        self.assertTrue((DOCS_ROOT / "DOCUMENTATION_FITNESS.md").is_file())

    def test_documentation_fitness_distinguishes_design_from_protected_main(self) -> None:
        """A broad design pack must not be mislabeled as code-current closure."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        self.assertIn("DESIGN-SUFFICIENT", assessment)
        self.assertIn("PROTECTED-MAIN-PARTIAL", assessment)
        self.assertIn("File existence alone is never sufficient", assessment)
        self.assertIn("Historical HTTP PR", assessment)
        self.assertIn("MV3 compatibility evidence", assessment)
        self.assertIn("Browser authority", assessment)

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

    def test_fitness_audit_tracks_current_replacement_and_buyer_gap_lanes(self) -> None:
        """The dated audit must identify the current implementation lanes it evaluated."""
        assessment = (DOCS_ROOT / "DOCUMENTATION_FITNESS.md").read_text(encoding="utf-8")
        for marker in ("PR #37", "PR #40", "Issue #27", "issue #28"):
            with self.subTest(marker=marker):
                self.assertIn(marker, assessment)

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
        self.assertIn("Resource-pressure state/sequence — already present", assessment)
        self.assertIn("Hourly autonomous-development authority flow — already present", assessment)
        self.assertIn("## 9. Resource-pressure and fallback flow", uml_index)
        self.assertIn("## 10. Hourly product-development gate-to-model flow", uml_index)


if __name__ == "__main__":
    unittest.main()
