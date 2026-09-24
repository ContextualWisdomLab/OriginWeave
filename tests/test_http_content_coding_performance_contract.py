"""Contract for the stacked HTTP content-coding buyer-path performance boundary."""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-http/examples/content_coding_stack_performance.rs"


class HttpContentCodingPerformanceContractTests(unittest.TestCase):
    """Prevent buyer-path receipts from omitting governed transport or evidence authority."""

    def test_timer_covers_destination_tcp_tls_and_http_exchange(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("fn measure_profile(")
        end = source.index("fn receipt_line(", start)
        measure_profile = source[start:end]

        timer = measure_profile.index("let started = Instant::now();")
        authentication = measure_profile.index(
            "let connection = authenticated_connection(&origin, socket_address, root_der.to_vec())?;"
        )
        execution = measure_profile.index("let response = plan")

        self.assertLess(
            timer,
            authentication,
            "buyer-path timer must start before governed TCP/TLS authentication",
        )
        self.assertLess(
            authentication,
            execution,
            "authenticated transport must precede the HTTP exchange",
        )
        self.assertIn(
            "direct_connection(origin, socket_address)?",
            source,
            "benchmark must retain the destination-authorized direct connection path",
        )
        self.assertIn(
            "plan.authenticate()",
            source,
            "benchmark must retain authenticated TLS before HTTP execution",
        )

    def test_caller_produced_receipt_cannot_claim_authenticated_evidence_acceptance(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        for marker in [
            "evidence_authority",
            "caller_produced_unattested_receipt",
            "evidence_acceptance_status",
            "UNACCEPTED_UNATTESTED_RECEIPT",
        ]:
            self.assertIn(
                marker,
                source,
                f"performance receipt must expose fail-closed evidence authority marker: {marker}",
            )


if __name__ == "__main__":
    unittest.main()
