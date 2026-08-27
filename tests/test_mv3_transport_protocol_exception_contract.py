"""Regression contracts for bounded MV3 transport failures and their release notes."""

from __future__ import annotations

import pathlib
import runpy
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
CHANGELOG = ROOT / "CHANGELOG.md"


class ManifestV3TransportProtocolExceptionContractTests(unittest.TestCase):
    """Keep recoverable HTTP parser failures inside the typed runner boundary."""

    def test_http_protocol_exceptions_are_classified_without_raw_transport_text(self) -> None:
        """Parser failures must become bounded errors with no retained raw exception chain."""

        namespace = runpy.run_path(str(RUNNER), run_name="mv3_transport_contract")
        json_request = namespace["_json_request"]
        http_module = namespace["http"]
        raw_secret = "secret-token /home/runner/private https://example.invalid"

        class BadStatusConnection:
            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> object:
                raise http_module.client.BadStatusLine(raw_secret)

            def close(self) -> None:
                return None

        class IncompleteReadResponse:
            status = 200

            def read(self, _limit: int) -> bytes:
                partial = raw_secret.encode("utf-8")
                raise http_module.client.IncompleteRead(partial, len(partial) + 10)

        class IncompleteReadConnection:
            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> IncompleteReadResponse:
                return IncompleteReadResponse()

            def close(self) -> None:
                return None

        class InvalidUtf8Response:
            status = 200

            def read(self, _limit: int) -> bytes:
                return b"\xffsecret-token /home/runner/private https://example.invalid"

        class InvalidUtf8Connection:
            def request(self, *_args: object, **_kwargs: object) -> None:
                return None

            def getresponse(self) -> InvalidUtf8Response:
                return InvalidUtf8Response()

            def close(self) -> None:
                return None

        for connection in (
            BadStatusConnection(),
            IncompleteReadConnection(),
            InvalidUtf8Connection(),
        ):
            with self.subTest(connection=type(connection).__name__):
                with unittest.mock.patch.object(
                    http_module.client,
                    "HTTPConnection",
                    return_value=connection,
                ):
                    with self.assertRaises(RuntimeError) as raised:
                        json_request(9515, "GET", "/status")

                rendered = str(raised.exception)
                self.assertEqual(rendered, "WebDriver transport protocol failure")
                self.assertNotIn("secret-token", rendered)
                self.assertNotIn("/home/runner/private", rendered)
                self.assertNotIn("example.invalid", rendered)
                self.assertIsNone(raised.exception.__cause__)
                self.assertIsNone(raised.exception.__context__)

    def test_unreleased_changelog_change_type_headings_are_unique(self) -> None:
        """Keep each Keep a Changelog change type singular within Unreleased."""

        text = CHANGELOG.read_text(encoding="utf-8")
        marker = "## [Unreleased]"
        self.assertIn(marker, text)
        unreleased = text.split(marker, 1)[1]
        next_release = unreleased.find("\n## [")
        if next_release >= 0:
            unreleased = unreleased[:next_release]
        headings = [
            line.strip()
            for line in unreleased.splitlines()
            if line.startswith("### ")
        ]
        self.assertEqual(
            len(headings),
            len(set(headings)),
            f"duplicate Unreleased change-type headings: {headings}",
        )


if __name__ == "__main__":
    unittest.main()
