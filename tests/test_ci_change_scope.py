"""Regression tests for exact-head CI path partitioning."""

from __future__ import annotations

import pathlib
import subprocess
import sys
import unittest

from scripts.ci.classify_ci_change_scope import (
    classify_paths,
    is_documentation_path,
    parse_nul_name_status,
    parse_nul_paths,
    render_outputs,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]
CLASSIFIER = ROOT / "scripts/ci/classify_ci_change_scope.py"


class CiChangeScopeTests(unittest.TestCase):
    """Keep documentation verification fail-closed without forcing Rust on prose-only heads."""

    def test_documentation_paths_are_lightweight(self) -> None:
        """Docs and root Markdown files should retain the lightweight contract lane."""

        paths = ("docs/adr/0103-example.md", "README.md", "CHANGELOG.md")
        self.assertTrue(all(is_documentation_path(path) for path in paths))
        self.assertEqual(classify_paths(paths), (True, False))

    def test_nested_markdown_outside_docs_requires_rust(self) -> None:
        """A Markdown suffix alone must not widen the reviewed prose-only surface."""

        path = "crates/originweave-core/README.md"
        self.assertFalse(is_documentation_path(path))
        self.assertEqual(classify_paths((path,)), (False, True))

    def test_non_documentation_path_requires_rust(self) -> None:
        """Any code, workflow, test, config, or other non-prose path keeps Rust-heavy CI."""

        for path in (
            "Cargo.toml",
            "crates/originweave-core/src/lib.rs",
            ".github/workflows/ci.yml",
            "tests/test_repository_contract.py",
            "scripts/ci/verify_coverage.py",
        ):
            with self.subTest(path=path):
                self.assertFalse(is_documentation_path(path))
                self.assertEqual(classify_paths((path,)), (False, True))

    def test_mixed_change_requires_rust(self) -> None:
        """A prose edit cannot hide a code-bearing delta from the Rust lanes."""

        self.assertEqual(
            classify_paths(("docs/PRD.md", "crates/originweave-policy/src/lib.rs")),
            (False, True),
        )

    def test_empty_change_fails_closed_to_rust(self) -> None:
        """Missing changed-path evidence must not be treated as documentation-only."""

        self.assertEqual(classify_paths(()), (False, True))

    def test_nul_path_parser_preserves_spaces_and_unicode(self) -> None:
        """Git's NUL framing must preserve valid repository names without shell splitting."""

        data = "docs/운영 문서.md\0README.md\0".encode()
        self.assertEqual(parse_nul_paths(data), ("docs/운영 문서.md", "README.md"))

    def test_nul_path_parser_rejects_invalid_utf8(self) -> None:
        """Ambiguous path bytes must fail before a CI scope decision is emitted."""

        with self.assertRaisesRegex(ValueError, "valid UTF-8"):
            parse_nul_paths(b"docs/ok.md\0\xff\0")

    def test_nul_path_parser_rejects_absolute_path(self) -> None:
        """Classification accepts repository-relative paths only."""

        with self.assertRaisesRegex(ValueError, "repository-relative"):
            parse_nul_paths(b"/docs/PRD.md\0")

    def test_nul_path_parser_rejects_parent_traversal(self) -> None:
        """Classification accepts repository paths, never parent-relative spellings."""

        with self.assertRaisesRegex(ValueError, "parent traversal"):
            parse_nul_paths(b"docs/../Cargo.toml\0")

    def test_name_status_parser_preserves_docs_only_changes(self) -> None:
        """Status framing must not force Rust when every affected path is prose-only."""

        data = b"M\0docs/PRD.md\0A\0README.md\0D\0docs/old.md\0"
        paths = parse_nul_name_status(data)
        self.assertEqual(paths, ("docs/PRD.md", "README.md", "docs/old.md"))
        self.assertEqual(classify_paths(paths), (True, False))

    def test_code_to_docs_rename_requires_rust(self) -> None:
        """A post-image docs path must not hide a code-bearing rename preimage."""

        data = b"R100\0crates/demo/src/lib.rs\0docs/lib.md\0"
        paths = parse_nul_name_status(data)
        self.assertEqual(paths, ("crates/demo/src/lib.rs", "docs/lib.md"))
        self.assertEqual(classify_paths(paths), (False, True))

    def test_docs_to_docs_rename_remains_lightweight(self) -> None:
        """A rename whose preimage and postimage are both docs stays in the lightweight lane."""

        data = b"R095\0docs/old.md\0docs/new.md\0"
        paths = parse_nul_name_status(data)
        self.assertEqual(paths, ("docs/old.md", "docs/new.md"))
        self.assertEqual(classify_paths(paths), (True, False))

    def test_name_status_parser_rejects_truncated_rename(self) -> None:
        """Rename/copy records must include both source and destination paths."""

        with self.assertRaisesRegex(ValueError, "rename/copy"):
            parse_nul_name_status(b"R100\0docs/old.md\0")

    def test_name_status_parser_rejects_unknown_status(self) -> None:
        """Unknown Git status records fail closed instead of guessing path cardinality."""

        with self.assertRaisesRegex(ValueError, "status"):
            parse_nul_name_status(b"Q\0docs/PRD.md\0")

    def test_outputs_are_exact_booleans(self) -> None:
        """Workflow outputs stay deterministic for job-level conditions."""

        self.assertEqual(
            render_outputs(True, False),
            "documentation_only=true\nrust_required=false\n",
        )
        self.assertEqual(
            render_outputs(False, True),
            "documentation_only=false\nrust_required=true\n",
        )

    def test_cli_emits_lightweight_scope_for_nul_delimited_docs_status(self) -> None:
        """The executable boundary must preserve Git's status-aware NUL framing."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=b"M\0docs/PRD.md\0A\0README.md\0",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr.decode())
        self.assertEqual(
            completed.stdout.decode(),
            "documentation_only=true\nrust_required=false\n",
        )
        self.assertEqual(completed.stderr, b"")

    def test_cli_requires_rust_for_code_to_docs_rename(self) -> None:
        """The executable boundary must classify both sides of a rename."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=b"R100\0crates/demo/src/lib.rs\0docs/lib.md\0",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr.decode())
        self.assertEqual(
            completed.stdout.decode(),
            "documentation_only=false\nrust_required=true\n",
        )
        self.assertEqual(completed.stderr, b"")

    def test_cli_fails_closed_on_invalid_path_bytes(self) -> None:
        """Malformed Git path evidence must return non-zero without emitting scope outputs."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=b"M\0docs/PRD.md\0M\0\xff\0",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 2)
        self.assertEqual(completed.stdout, b"")
        self.assertIn(b"CI scope classification failed", completed.stderr)
        self.assertIn(b"valid UTF-8", completed.stderr)

    def test_workflow_keeps_docs_contracts_separate_from_rust(self) -> None:
        """The CI workflow must always run docs contracts and gate Rust jobs by scope."""

        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertIn("name: Repository and documentation contracts", workflow)
        self.assertIn("python3 -m unittest discover -s tests -p 'test_*.py'", workflow)
        self.assertIn("needs: [scope, contracts]", workflow)
        self.assertGreaterEqual(
            workflow.count("needs.scope.outputs.rust_required == 'true'"),
            2,
        )
        self.assertIn("git diff --name-status -z", workflow)
        self.assertIn("classify_ci_change_scope.py", workflow)


if __name__ == "__main__":
    unittest.main()
