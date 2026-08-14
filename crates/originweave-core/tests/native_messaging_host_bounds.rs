#![allow(clippy::expect_used)]

use originweave_core::{NativeMessagingHostName, NativeMessagingHostNameError};

#[test]
fn native_messaging_host_name_is_bounded_before_it_becomes_authority() {
    let exact_limit = "a".repeat(256);
    assert_eq!(
        NativeMessagingHostName::parse(&exact_limit)
            .expect("the exact local authority bound remains accepted")
            .as_str(),
        exact_limit
    );

    let one_over = "a".repeat(257);
    assert_eq!(
        NativeMessagingHostName::parse(&one_over),
        Err(NativeMessagingHostNameError::InvalidHostName)
    );
}
