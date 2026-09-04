"""Regression tests for exact-head CI path partitioning."""

from __future__ import annotations

import pathlib
import subprocess
import sys
import unittest

from scripts.ci.classify_ci_change_scope import (
    classify_changes,
    classify_paths,
    is_documentation_path,
    parse_nul_name_status,
    parse_nul_paths,
    parse_nul_raw_changes,
    render_outputs,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]
CLASSIFIER = ROOT / "scripts/ci/classify_ci_change_scope.py"


def _raw_record(
    source_mode: str,
    destination_mode: str,
    status: str,
    *paths: str,
) -> bytes:
    """Build one deterministic NUL-framed raw-diff record for focused tests."""

    source_oid = "0" * 40 if source_mode == "000000" else "1" * 40
    destination_oid = "0" * 40 if destination_mode == "000000" else "2" * 40
    metadata = (
        f":{source_mode} {destination_mode} {source_oid} {destination_oid} {status}\0"
    ).encode()
    return metadata + b"\0".join(path.encode() for path in paths) + b"\0"


class CiChangeScopeTests(unittest.TestCase):
    """Keep documentation verification fail-closed without forcing Rust on proven prose heads."""

    def test_documentation_paths_need_mode_evidence_before_lightweight_classification(self) -> None:
        """Paths alone cannot prove that a docs entry is an ordinary prose blob."""

        paths = ("docs/adr/0103-example.md", "README.md", "CHANGELOG.md")
        self.assertTrue(all(is_documentation_path(path) for path in paths))
        self.assertEqual(classify_paths(paths), (False, True))

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

    def test_empty_change_fails_closed_to_rust(self) -> None:
        """Missing changed-file evidence must not be treated as documentation-only."""

        self.assertEqual(classify_paths(()), (False, True))
        self.assertEqual(classify_changes(()), (False, True))

    def test_raw_regular_docs_changes_are_lightweight(self) -> None:
        """Ordinary non-executable blobs on both materialized sides prove the docs lane."""

        data = (
            _raw_record("100644", "100644", "M", "docs/PRD.md")
            + _raw_record("000000", "100644", "A", "README.md")
            + _raw_record("100644", "000000", "D", "docs/old.md")
        )
        changes = parse_nul_raw_changes(data)
        self.assertEqual(classify_changes(changes), (True, False))

    def test_raw_code_change_requires_rust(self) -> None:
        """Mode-aware evidence does not weaken the existing path boundary."""

        data = _raw_record(
            "100644",
            "100644",
            "M",
            "crates/originweave-core/src/lib.rs",
        )
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_code_to_docs_rename_requires_rust(self) -> None:
        """Raw rename records must preserve both the code preimage and docs postimage."""

        data = _raw_record(
            "100644",
            "100644",
            "R100",
            "crates/demo/src/lib.rs",
            "docs/lib.md",
        )
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_docs_to_docs_rename_remains_lightweight(self) -> None:
        """A regular-blob rename wholly inside docs stays in the lightweight lane."""

        data = _raw_record(
            "100644",
            "100644",
            "R095",
            "docs/old.md",
            "docs/new.md",
        )
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (True, False))

    def test_raw_symlink_edit_requires_rust(self) -> None:
        """A modified symlink under docs is not proven prose despite status M and a docs path."""

        data = _raw_record("120000", "120000", "M", "docs/guide.md")
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_gitlink_edit_requires_rust(self) -> None:
        """A gitlink under docs must not be treated as lightweight documentation."""

        data = _raw_record("160000", "160000", "M", "docs/vendor")
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_executable_blob_requires_rust(self) -> None:
        """Executable entries do not belong to the prose-only contract surface."""

        data = _raw_record("100755", "100755", "M", "docs/generate.md")
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_type_change_requires_rust(self) -> None:
        """A regular docs blob converted to a symlink must fail closed to Rust."""

        data = _raw_record("100644", "120000", "T", "docs/guide.md")
        self.assertEqual(classify_changes(parse_nul_raw_changes(data)), (False, True))

    def test_raw_parser_rejects_unterminated_stream(self) -> None:
        """Truncated raw evidence must fail before a scope decision is emitted."""

        with self.assertRaisesRegex(ValueError, "NUL-terminated"):
            parse_nul_raw_changes(
                b":100644 100644 "
                + b"1" * 40
                + b" "
                + b"2" * 40
                + b" M\0docs/PRD.md"
            )

    def test_raw_parser_rejects_invalid_mode(self) -> None:
        """Malformed raw mode metadata cannot be accepted as ordinary prose evidence."""

        with self.assertRaisesRegex(ValueError, "mode"):
            parse_nul_raw_changes(
                _raw_record("10064x", "100644", "M", "docs/PRD.md")
            )

    def test_raw_parser_rejects_invalid_utf8_path(self) -> None:
        """Ambiguous raw pathname bytes fail before classification."""

        with self.assertRaisesRegex(ValueError, "valid UTF-8"):
            parse_nul_raw_changes(
                b":100644 100644 "
                + b"1" * 40
                + b" "
                + b"2" * 40
                + b" M\0docs/ok.md\xff\0"
            )

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

    def test_nul_path_parser_rejects_unterminated_stream(self) -> None:
        """A truncated path stream must not be accepted as complete change evidence."""

        with self.assertRaisesRegex(ValueError, "NUL-terminated"):
            parse_nul_paths(b"docs/PRD.md")

    def test_name_status_parser_preserves_paths_but_is_not_scope_authority(self) -> None:
        """Legacy status evidence remains decodable but never proves ordinary blob modes."""

        data = b"M\0docs/PRD.md\0A\0README.md\0D\0docs/old.md\0"
        paths = parse_nul_name_status(data)
        self.assertEqual(paths, ("docs/PRD.md", "README.md", "docs/old.md"))
        self.assertEqual(classify_paths(paths), (False, True))

    def test_name_status_parser_preserves_rename_preimage(self) -> None:
        """Path preservation remains useful for diagnostics even though modes are absent."""

        data = b"R100\0crates/demo/src/lib.rs\0docs/lib.md\0"
        paths = parse_nul_name_status(data)
        self.assertEqual(paths, ("crates/demo/src/lib.rs", "docs/lib.md"))
        self.assertEqual(classify_paths(paths), (False, True))

    def test_name_status_parser_rejects_truncated_rename(self) -> None:
        """Rename/copy records must include both source and destination paths."""

        with self.assertRaisesRegex(ValueError, "rename/copy"):
            parse_nul_name_status(b"R100\0docs/old.md\0")

    def test_name_status_parser_rejects_unterminated_stream(self) -> None:
        """Missing terminal NUL must fail closed before classification."""

        with self.assertRaisesRegex(ValueError, "NUL-terminated"):
            parse_nul_name_status(b"M\0docs/PRD.md")

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

    def test_cli_emits_lightweight_scope_for_mode_aware_docs_raw_diff(self) -> None:
        """The executable boundary must require raw modes before skipping Rust."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=(
                _raw_record("100644", "100644", "M", "docs/PRD.md")
                + _raw_record("000000", "100644", "A", "README.md")
            ),
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

    def test_cli_fails_closed_to_rust_for_mode_blind_name_status(self) -> None:
        """Status/path-only input cannot prove that a docs entry is an ordinary blob."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=b"M\0docs/PRD.md\0",
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

    def test_cli_fails_closed_on_unterminated_status_stream(self) -> None:
        """A truncated Git stream must not emit a lightweight CI decision."""

        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER)],
            input=b"M\0docs/PRD.md",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(completed.returncode, 2)
        self.assertEqual(completed.stdout, b"")
        self.assertIn(b"CI scope classification failed", completed.stderr)
        self.assertIn(b"NUL-terminated", completed.stderr)

    def test_workflow_keeps_docs_contracts_separate_from_rust(self) -> None:
        """The CI workflow must always run docs contracts and gate Rust jobs by raw scope."""

        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertIn("name: Repository and documentation contracts", workflow)
        self.assertIn("python3 -m unittest discover -s tests -p 'test_*.py'", workflow)
        self.assertIn("needs: [scope, contracts]", workflow)
        self.assertGreaterEqual(
            workflow.count("needs.scope.outputs.rust_required == 'true'"),
            2,
        )
        self.assertIn("git diff --raw -z --no-abbrev", workflow)
        self.assertIn("classify_ci_change_scope.py", workflow)


if __name__ == "__main__":
    unittest.main()
