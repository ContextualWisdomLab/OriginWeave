#![allow(clippy::expect_used)]

use std::io;

use crate::HttpError;

fn restoration_failure() -> HttpError {
    HttpError::TimeoutRestorationFailed {
        source: io::Error::other("expected restoration failure"),
    }
}

#[test]
fn primary_exchange_failure_precedes_timeout_restoration_failure() {
    let result = crate::exchange::combine_exchange_and_restoration(
        Err::<usize, _>(HttpError::IncompleteResponse),
        Err(restoration_failure()),
    );
    assert!(matches!(result, Err(HttpError::IncompleteResponse)));
}

#[test]
fn restoration_failure_is_reported_after_a_successful_exchange() {
    let result = crate::exchange::combine_exchange_and_restoration(
        Ok::<usize, HttpError>(42),
        Err(restoration_failure()),
    );
    assert!(matches!(result, Err(HttpError::TimeoutRestorationFailed { .. })));
}

#[test]
fn successful_exchange_and_restoration_preserve_the_response() {
    assert_eq!(
        crate::exchange::combine_exchange_and_restoration(Ok::<usize, HttpError>(42), Ok(()))
            .expect("successful exchange and restoration"),
        42
    );
}
