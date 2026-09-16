"""Repository contracts for Browser Session recovery-only adapter custody."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates/originweave-browser-session"


def _rust_impl_headers(source: str) -> list[str]:
    """Return Rust impl headers while ignoring nested delimiters and literal/comment text."""

    headers: list[str] = []
    length = len(source)
    index = 0

    def starts_lifetime(position: int) -> bool:
        """Distinguish Rust lifetimes/labels from quoted character literals."""

        if source[position] != "'" or position + 1 >= length:
            return False
        cursor = position + 1
        if not (source[cursor].isalpha() or source[cursor] == "_"):
            return False
        cursor += 1
        while cursor < length and (source[cursor].isalnum() or source[cursor] == "_"):
            cursor += 1
        return cursor >= length or source[cursor] != "'"

    def skip_non_code(position: int) -> int:
        if source.startswith("//", position):
            newline = source.find("\n", position + 2)
            return length if newline < 0 else newline + 1
        if source.startswith("/*", position):
            depth = 1
            cursor = position + 2
            while cursor < length and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            return cursor

        raw = re.match(r"(?:br|r)(?P<hashes>#{0,255})\"", source[position:])
        if raw:
            hashes = raw.group("hashes")
            cursor = position + raw.end()
            terminator = '"' + hashes
            end = source.find(terminator, cursor)
            return length if end < 0 else end + len(terminator)

        if source[position] == "'" and starts_lifetime(position):
            return position

        if source[position] in ('"', "'"):
            quote = source[position]
            cursor = position + 1
            while cursor < length:
                if source[cursor] == "\\":
                    cursor += 2
                    continue
                if source[cursor] == quote:
                    return cursor + 1
                cursor += 1
            return length
        return position

    while index < length:
        skipped = skip_non_code(index)
        if skipped != index:
            index = skipped
            continue
        match = re.match(r"impl\b", source[index:])
        if not match:
            index += 1
            continue
        if index > 0 and (source[index - 1].isalnum() or source[index - 1] == "_"):
            index += 1
            continue

        start = index
        cursor = index + match.end()
        angle_depth = 0
        paren_depth = 0
        bracket_depth = 0
        nested_brace_depth = 0
        body_start: int | None = None
        non_code_ranges: list[tuple[int, int]] = []

        while cursor < length:
            skipped = skip_non_code(cursor)
            if skipped != cursor:
                non_code_ranges.append((cursor, skipped))
                cursor = skipped
                continue
            char = source[cursor]
            if nested_brace_depth:
                if char == "{":
                    nested_brace_depth += 1
                elif char == "}":
                    nested_brace_depth -= 1
                cursor += 1
                continue
            if char == "(":
                paren_depth += 1
            elif char == ")" and paren_depth:
                paren_depth -= 1
            elif char == "[":
                bracket_depth += 1
            elif char == "]" and bracket_depth:
                bracket_depth -= 1
            elif char == "<" and paren_depth == 0 and bracket_depth == 0:
                angle_depth += 1
            elif char == ">" and angle_depth and paren_depth == 0 and bracket_depth == 0:
                angle_depth -= 1
            elif char == "{":
                if angle_depth == 0 and paren_depth == 0 and bracket_depth == 0:
                    body_start = cursor
                    break
                nested_brace_depth = 1
            elif char == ";" and angle_depth == 0 and paren_depth == 0 and bracket_depth == 0:
                break
            cursor += 1

        if body_start is not None:
            header_parts: list[str] = []
            fragment_start = start
            for non_code_start, non_code_end in non_code_ranges:
                header_parts.append(source[fragment_start:non_code_start])
                header_parts.append(" ")
                fragment_start = non_code_end
            header_parts.append(source[fragment_start:body_start])
            headers.append(re.sub(r"\s+", " ", "".join(header_parts)).strip())
            index = body_start + 1
        else:
            index = cursor + 1

    return headers


class BrowserSessionRecoveryOperationContractTests(unittest.TestCase):
    """Keep recovery I/O purpose-bounded to one exact fact on the consumed adapter."""

    def test_recovery_dispatch_stays_inside_native_owner_module(self) -> None:
        """Do not reopen raw adapter access to bridge recovery custody."""

        lib_source = (CRATE / "src/lib.rs").read_text(encoding="utf-8")
        browser_source = (CRATE / "src/browser_session.rs").read_text(encoding="utf-8")
        recovery_source = (CRATE / "src/recovery.rs").read_text(encoding="utf-8")

        self.assertIn("mod browser_session;", lib_source)
        self.assertNotIn('include!("browser_session.rs")', lib_source)
        self.assertIn("pub(crate) fn dispatch_recovery_operation", browser_source)
        self.assertNotIn("pub fn dispatch_recovery_operation", browser_source)

        for symbol in (
            "pub struct RecoveryContextOperationRequest",
            "pub trait RecoveryContextOperationPort",
            "pub enum RecoveryContextOperationError",
            "pub fn execute_recovery_context_operation",
        ):
            self.assertIn(symbol, recovery_source)

        request_struct = recovery_source.split(
            "pub struct RecoveryContextOperationRequest<O> {", 1
        )[1].split("\n}", 1)[0]
        self.assertNotRegex(request_struct, r"(?m)^\s*pub(?:\([^)]*\))?\s+")
        self.assertIn("Option<BrowserSessionRecoveryEvidence>", request_struct)
        self.assertIn("Option<DisposableContextCreateRecoveryEvidence>", request_struct)
        self.assertNotIn("Vec<BrowserSessionRecoveryEvidence>", request_struct)
        self.assertNotIn("Vec<DisposableContextCreateRecoveryEvidence>", request_struct)

        operation_impl = recovery_source.split(
            "impl<P: RecoveryContextOperationPort> BoundBrowserSessionRecovery<P>", 1
        )[1].split("impl<P: RecoverySettlementPort>", 1)[0]
        self.assertIn("fact: RecoveryFact", operation_impl)
        self.assertIn("select_recovery_fact(fact)", operation_impl)
        self.assertIn("RecoveryContextOperationError::AuthorityMismatch", operation_impl)
        self.assertIn("RecoveryContextOperationError::StaleFact", operation_impl)

        request_impl_headers = [
            header
            for header in _rust_impl_headers(recovery_source)
            if "RecoveryContextOperationRequest" in header
        ]
        inherent_impl_headers = [
            header
            for header in request_impl_headers
            if " for RecoveryContextOperationRequest" not in header
        ]
        self.assertEqual(
            inherent_impl_headers,
            ["impl<O> RecoveryContextOperationRequest<O>"],
            "request accessors must remain the sole inherent impl; every new inherent impl requires review",
        )

        request_impl = recovery_source.split(
            "impl<O> RecoveryContextOperationRequest", 1
        )[1].split("pub trait RecoveryContextOperationPort", 1)[0]
        public_methods = re.findall(
            r"(?m)^\s*pub(?:\s+const)?\s+fn\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)",
            request_impl,
        )
        self.assertGreater(len(public_methods), 0)
        for method_name, parameters in public_methods:
            self.assertIn(
                "&self",
                parameters,
                f"{method_name} must remain an accessor, not a public construction path",
            )

        for constructor_pattern in (
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::default::)?Default\s+for\s+RecoveryContextOperationRequest",
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::convert::)?From<[^{}]+?>\s+for\s+RecoveryContextOperationRequest",
            r"impl(?:\s*<[^{}]*?>)?\s+(?:::)?(?:(?:core|std)::convert::)?TryFrom<[^{}]+?>\s+for\s+RecoveryContextOperationRequest",
        ):
            self.assertNotRegex(recovery_source, constructor_pattern)

        self.assertNotIn("pub fn browser_session(&self)", recovery_source)
        self.assertNotIn("pub fn port", recovery_source)
        self.assertNotIn("pub const fn port", recovery_source)

    def test_impl_header_parser_keeps_braced_const_where_predicates_visible(self) -> None:
        """A braced const expression in a where clause must not hide a second inherent impl."""

        sample = """
