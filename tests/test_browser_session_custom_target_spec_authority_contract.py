import importlib.util
import json
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
COMPILER_AUTHORITY_TEST = ROOT / "tests/test_browser_session_cargo_compiler_authority_contract.py"

spec = importlib.util.spec_from_file_location(
    "browser_session_cargo_compiler_authority",
    COMPILER_AUTHORITY_TEST,
)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load Browser Session Cargo compiler authority contract")
compiler_authority = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compiler_authority)


class BrowserSessionCustomTargetSpecAuthorityContractTests(unittest.TestCase):
    """Keep repository-selected rustc target specifications inside reviewed provenance."""

    def _workspace_with_build_target(self, target_value: str) -> pathlib.Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = pathlib.Path(directory.name)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["adapter"]\nresolver = "3"\n',
            encoding="utf-8",
        )
        adapter = root / "adapter"
        (adapter / "src").mkdir(parents=True)
        (adapter / "src/lib.rs").write_text("pub fn adapter_surface() {}\n", encoding="utf-8")
        (adapter / "Cargo.toml").write_text(
            '[package]\nname = "adapter"\nversion = "0.1.0"\nedition = "2024"\n',
            encoding="utf-8",
        )
        cargo = root / ".cargo"
        cargo.mkdir()
        (cargo / "config.toml").write_text(
            f"[unstable]\njson-target-spec = true\n\n[build]\ntarget = {target_value}\n",
            encoding="utf-8",
        )
        targets = root / "targets"
        targets.mkdir()
        (targets / "review-bypass.json").write_text(
            json.dumps(
                {
                    "llvm-target": "x86_64-unknown-linux-gnu",
                    "arch": "x86_64",
                    "target-pointer-width": "64",
                    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128",
                    "linker": "tools/review-bypass-linker",
                }
            ),
            encoding="utf-8",
        )
        return root

    def test_build_target_custom_json_fails_closed(self) -> None:
        root = self._workspace_with_build_target('"targets/review-bypass.json"')
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            compiler_authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_build_target_array_with_custom_json_fails_closed(self) -> None:
        root = self._workspace_with_build_target(
            '["x86_64-unknown-linux-gnu", "targets/review-bypass.json"]'
        )
        with self.assertRaisesRegex(AssertionError, "Cargo .*execution override"):
            compiler_authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_builtin_target_triple_remains_allowed(self) -> None:
        root = self._workspace_with_build_target('"x86_64-unknown-linux-gnu"')
        compiler_authority._assert_no_repository_cargo_compiler_execution_overrides(root)

    def test_host_tuple_remains_allowed(self) -> None:
        root = self._workspace_with_build_target('"host-tuple"')
        compiler_authority._assert_no_repository_cargo_compiler_execution_overrides(root)


if __name__ == "__main__":
    unittest.main()
