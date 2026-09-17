import pathlib
import runpy
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
CANONICAL_CONTRACT = ROOT / "tests/test_browser_session_trusted_adapter_boundary.py"


def _canonical_workspace_member_manifests(root: pathlib.Path) -> list[pathlib.Path]:
    """Load the single-writer workspace-member scanner from the canonical security contract."""
    namespace = runpy.run_path(str(CANONICAL_CONTRACT))
    return namespace["_workspace_member_manifests"](root)


class BrowserSessionWorkspaceMemberSymlinkContractTests(unittest.TestCase):
    """Keep symlinked workspace members inside the repository review root."""

    def test_symlinked_external_workspace_member_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            sandbox = pathlib.Path(directory)
            root = sandbox / "workspace"
            external = sandbox / "external-browser-adapter"
            root.mkdir()
            external.mkdir()
            (external / "Cargo.toml").write_text(
                '[package]\nname = "external-browser-adapter"\nversion = "0.1.0"\n',
                encoding="utf-8",
            )
            (root / "linked-browser-adapter").symlink_to(external, target_is_directory=True)
            (root / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["linked-browser-adapter"]\n',
                encoding="utf-8",
            )

            with self.assertRaisesRegex(AssertionError, "member escapes repository review root"):
                _canonical_workspace_member_manifests(root)


if __name__ == "__main__":
    unittest.main()