impl<O> RecoveryContextOperationRequest<O> {
    pub fn operation(&self) -> &O { todo!() }
}
impl<O> RecoveryContextOperationRequest<O>
where
    O: RecoveryMarker<{ 1 + 1 }>,
{
    pub fn from_raw(operation: O) -> Self { todo!() }
}
"""
        request_headers = [
            header
            for header in _rust_impl_headers(sample)
            if "RecoveryContextOperationRequest" in header
        ]
        self.assertEqual(len(request_headers), 2)
        self.assertIn("RecoveryMarker<{ 1 + 1 }>", request_headers[1])

    def test_impl_header_parser_removes_non_code_text_before_classification(self) -> None:
        """Comments and literals must not disguise an inherent request impl as a trait impl."""

        sample = r'''
impl<O> RecoveryContextOperationRequest<O> {
    pub fn operation(&self) -> &O { todo!() }
}
impl<O> RecoveryContextOperationRequest<O>
// for RecoveryContextOperationRequest
{
    pub fn from_raw(operation: O) -> Self { todo!() }
}
impl<O> RecoveryContextOperationRequest<O>
/* for RecoveryContextOperationRequest */
{
    pub fn from_raw_again(operation: O) -> Self { todo!() }
}
impl<O> RecoveryContextOperationRequest<O>
where
    O: RecoveryMarker<{ b" for RecoveryContextOperationRequest".len() }>,
{
    pub fn from_byte_literal(operation: O) -> Self { todo!() }
}
impl<O> RecoveryContextOperationRequest<O>
where
    O: RecoveryMarker<{ br#" for RecoveryContextOperationRequest"#.len() }>,
{
    pub fn from_raw_literal(operation: O) -> Self { todo!() }
}
'''
        request_headers = [
            header
            for header in _rust_impl_headers(sample)
            if "RecoveryContextOperationRequest" in header
        ]
        self.assertEqual(len(request_headers), 5)
        self.assertEqual(
            [
                header
                for header in request_headers
                if " for RecoveryContextOperationRequest" not in header
            ],
            request_headers,
        )

    def test_impl_header_parser_preserves_lifetimes_while_skipping_char_literals(self) -> None:
        """Apostrophe handling must not consume lifetimes while removing character literals."""

        sample = """
