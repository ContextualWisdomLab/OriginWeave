"""Regression contract for credential-safe HTTP response debugging."""

from pathlib import Path


EVIDENCE_SOURCE = Path("crates/originweave-http/src/evidence.rs")


def test_authenticated_http_response_debug_is_length_only() -> None:
    """Debug output must not expose remote body or reason-phrase octets."""

    source = EVIDENCE_SOURCE.read_text(encoding="utf-8")
    marker = "pub struct AuthenticatedHttpResponse"
    start = source.index(marker)
    response_section = source[start:]

    assert "#[derive(Debug)]\npub struct AuthenticatedHttpResponse" not in source
    assert "impl fmt::Debug for AuthenticatedHttpResponse" in response_section
    assert '.field("content_byte_count", &self.content.len())' in response_section
    assert (
        '.field("reason_phrase_byte_count", &self.reason_phrase.len())'
        in response_section
    )
    assert '.field("content", &self.content)' not in response_section
    assert '.field("reason_phrase", &self.reason_phrase)' not in response_section
