use std::fmt;

use crate::webdriver_bidi_websocket_handshake_raw as raw;

/// Caller-supplied RFC 6455 mask key for one client-to-server frame.
///
/// RFC 6455 requires every client frame to carry a fresh, unpredictable four-byte key. This
/// public wrapper keeps those bytes available only to the framing boundary while ensuring generic
/// diagnostics cannot render the masking entropy. Callers remain responsible for obtaining a fresh
/// key from an approved randomness source for every client frame.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct WebDriverBiDiWebSocketMaskKey(raw::WebDriverBiDiWebSocketMaskKey);

impl fmt::Debug for WebDriverBiDiWebSocketMaskKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted WebSocket masking key>")
    }
}

impl WebDriverBiDiWebSocketMaskKey {
    /// Admit one four-byte caller-supplied frame masking key.
    #[must_use]
    pub const fn new(value: [u8; 4]) -> Self {
        Self(raw::WebDriverBiDiWebSocketMaskKey::new(value))
    }

    /// Borrow the exact four-byte key for the reviewed wire-framing boundary.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 4] {
        self.0.as_bytes()
    }

    pub(crate) const fn into_raw(self) -> raw::WebDriverBiDiWebSocketMaskKey {
        self.0
    }
}
