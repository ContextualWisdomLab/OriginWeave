"""Regression contract for the Linux /proc primary-source citation identity."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
REFERENCE_PREFIX = "Linux Kernel Documentation. (n.d.). *The /proc filesystem*. Retrieved "
REFERENCE_URL = "https://www.kernel.org/doc/html/latest/filesystems/proc.html"
REFERENCE_PATHS = (
    ROOT / "CHANGELOG.md",
    ROOT / "docs" / "adr" / "0105-resource-governor-priority.md",
    ROOT / "docs" / "doctoring.md",
)


class LinuxProcReferenceConsistencyTests(unittest.TestCase):
    """Keep the same reviewed primary-source citation identical across canonical docs."""

    def test_linux_proc_reference_is_identical_across_canonical_docs(self) -> None:
        """One source must not acquire contradictory retrieval-date identities."""
        references: list[str] = []
        for path in REFERENCE_PATHS:
            matches = [
                line
                for line in path.read_text(encoding="utf-8").splitlines()
                if line.startswith(REFERENCE_PREFIX) and REFERENCE_URL in line
            ]
            self.assertEqual(matches.__len__(), 1, path.as_posix())
            references.append(matches[0])

        self.assertEqual(len(set(references)), 1, references)


if __name__ == "__main__":
    unittest.main()
