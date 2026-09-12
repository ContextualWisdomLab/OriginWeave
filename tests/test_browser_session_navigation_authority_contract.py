"""Repository contract for navigation-invalidated presentation authority."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-browser-session/src/lib.rs"

NAVIGATION_PRODUCTION_MARKERS = (
    "pub struct NavigationSettlementAuthority",
    "pub enum NavigationTerminationOutcome",
    "pub fn record_observed_navigation(",
    "pub fn record_observed_navigation_settled(",
    "pub fn record_observed_navigation_terminated(",
    "pub fn reestablish_presentation_authority(",
)


def _navigation_production_slice_present(source: str) -> bool:
    """Require the production navigation API to arrive as one coherent contract slice."""

    present = {marker: marker in source for marker in NAVIGATION_PRODUCTION_MARKERS}
    if any(present.values()) and not all(present.values()):
        missing = sorted(marker for marker, is_present in present.items() if not is_present)
        raise AssertionError(
            "partial Browser Session navigation production API is forbidden; missing: "
            + ", ".join(missing)
        )
    return all(present.values())


def _mask_rust_non_code(source: str) -> str:
    """Mask Rust comments and literals while preserving byte-for-byte positions."""

    masked = list(source)
    length = len(source)

    def blank(start: int, end: int) -> None:
        for index in range(start, end):
            if masked[index] != "\n":
                masked[index] = " "

    index = 0
    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end == -1:
                end = length
            blank(index, end)
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            cursor = index + 2
            while cursor < length and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            blank(index, cursor)
            index = cursor
            continue

        raw_match = re.match(r'(?:br|r)(?P<hashes>#{0,16})"', source[index:])
        if raw_match:
            hashes = raw_match.group("hashes")
            terminator = '"' + hashes
            body_start = index + raw_match.end()
            end = source.find(terminator, body_start)
            end = length if end == -1 else end + len(terminator)
            blank(index, end)
            index = end
            continue

        string_prefix = 2 if source.startswith('b"', index) else 1 if source[index] == '"' else 0
        if string_prefix:
            cursor = index + string_prefix
            escaped = False
            while cursor < length:
                char = source[cursor]
                cursor += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    break
            blank(index, cursor)
            index = cursor
            continue

        char_match = re.match(
            r"(?:b)?'(?:\\(?:[nrt0\\'\" ]|x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\})|[^'\\\n])'",
            source[index:],
        )
        if char_match:
            end = index + char_match.end()
            blank(index, end)
            index = end
            continue

        index += 1

    return "".join(masked)


def _inherent_impl_blocks(source: str, type_name: str) -> list[str]:
    """Return masked inherent impl bodies, including generic impl headers."""

    code = _mask_rust_non_code(source)
    pattern = re.compile(
        rf"\bimpl(?:\s*<[^{{}};]*>)?\s+{re.escape(type_name)}"
        rf"(?:\s*<[^{{}};]*>)?(?:\s+where\b[^{{}};]*)?\s*\{{"
    )
    blocks: list[str] = []
    for match in pattern.finditer(code):
        opening_brace = code.find("{", match.start(), match.end())
        depth = 0
        for index in range(opening_brace, len(code)):
            if code[index] == "{":
                depth += 1
            elif code[index] == "}":
                depth -= 1
                if depth == 0:
                    blocks.append(code[opening_brace + 1 : index])
                    break
    return blocks


class BrowserSessionNavigationAuthorityContractTests(unittest.TestCase):
    """Keep fresh presentation authority on the exact mutable bound owner."""

    def test_read_projection_cannot_mint_presentation_authority(self) -> None:
        """A raw browsing-context id on the read model must not mint authority."""

        source = SOURCE.read_text(encoding="utf-8")
        if not _navigation_production_slice_present(source):
            return
        browser_session_impl = source.split("impl BrowserSession {", 1)[1].split(
            "impl<P: DisposableContextPort> BoundBrowserSession<P>", 1
        )[0]

        self.assertNotIn("pub fn presentation_authority(", browser_session_impl)
        self.assertIn("pub fn record_observed_navigation(", source)
        self.assertIn("pub fn record_observed_navigation_settled(", source)
        self.assertIn("pub fn record_observed_navigation_terminated(", source)
        self.assertIn("pub fn reestablish_presentation_authority(", source)

    def test_navigation_observation_is_bound_to_exact_session_context_generation(self) -> None:
        """A delayed event must not alias across aggregate incarnations, contexts, or epochs."""

        source = SOURCE.read_text(encoding="utf-8")
        if not _navigation_production_slice_present(source):
            return
        signature = re.search(
            r"pub fn record_observed_navigation\s*\((?P<params>.*?)\)\s*->(?P<return_type>[^\{]+)\{",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("BrowserSessionIncarnation", params)
        self.assertIn("BrowsingContextId", params)
        self.assertIn("BrowserContextEpoch", params)
        self.assertIn(
            "NavigationSettlementAuthority",
            signature.group("return_type"),
            "an admitted navigation start must issue an opaque aggregate-bound settlement authority",
        )

    def test_navigation_settlement_requires_aggregate_issued_authority(self) -> None:
        """Raw provenance must never become authority to unlock presentation re-establishment."""

        source = SOURCE.read_text(encoding="utf-8")
        if not _navigation_production_slice_present(source):
            return
        signature = re.search(
            r"pub fn record_observed_navigation_settled\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("NavigationSettlementAuthority", params)
        self.assertNotIn(
            "BrowserSessionIncarnation",
            params,
            "session incarnation is provenance for minting the settlement authority, not settlement authority itself",
        )
        self.assertNotIn(
            "BrowsingContextId",
            params,
            "raw browser addressability must not unlock a pending navigation",
        )
        self.assertNotIn(
            "BrowserContextEpoch",
            params,
            "a reconstructible epoch is correlation evidence, not a settlement capability",
        )

    def test_navigation_abort_and_failure_have_explicit_terminal_transition(self) -> None:
        """Negative terminal events must close pending state without pretending they committed."""

        source = SOURCE.read_text(encoding="utf-8")
        if not _navigation_production_slice_present(source):
            return
        signature = re.search(
            r"pub fn record_observed_navigation_terminated\s*\((?P<params>.*?)\)\s*->",
            source,
            flags=re.DOTALL,
        )
        declaration = re.search(
            r"pub enum NavigationTerminationOutcome\s*\{(?P<body>.*?)\}",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(signature)
        params = signature.group("params")
        self.assertIn("NavigationSettlementAuthority", params)
        self.assertIn("NavigationTerminationOutcome", params)
        self.assertNotIn("BrowserSessionIncarnation", params)
        self.assertNotIn("BrowsingContextId", params)
        self.assertNotIn("BrowserContextEpoch", params)

        self.assertIsNotNone(declaration)
        body = declaration.group("body")
        self.assertRegex(body, r"\bAborted\b")
        self.assertRegex(body, r"\bFailed\b")

    def test_navigation_settlement_authority_is_not_publicly_constructible(self) -> None:
        """Only Browser Session may mint the witness that unlocks settlement."""

        source = SOURCE.read_text(encoding="utf-8")
        if not _navigation_production_slice_present(source):
            return
        code = _mask_rust_non_code(source)
        declaration = re.search(
            r"pub struct NavigationSettlementAuthority\s*\{(?P<body>.*?)\}",
            code,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(declaration)
        self.assertNotRegex(
            declaration.group("body"),
            r"\bpub(?:\([^)]*\))?\s+\w+\s*:",
            "settlement-authority state must remain private to the Browser Session crate",
        )
        self.assertNotRegex(
            code,
            r"#\s*\[\s*derive\s*\([^\]]*\bDefault\b[^\]]*\)\s*\]"
            r"(?:(?:\s*#\s*\[[^\]]*\])|\s)*"
            r"pub\s+struct\s+NavigationSettlementAuthority\b",
            "Default would let raw callers fabricate a settlement witness",
        )
        self.assertNotRegex(
            code,
            r"\bimpl(?:\s*<[^{};]*>)?\s+"
            r"(?:Default|From\s*<[^{};]+>|TryFrom\s*<[^{};]+>)\s+for\s+"
            r"NavigationSettlementAuthority\b",
            "conversion/default traits must not expose a caller-mintable witness path",
        )

        impl_blocks = _inherent_impl_blocks(source, "NavigationSettlementAuthority")
        public_functions = re.compile(
            r"\bpub(?:\([^)]*\))?\s+"
            r"(?:(?:const|async|unsafe|extern(?:\s+\"[^\"]+\")?)\s+)*"
            r"fn\s+(?P<name>\w+)\s*(?:<[^>{}]*>)?\s*"
            r"\((?P<params>.*?)\)",
            flags=re.DOTALL,
        )
        for impl_block in impl_blocks:
            for public_function in public_functions.finditer(impl_block):
                params = public_function.group("params").strip()
                first_param = params.split(",", 1)[0].strip() if params else ""
                self.assertRegex(
                    first_param,
                    r"^(?:&\s*(?:'\w+\s*)?(?:mut\s+)?self|(?:mut\s+)?self)\b",
                    f"public associated function {public_function.group('name')} would let raw callers construct or transform settlement authority without an existing witness",
                )


if __name__ == "__main__":
    unittest.main()
