use std::error::Error;

use originweave_core::{
    BoundedWebDriverBiDiResponseDocument, MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES,
    WebDriverBiDiResponseDocumentAdmissionError,
};

#[test]
fn bounded_response_document_retains_exact_wire_text() -> Result<(), Box<dyn Error>> {
    let raw = " \r\n{\"id\":42,\"type\":\"success\",\"result\":{}}\t";
    let document = BoundedWebDriverBiDiResponseDocument::new(raw)?;

    assert_eq!(document.as_str(), raw);
    Ok(())
}

#[test]
fn empty_or_json_whitespace_only_response_document_fails_closed() {
    for raw in ["", " ", "\t\r\n"] {
        assert_eq!(
            BoundedWebDriverBiDiResponseDocument::new(raw),
            Err(WebDriverBiDiResponseDocumentAdmissionError::EmptyDocument)
        );
    }
}

#[test]
fn response_document_requires_an_object_boundary_without_claiming_json_validation() {
    for raw in ["[]", "null", "{", "}", "\u{00a0}{}\u{00a0}"] {
        assert_eq!(
            BoundedWebDriverBiDiResponseDocument::new(raw),
            Err(WebDriverBiDiResponseDocumentAdmissionError::InvalidObjectBoundary)
        );
    }

    let coarse_only = BoundedWebDriverBiDiResponseDocument::new("{not-json}")
        .expect("coarse admission intentionally does not parse JSON");
    assert_eq!(coarse_only.as_str(), "{not-json}");
}

#[test]
fn response_document_budget_accepts_exact_limit_and_rejects_one_more_byte() -> Result<(), Box<dyn Error>> {
    const OBJECT_OVERHEAD_BYTES: usize = 8;
    let exact = format!(
        "{{\"x\":\"{}\"}}",
        "a".repeat(MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES - OBJECT_OVERHEAD_BYTES)
    );
    assert_eq!(exact.len(), MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES);
    assert!(BoundedWebDriverBiDiResponseDocument::new(&exact).is_ok());

    let oversized = format!("{exact} ");
    assert_eq!(oversized.len(), MAX_WEBDRIVER_BIDI_RESPONSE_DOCUMENT_BYTES + 1);
    assert_eq!(
        BoundedWebDriverBiDiResponseDocument::new(&oversized),
        Err(WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge)
    );
    Ok(())
}

#[test]
fn response_document_errors_are_deterministic_and_source_free() {
    for error in [
        WebDriverBiDiResponseDocumentAdmissionError::EmptyDocument,
        WebDriverBiDiResponseDocumentAdmissionError::DocumentTooLarge,
        WebDriverBiDiResponseDocumentAdmissionError::InvalidObjectBoundary,
    ] {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
}
