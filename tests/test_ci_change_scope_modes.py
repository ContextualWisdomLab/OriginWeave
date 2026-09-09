"""Security regressions for mode-aware CI change-scope evidence."""

from __future__ import annotations

import unittest

from scripts.ci.classify_ci_change_scope import classify_paths, parse_nul_name_status


class CiChangeScopeModeTests(unittest.TestCase):
    """Do not treat path/status-only Git evidence as proof of a prose-only blob."""

    def test_mode_blind_modified_docs_path_fails_closed(self) -> None:
        """A modified docs path could be a symlink or gitlink when modes are absent."""

        paths = parse_nul_name_status(b"M\0docs/guide.md\0")
        self.assertEqual(paths, ("docs/guide.md",))
        self.assertEqual(classify_paths(paths), (False, True))

    def test_type_changed_docs_path_fails_closed(self) -> None:
        """A regular-file-to-symlink type change must never enter the lightweight lane."""

        paths = parse_nul_name_status(b"T\0docs/guide.md\0")
        self.assertEqual(paths, ("docs/guide.md",))
        self.assertEqual(classify_paths(paths), (False, True))


if __name__ == "__main__":
    unittest.main()
