"""Regression tests for credential-safe ChromeDriver error diagnostics."""

from __future__ import annotations

import http.server
import pathlib
import runpy
import threading
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "scripts" / "ci" / "run_mv3_compatibility.py"
SECRET_MARKER = "buyer-secret-marker-must-not-reach-ci"


class _ErrorResponseHandler(http.server.BaseHTTPRequestHandler):
    """Serve deterministic hostile ChromeDriver-shaped error responses."""

    response_status = 403
    response_body = (
        b'{"value":{"error":"unknown error","message":"'
        + SECRET_MARKER.encode("ascii")
        + b'"}}'
    )

    def do_GET(self) -> None:  # noqa: N802 - stdlib handler contract.
        self.send_response(self.response_status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(self.response_body)))
        self.end_headers()
        self.wfile.write(self.response_body)

    def log_message(self, _format: str, *args: object) -> None:
        """Keep the hostile marker out of test-server logging."""


class ChromeDriverErrorDiagnosticContractTests(unittest.TestCase):
    """ChromeDriver-controlled response bytes must not be reflected into CI errors."""

    def _request_against(self, *, status: int) -> RuntimeError:
        namespace = runpy.run_path(str(RUNNER), run_name="chromedriver_error_diagnostic_contract")
        json_request = namespace["_json_request"]

        class Handler(_ErrorResponseHandler):
            response_status = status

        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with self.assertRaises(RuntimeError) as captured:
                json_request(int(server.server_port), "GET", "/status")
            return captured.exception
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2.0)

    def test_http_error_does_not_reflect_response_body(self) -> None:
        """An HTTP error may retain its status but never ChromeDriver-controlled detail."""

        error = self._request_against(status=403)
        self.assertIn("403", str(error))
        self.assertNotIn(SECRET_MARKER, str(error))

    def test_webdriver_error_does_not_reflect_response_message(self) -> None:
        """A 2xx WebDriver error object must remain fail-closed without its raw message."""

        error = self._request_against(status=200)
        self.assertIn("WebDriver", str(error))
        self.assertNotIn(SECRET_MARKER, str(error))


if __name__ == "__main__":
    unittest.main()
