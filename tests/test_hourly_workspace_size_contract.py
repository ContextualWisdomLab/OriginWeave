"""Regression contract for bounded workspace-file comparison in the hourly loop."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
WORKFLOW = ROOT / ".github/workflows/hourly-product-development.yml"


def _bundle_step(workflow: str) -> str:
    """Return the credential-free bundle-validation workflow step."""

    marker = "      - name: Validate and seal the credential-free change bundle\n"
    _before, separator, remainder = workflow.partition(marker)
    if not separator:
        raise AssertionError("missing credential-free bundle-validation step")
    return remainder.partition("\n      - name: ")[0]


class HourlyWorkspaceSizeContractTests(unittest.TestCase):
    """Prevent model-controlled files from being fully read before size rejection."""

    def test_workspace_files_are_bounded_before_changed_file_byte_comparison(self) -> None:
        """Oversized workspace files must fail before the first full comparison read."""

        bundle = _bundle_step(WORKFLOW.read_text(encoding="utf-8"))
        limit = "MAX_CHANGED_FILE_BYTES = 1_048_576"
        workspace_files = "new_files = files(workspace)"
        iteration = "for name, path in new_files.items()"
        size_probe = "if path.stat().st_size > MAX_CHANGED_FILE_BYTES"
        rejection = 'raise SystemExit(f"oversized workspace files rejected: {oversized}")'
        changed = "changed = sorted("
        full_read = "old_files[name].read_bytes() != new_files[name].read_bytes()"

        for contract in (
            limit,
            workspace_files,
            iteration,
            size_probe,
            rejection,
            changed,
            full_read,
        ):
            with self.subTest(contract=contract):
                self.assertIn(contract, bundle)

        self.assertLess(bundle.index(limit), bundle.index(workspace_files))
        self.assertLess(bundle.index(workspace_files), bundle.index(iteration))
        self.assertLess(bundle.index(iteration), bundle.index(size_probe))
        self.assertLess(bundle.index(size_probe), bundle.index(rejection))
        self.assertLess(bundle.index(rejection), bundle.index(changed))
        self.assertLess(bundle.index(changed), bundle.index(full_read))


if __name__ == "__main__":
    unittest.main()
