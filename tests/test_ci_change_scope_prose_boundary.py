"""Regression tests for the reviewed prose-only CI surface."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import (
    classify_changes,
    is_documentation_path,
    parse_nul_raw_changes,
)


def _raw_record(path: str) -> bytes:
    """Build one regular-blob modification for an exact repository path."""

    return (
        b":100644 100644 1111111 2222222 M\0"
        + path.encode("utf-8")
        + b"\0"
    )


class CiChangeScopeProseBoundaryTests(unittest.TestCase):
    """Do not let the docs directory become a generic code-bearing skip surface."""

    def test_non_prose_regular_blobs_under_docs_require_rust(self) -> None:
        """A docs/ prefix alone must not authorize skipping code-oriented gates."""

        for path in (
            "docs/browser_fixture.js",
            "docs/generate.py",
            "docs/runtime.rs",
            "docs/package.json",
        ):
            with self.subTest(path=path):
                self.assertFalse(is_documentation_path(path))
                self.assertEqual(
                    classify_changes(parse_nul_raw_changes(_raw_record(path))),
                    (False, True),
                )

    def test_agent_instruction_control_plane_requires_rust(self) -> None:
        """Contributor authority documents must never enter the lightweight prose lane."""

        for path in (
            "AGENTS.md",
            "CLAUDE.md",
            "docs/AGENTS.md",
            "docs/CLAUDE.md",
            "docs/doctoring/AGENTS.md",
            "docs/doctoring/CLAUDE.md",
        ):
            with self.subTest(path=path):
                self.assertFalse(is_documentation_path(path))
                self.assertEqual(
                    classify_changes(parse_nul_raw_changes(_raw_record(path))),
                    (False, True),
                )

    def test_markdown_under_docs_remains_lightweight(self) -> None:
        """The existing reviewed Markdown documentation surface remains eligible."""

        path = "docs/doctoring/browser-policy.md"
        self.assertTrue(is_documentation_path(path))
        self.assertEqual(
            classify_changes(parse_nul_raw_changes(_raw_record(path))),
            (True, False),
        )

    def test_noncanonical_docs_paths_fail_before_scope_classification(self) -> None:
        """Tree-diff evidence must use Git's canonical repository-relative path spelling."""

        for path in (
            "docs//browser-policy.md",
            "docs/./browser-policy.md",
            "docs/doctoring//browser-policy.md",
        ):
            with self.subTest(path=path):
                with self.assertRaisesRegex(ValueError, "canonical repository path"):
                    parse_nul_raw_changes(_raw_record(path))


if __name__ == "__main__":
    unittest.main()
