"""Contract for the stacked HTTP content-coding buyer-path performance boundary."""

from __future__ import annotations

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/originweave-http/examples/content_coding_stack_performance.rs"
EVIDENCE_PACKAGER = (
    ROOT / "crates/originweave-http/examples/content_coding_performance_evidence.rs"
)


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

    def test_synthetic_fixture_cannot_claim_commercial_acceptance(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        for marker in [
            "fixture_authority",
            "deterministic_synthetic_no_external_dataset",
            "fixture_acceptance_status",
            "UNACCEPTED_SYNTHETIC_FIXTURE",
            "FIXTURE_AUTHORITY_SOURCE.acceptance_eligible()",
        ]:
            self.assertIn(
                marker,
                source,
                f"synthetic buyer-path fixture must remain explicit and fail closed: {marker}",
            )

        packager = EVIDENCE_PACKAGER.read_text(encoding="utf-8")
        for marker in [
            "fixture_authority",
            "fixture_acceptance_status",
            '"fixture_kind"',
            "deterministic_synthetic_no_external_dataset",
            "UNACCEPTED_SYNTHETIC_FIXTURE",
        ]:
            self.assertIn(
                marker,
                packager,
                f"attestable fixture document must preserve fixture authority: {marker}",
            )

    def test_source_acceptance_is_independent_of_latency_budget(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "const fn acceptance_status(self) -> &'static str",
            source,
            "source provenance acceptance must not take latency-budget state as input",
        )
        self.assertNotIn(
            "acceptance_status(self, budget_passed",
            source,
            "latency failure must not be reclassified as source-provenance failure",
        )
        self.assertIn(
            "Self::Explicit => \"PASS\"",
            source,
            "an explicit exact source revision remains provenance-accepted independently of p95",
        )
        self.assertIn(
            "else {\n        budget_status\n    };",
            source,
            "final commercial acceptance must evaluate budget separately after authority gates",
        )

    def test_leaf_evidence_can_be_materialized_for_central_attestation_intake(self) -> None:
        source = EVIDENCE_PACKAGER.read_text(encoding="utf-8")
        for marker in [
            'const SELECTED_PROFILE: &str = "stacked-content-coding";',
            'const RESULT_FILENAME: &str = "content-coding-result.json";',
            'const RUNTIME_FILENAME: &str = "content-coding-runtime.json";',
            'const FIXTURE_FILENAME: &str = "content-coding-fixture.json";',
            '"selected_profile"',
            '"candidate_sha"',
            'create_new(true)',
            'result_sha256=',
            'runtime_evidence_sha256=',
            'fixture_sha256=',
        ]:
            self.assertIn(
                marker,
                source,
                f"leaf performance evidence packager must expose central-intake marker: {marker}",
            )
        self.assertNotIn(
            "product-performance-attestation.yml@",
            source,
            "product evidence shaping must not consume a mutable central workflow reference",
        )


if __name__ == "__main__":
    unittest.main()
