"""Repository contract for navigation-invalidated presentation authority."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-browser-session/src/lib.rs"


def _inherent_impl_blocks(source: str, type_name: str) -> list[str]:
    """Return exact inherent impl bodies without depending on a Rust parser."""

    blocks: list[str] = []
    pattern = re.compile(rf"\bimpl\s+{re.escape(type_name)}\s*\{{")
    for match in pattern.finditer(source):
        opening_brace = source.find("{", match.start())
        depth = 0
        for index in range(opening_brace, len(source)):
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
                if depth == 0:
                    blocks.append(source[opening_brace + 1 : index])
                    break
    return blocks


class BrowserSessionNavigationAuthorityContractTests(unittest.TestCase):
    """Keep fresh presentation authority on the exact mutable bound owner."""

    def test_read_projection_cannot_mint_presentation_authority(self) -> None:
        """A raw browsing-context id on the read model must not mint authority."""

        source = SOURCE.read_text(encoding="utf-8")
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
        declaration = re.search(
            r"pub struct NavigationSettlementAuthority\s*\{(?P<body>.*?)\}",
            source,
            flags=re.DOTALL,
        )

        self.assertIsNotNone(declaration)
        self.assertNotRegex(
            declaration.group("body"),
            r"\bpub(?:\([^)]*\))?\s+\w+\s*:",
            "settlement-authority state must remain private to the Browser Session crate",
        )
        self.assertNotRegex(
            source,
            r"#\[derive\([^\]]*\bDefault\b[^\]]*\)\]\s*pub struct NavigationSettlementAuthority",
            "Default would let raw callers fabricate a settlement witness",
        )
        self.assertNotRegex(
            source,
            r"impl\s+(?:Default|From<[^>]+>|TryFrom<[^>]+>)\s+for\s+NavigationSettlementAuthority\b",
            "conversion/default traits must not expose a caller-mintable witness path",
        )

        impl_blocks = _inherent_impl_blocks(source, "NavigationSettlementAuthority")
        public_functions = re.compile(
            r"\bpub(?:\([^)]*\))?\s+(?:const\s+)?fn\s+(?P<name>\w+)\s*\((?P<params>.*?)\)",
            flags=re.DOTALL,
        )
        for impl_block in impl_blocks:
            for public_function in public_functions.finditer(impl_block):
                params = public_function.group("params").strip()
                first_param = params.split(",", 1)[0].strip() if params else ""
                self.assertRegex(
                    first_param,
                    r"^(?:&\s*(?:mut\s+)?self|(?:mut\s+)?self)\b",
                    f"public associated function {public_function.group('name')} would let raw callers construct or transform settlement authority without an existing witness",
                )


if __name__ == "__main__":
    unittest.main()
