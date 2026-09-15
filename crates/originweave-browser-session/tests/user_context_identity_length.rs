use originweave_browser_session::DisposableIsolationId;

#[test]
fn webdriver_bidi_user_context_preserves_protocol_text_without_domain_grammar() {
    let cases = [
        String::new(),
        " context ".to_owned(),
        "ctx\n".to_owned(),
        "u".repeat(4097),
    ];

    for remote_user_context in cases {
        let identity = DisposableIsolationId::parse(&remote_user_context).expect(
            "WebDriver BiDi browser.UserContext is CDDL text; Browser Session must preserve the exact remote identity",
        );

        assert_eq!(identity.as_str(), remote_user_context);
    }
}
