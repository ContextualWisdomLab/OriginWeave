"""Defense-in-depth contracts for recovery request impl classification."""

from __future__ import annotations

import importlib.util
import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE_CONTRACT = ROOT / "tests/test_browser_session_recovery_operation_contract.py"
RECOVERY_SOURCE = ROOT / "crates/originweave-browser-session/src/recovery.rs"

_spec = importlib.util.spec_from_file_location("_recovery_operation_contract", BASE_CONTRACT)
if _spec is None or _spec.loader is None:
    raise RuntimeError("cannot load recovery operation contract helper")
_base_contract = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_base_contract)
_rust_impl_headers = _base_contract._rust_impl_headers


def _is_inherent_impl_header(header: str) -> bool:
    """Classify impl kind using only top-level Rust tokens before a where clause."""

    if not re.match(r"^impl\b", header):
        return False

    angle_depth = 0
    paren_depth = 0
    bracket_depth = 0
    brace_depth = 0
    cursor = len("impl")
    length = len(header)

    while cursor < length:
        char = header[cursor]
        if char == "<":
            angle_depth += 1
            cursor += 1
            continue
        if char == ">" and angle_depth:
            angle_depth -= 1
            cursor += 1
            continue
        if char == "(":
            paren_depth += 1
            cursor += 1
            continue
        if char == ")" and paren_depth:
            paren_depth -= 1
            cursor += 1
            continue
        if char == "[":
            bracket_depth += 1
            cursor += 1
            continue
        if char == "]" and bracket_depth:
            bracket_depth -= 1
            cursor += 1
            continue
        if char == "{":
            brace_depth += 1
            cursor += 1
            continue
        if char == "}" and brace_depth:
            brace_depth -= 1
            cursor += 1
            continue

        top_level = not (angle_depth or paren_depth or bracket_depth or brace_depth)
        if top_level and (char.isalpha() or char == "_"):
            end = cursor + 1
            while end < length and (header[end].isalnum() or header[end] == "_"):
                end += 1
            token = header[cursor:end]
            if token == "for":
                return False
            if token == "where":
                return True
            cursor = end
            continue
        cursor += 1

    return True


class BrowserSessionRecoveryImplSurfaceContractTests(unittest.TestCase):
    """Prevent nested token text from hiding a second inherent request impl."""

    def test_current_request_has_one_inherent_impl(self) -> None:
        """Trait classification must depend on top-level syntax, not substring text."""

        recovery_source = RECOVERY_SOURCE.read_text(encoding="utf-8")
        request_headers = [
            header
            for header in _rust_impl_headers(recovery_source)
            if "RecoveryContextOperationRequest" in header
        ]
        inherent_headers = [
            header for header in request_headers if _is_inherent_impl_header(header)
        ]
        self.assertEqual(
            inherent_headers,
            ["impl<O> RecoveryContextOperationRequest<O>"],
            "request accessors must remain the sole inherent impl",
        )

    def test_nested_macro_for_tokens_do_not_disguise_inherent_impl(self) -> None:
        """A legal macro token tree containing `for Type` is not a trait impl."""

        sample = r'''
macro_rules! marker {
    ($($tokens:tt)*) => { 2usize };
}
impl<O> RecoveryContextOperationRequest<O>
where
    O: RecoveryMarker<{ marker! { for RecoveryContextOperationRequest } }>,
{
    pub fn from_macro(operation: O) -> Self { todo!() }
}
impl<O> RecoveryContextOperationPort for RecoveryContextOperationRequest<O> {
    fn execute(&mut self) { todo!() }
}
'''
        request_headers = [
            header
            for header in _rust_impl_headers(sample)
            if "RecoveryContextOperationRequest" in header
        ]
        self.assertEqual(len(request_headers), 2)
        self.assertTrue(_is_inherent_impl_header(request_headers[0]))
        self.assertFalse(_is_inherent_impl_header(request_headers[1]))
        self.assertIn(" for RecoveryContextOperationRequest", request_headers[0])


if __name__ == "__main__":
    unittest.main()
