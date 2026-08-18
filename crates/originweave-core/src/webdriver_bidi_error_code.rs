/// Typed current WebDriver BiDi protocol error code retained from one validated error response.
///
/// This vocabulary is deliberately closed over the protocol error codes reviewed by OriginWeave.
/// Unknown wire text remains fail-closed and cannot become typed protocol evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiErrorCode {
    /// The command or one of its arguments is invalid.
    InvalidArgument,
    /// A selector argument is invalid.
    InvalidSelector,
    /// The referenced browser session does not exist.
    InvalidSessionId,
    /// The referenced web extension is invalid.
    InvalidWebExtension,
    /// A requested pointer move target is outside the allowed bounds.
    MoveTargetOutOfBounds,
    /// The referenced user prompt does not exist.
    NoSuchAlert,
    /// The referenced client window does not exist.
    NoSuchClientWindow,
    /// The referenced network collector does not exist.
    NoSuchNetworkCollector,
    /// The referenced element does not exist.
    NoSuchElement,
    /// The referenced frame does not exist.
    NoSuchFrame,
    /// The referenced handle does not exist.
    NoSuchHandle,
    /// The referenced history entry does not exist.
    NoSuchHistoryEntry,
    /// The referenced network intercept does not exist.
    NoSuchIntercept,
    /// The requested network data does not exist.
    NoSuchNetworkData,
    /// The referenced node does not exist.
    NoSuchNode,
    /// The referenced network request does not exist.
    NoSuchRequest,
    /// The referenced screencast does not exist.
    NoSuchScreencast,
    /// The referenced script does not exist.
    NoSuchScript,
    /// The referenced storage partition does not exist.
    NoSuchStoragePartition,
    /// The referenced user context does not exist.
    NoSuchUserContext,
    /// The referenced web extension does not exist.
    NoSuchWebExtension,
    /// A browser session could not be created.
    SessionNotCreated,
    /// The browser could not capture the requested screen image.
    UnableToCaptureScreen,
    /// The browser could not close as requested.
    UnableToCloseBrowser,
    /// The browser could not set the requested cookie.
    UnableToSetCookie,
    /// The browser could not set the requested file input.
    UnableToSetFileInput,
    /// Requested network data is temporarily unavailable.
    UnavailableNetworkData,
    /// The supplied storage-partition descriptor is underspecified.
    UnderspecifiedStoragePartition,
    /// The command is unknown to the remote end.
    UnknownCommand,
    /// The remote end reported an otherwise unclassified protocol error.
    UnknownError,
    /// The requested operation is unsupported by the remote end.
    UnsupportedOperation,
}

/// Parse one exact decoded WebDriver BiDi `ErrorCode` value into typed protocol evidence.
pub(crate) fn parse_webdriver_bidi_error_code(value: &[u8]) -> Option<WebDriverBiDiErrorCode> {
    const ERROR_CODES: &[(&[u8], WebDriverBiDiErrorCode)] = &[
        (b"invalid argument", WebDriverBiDiErrorCode::InvalidArgument),
        (b"invalid selector", WebDriverBiDiErrorCode::InvalidSelector),
        (b"invalid session id", WebDriverBiDiErrorCode::InvalidSessionId),
        (b"invalid web extension", WebDriverBiDiErrorCode::InvalidWebExtension),
        (
            b"move target out of bounds",
            WebDriverBiDiErrorCode::MoveTargetOutOfBounds,
        ),
        (b"no such alert", WebDriverBiDiErrorCode::NoSuchAlert),
        (
            b"no such client window",
            WebDriverBiDiErrorCode::NoSuchClientWindow,
        ),
        (
            b"no such network collector",
            WebDriverBiDiErrorCode::NoSuchNetworkCollector,
        ),
        (b"no such element", WebDriverBiDiErrorCode::NoSuchElement),
        (b"no such frame", WebDriverBiDiErrorCode::NoSuchFrame),
        (b"no such handle", WebDriverBiDiErrorCode::NoSuchHandle),
        (
            b"no such history entry",
            WebDriverBiDiErrorCode::NoSuchHistoryEntry,
        ),
        (b"no such intercept", WebDriverBiDiErrorCode::NoSuchIntercept),
        (
            b"no such network data",
            WebDriverBiDiErrorCode::NoSuchNetworkData,
        ),
        (b"no such node", WebDriverBiDiErrorCode::NoSuchNode),
        (b"no such request", WebDriverBiDiErrorCode::NoSuchRequest),
        (b"no such screencast", WebDriverBiDiErrorCode::NoSuchScreencast),
        (b"no such script", WebDriverBiDiErrorCode::NoSuchScript),
        (
            b"no such storage partition",
            WebDriverBiDiErrorCode::NoSuchStoragePartition,
        ),
        (
            b"no such user context",
            WebDriverBiDiErrorCode::NoSuchUserContext,
        ),
        (
            b"no such web extension",
            WebDriverBiDiErrorCode::NoSuchWebExtension,
        ),
        (
            b"session not created",
            WebDriverBiDiErrorCode::SessionNotCreated,
        ),
        (
            b"unable to capture screen",
            WebDriverBiDiErrorCode::UnableToCaptureScreen,
        ),
        (
            b"unable to close browser",
            WebDriverBiDiErrorCode::UnableToCloseBrowser,
        ),
        (b"unable to set cookie", WebDriverBiDiErrorCode::UnableToSetCookie),
        (
            b"unable to set file input",
            WebDriverBiDiErrorCode::UnableToSetFileInput,
        ),
        (
            b"unavailable network data",
            WebDriverBiDiErrorCode::UnavailableNetworkData,
        ),
        (
            b"underspecified storage partition",
            WebDriverBiDiErrorCode::UnderspecifiedStoragePartition,
        ),
        (b"unknown command", WebDriverBiDiErrorCode::UnknownCommand),
        (b"unknown error", WebDriverBiDiErrorCode::UnknownError),
        (
            b"unsupported operation",
            WebDriverBiDiErrorCode::UnsupportedOperation,
        ),
    ];

    ERROR_CODES
        .iter()
        .find_map(|(raw, code)| (*raw == value).then_some(*code))
}