impl<'a, O> RecoveryContextOperationRequest<&'a O>
where
    O: RecoveryMarker<'a, {'x' as u32}>,
{
    pub fn borrow(&self) -> &'a O { todo!() }
}
"""
        headers = _rust_impl_headers(sample)
        self.assertEqual(len(headers), 1)
        self.assertIn("impl<'a, O> RecoveryContextOperationRequest<&'a O>", headers[0])
        self.assertNotIn("'x'", headers[0])

    def test_hostile_fixture_preserves_uncertainty_after_adapter_result(self) -> None:
        """Adapter success or failure must not silently become reconciliation proof."""

        hostile = (CRATE / "tests/recovery_same_adapter_operation.rs").read_text(
            encoding="utf-8"
        )
        for token in (
            "RecoveryContextOperationPort",
            "execute_recovery_context_operation",
            "recovery.recovery_fact(0)",
            "recovery.create_attempt_recovery_fact(0)",
            "request.browser_session()",
            "request.incarnation()",
            "request.state()",
            "request.recovery_evidence()",
            "request.create_attempt_recovery_evidence()",
            "RecoveryContextOperationError::Adapter",
            "expected_recovery_evidence",
            "expected_create_attempt_recovery_evidence",
            "request must expose only the selected exact recovery fact",
            "request must expose only the selected exact create-attempt fact",
            "generic recovery adapter success is not itself destruction or reconciliation proof",
            "recovery operation dispatch must not erase unresolved ownership evidence",
            "recovery operation dispatch must not erase create-attempt provenance",
        ):
            self.assertIn(token, hostile)

    def test_hostile_fixture_rejects_foreign_and_stale_operation_facts_before_io(self) -> None:
        """One fact may authorize only its current exact recovery operation scope."""

        hostile = (CRATE / "tests/recovery_operation_exact_fact_scope.rs").read_text(
            encoding="utf-8"
        )
        for token in (
            "execute_recovery_context_operation(first_fact",
            "adapter request must expose only the selected recovery fact, not sibling uncertainty",
            "RecoveryContextOperationError::StaleFact",
            "RecoveryContextOperationError::AuthorityMismatch",
            "a fact issued before ledger mutation must fail before adapter I/O",
            "foreign_recovery_fact_is_rejected_before_operation_io",
        ):
            self.assertIn(token, hostile)

    def test_architecture_docs_describe_current_recovery_surface(self) -> None:
        """ADR, traceability, and UML must not describe the pre-operation wrapper."""

        adr = (
            ROOT / "docs/adr/0116-browser-session-recovery-custody-and-hot-ownership.md"
        ).read_text(encoding="utf-8")
        trace = (
            ROOT / "docs/traceability/browser-session-lifecycle-authority.md"
        ).read_text(encoding="utf-8")
        uml = (ROOT / "docs/uml/browser-session-lifecycle-authority.md").read_text(
            encoding="utf-8"
        )

        for document in (adr, trace, uml):
            self.assertIn("RecoveryContextOperationPort", document)
            self.assertIn("RecoveryContextOperationRequest", document)
            self.assertIn("same", document.lower())
            self.assertIn("RecoveryFact", document)
        self.assertIn("pub(crate)", adr)
        self.assertIn("pub(crate)", trace)
        self.assertIn("success/failure does not clear Browser Session uncertainty", uml)
        self.assertIn("#316", adr)
        self.assertIn("#316", trace)
        self.assertIn("#316", uml)


if __name__ == "__main__":
    unittest.main()
