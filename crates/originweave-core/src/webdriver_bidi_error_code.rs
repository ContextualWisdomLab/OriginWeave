/// Returns whether `value` is one of the error codes admitted by the current WebDriver BiDi specification.
pub(crate) fn is_webdriver_bidi_error_code(value: &[u8]) -> bool {
    const ERROR_CODES: &[&[u8]] = &[
        b"invalid argument",
        b"invalid selector",
        b"invalid session id",
        b"invalid web extension",
        b"move target out of bounds",
        b"no such alert",
        b"no such client window",
        b"no such network collector",
        b"no such element",
        b"no such frame",
        b"no such handle",
        b"no such history entry",
        b"no such intercept",
        b"no such network data",
        b"no such node",
        b"no such request",
        b"no such screencast",
        b"no such script",
        b"no such storage partition",
        b"no such user context",
        b"no such web extension",
        b"session not created",
        b"unable to capture screen",
        b"unable to close browser",
        b"unable to set cookie",
        b"unable to set file input",
        b"unavailable network data",
        b"underspecified storage partition",
        b"unknown command",
        b"unknown error",
        b"unsupported operation",
    ];

    ERROR_CODES.contains(&value)
}
