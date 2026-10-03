"""Keep the added TCP admission regression functions meaningfully documented."""

from __future__ import annotations

import pathlib
import re
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class NetworkLiteralDocumentationContractTests(unittest.TestCase):
    """Require useful descriptions without pinning the implementation's wording."""

    def test_added_regression_functions_explain_their_evidence_scope(self) -> None:
        """Each added helper and regression must have a substantive Rust doc comment."""

        functions = {
            "crates/originweave-network/src/connection.rs": (
                "validation_errors_cover_every_public_contract",
                "network_errors_have_a_stable_standard_error_contract",
                "assert_standard_error_contract",
            ),
            "crates/originweave-network/tests/support/connection_literal_bounds.rs": (
                "approved_loopback",
                "literal_thirty_second_deadline_admits_exact_input_and_rejects_adjacent_overflow",
                "literal_four_attempt_budget_admits_every_count_and_rejects_adjacent_overflow",
            ),
        }
        for relative, names in functions.items():
            source = (ROOT / relative).read_text(encoding="utf-8")
            for name in names:
                with self.subTest(path=relative, function=name):
                    pattern = (
                        r"(?P<docs>(?:^[ \t]*///[^\n]*\n)+)"
                        r"(?:^[ \t]*#\[[^\n]*\]\n)*"
                        r"^[ \t]*fn " + re.escape(name) + r"\b"
                    )
                    match = re.search(pattern, source, re.MULTILINE)
                    self.assertIsNotNone(match, f"{relative}:{name} lacks a Rust doc comment")
                    if match is not None:
                        words = re.findall(r"[A-Za-z]+", match.group("docs"))
                        self.assertGreaterEqual(len(words), 6, f"{relative}:{name} needs a useful description")


if __name__ == "__main__":
    unittest.main()
