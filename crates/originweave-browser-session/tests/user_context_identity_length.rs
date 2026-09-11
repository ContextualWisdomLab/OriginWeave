use originweave_browser_session::DisposableIsolationId;

#[test]
fn webdriver_bidi_user_context_is_not_rejected_by_an_arbitrary_domain_length_cap() {
    let remote_user_context = "u".repeat(4097);

    let identity = DisposableIsolationId::parse(&remote_user_context)
        .expect("WebDriver BiDi browser.UserContext has no 4096-byte protocol limit");

    assert_eq!(identity.as_str(), remote_user_context);
}
