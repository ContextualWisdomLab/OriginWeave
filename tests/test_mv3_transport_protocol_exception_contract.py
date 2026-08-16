"""Regression contract for bounded WebDriver transport-protocol failures."""

from __future__ import annotations

import pathlib
import runpy
import unittest
import unittest.mock

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"


class ManifestV3TransportProtocolExceptionContractTests(unittest.TestCase):
    """Keep recoverable HTTP parser failures inside the typed runner boundary."""

    def test_http_protocol_exceptions_are_classified_without_raw_transport_text(self) -> None:
        """BadStatusLine and IncompleteRead must become one bounded RuntimeError."""

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

        for connection in (BadStatusConnection(), IncompleteReadConnection()):
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


if __name__ == "__main__":
    unittest.main()
