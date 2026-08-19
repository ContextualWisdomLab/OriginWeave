use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

/// Maximum admitted bytes for one WebDriver BiDi WebSocket endpoint.
///
/// This is an OriginWeave first-Chromium-fixture safety budget, not a
/// WebDriver BiDi protocol maximum.
pub const MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES: usize = 2_048;

/// One bounded canonical WebDriver BiDi session WebSocket endpoint.
///
/// This value is transport metadata only. Construction does not authenticate
/// Chromium, ChromeDriver, the operating-system peer, TLS, policy, or Agent
/// authority. The first real-Chromium fixture intentionally admits only
/// loopback listener identities; the connection boundary must still verify the
/// actual peer before exposing transport I/O.
#[derive(Debug, PartialEq, Eq)]
pub struct WebDriverBiDiWebSocketEndpoint {
    endpoint: String,
    secure: bool,
    host: String,
    port: u16,
    session_id: String,
}

impl WebDriverBiDiWebSocketEndpoint {
    /// Admit one bounded canonical first-fixture WebDriver BiDi endpoint.
    pub fn new(value: &str) -> Result<Self, WebDriverBiDiWebSocketEndpointAdmissionError> {
        if value.is_empty() {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::EmptyEndpoint);
        }
        if value.len() > MAX_WEBDRIVER_BIDI_WEBSOCKET_ENDPOINT_BYTES {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::EndpointTooLong);
        }
        if value.bytes().any(|byte| !byte.is_ascii_graphic()) {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidEndpointText);
        }
        if value.bytes().any(|byte| matches!(byte, b'?' | b'#')) {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::QueryOrFragmentForbidden);
        }

        let (secure, remainder) = if let Some(remainder) = value.strip_prefix("ws://") {
            (false, remainder)
        } else if let Some(remainder) = value.strip_prefix("wss://") {
            (true, remainder)
        } else {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidScheme);
        };

        let Some(path_start) = remainder.find('/') else {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource);
        };
        let authority = &remainder[..path_start];
        let resource = &remainder[path_start..];
        if authority.is_empty() || authority.contains('@') {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
        }

        let (host, port_text) = if let Some(bracketed) = authority.strip_prefix('[') {
            let Some(close) = bracketed.find(']') else {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            };
            let host_text = &bracketed[..close];
            let suffix = &bracketed[close + 1..];
            let Some(port_text) = suffix.strip_prefix(':') else {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            };
            if port_text.is_empty() {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            }
            let Ok(ip) = host_text.parse::<Ipv6Addr>() else {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            };
            if !ip.is_loopback() {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost);
            }
            if ip.to_string() != host_text {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            }
            (host_text.to_owned(), port_text)
        } else {
            let Some((host_text, port_text)) = authority.rsplit_once(':') else {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            };
            if host_text.is_empty() || port_text.is_empty() || host_text.contains(':') {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            }
            if host_text == "localhost" {
                (host_text.to_owned(), port_text)
            } else if let Ok(ip) = host_text.parse::<Ipv4Addr>() {
                if !ip.is_loopback() {
                    return Err(WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost);
                }
                (host_text.to_owned(), port_text)
            } else if host_text
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
            {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::NonLoopbackHost);
            } else {
                return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidAuthority);
            }
        };

        let Ok(port) = port_text.parse::<u16>() else {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidPort);
        };
        if port == 0 || port.to_string() != port_text {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidPort);
        }

        let Some(session_id) = resource.strip_prefix("/session/") else {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource);
        };
        if session_id.is_empty() || session_id.contains('/') {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionResource);
        }
        if !is_canonical_session_id(session_id) {
            return Err(WebDriverBiDiWebSocketEndpointAdmissionError::InvalidSessionId);
        }

        Ok(Self {
            endpoint: value.to_owned(),
            secure,
            host,
            port,
            session_id: session_id.to_owned(),
        })
    }

    /// Return the exact admitted endpoint text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.endpoint
    }

    /// Return whether the endpoint uses `wss` rather than `ws`.
    #[must_use]
    pub const fn is_secure(&self) -> bool {
        self.secure
    }

    /// Return the canonical loopback listener host without IPv6 brackets.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Return the explicit nonzero listener port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Return the exact canonical session identifier admitted from the WebDriver endpoint.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

fn is_lowercase_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn is_canonical_session_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    match bytes.len() {
        32 => bytes.iter().copied().all(is_lowercase_hex),
        36 => {
            for (index, byte) in bytes.iter().copied().enumerate() {
                let valid = if matches!(index, 8 | 13 | 18 | 23) {
                    byte == b'-'
                } else {
                    is_lowercase_hex(byte)
                };
                if !valid {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Fail-closed admission errors for WebDriver BiDi WebSocket endpoint metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebDriverBiDiWebSocketEndpointAdmissionError {
    /// The endpoint text is empty.
    EmptyEndpoint,
    /// The endpoint text exceeds the OriginWeave safety budget.
    EndpointTooLong,
    /// The endpoint contains non-ASCII, whitespace, or control text.
    InvalidEndpointText,
    /// The endpoint does not use the exact `ws` or `wss` scheme.
    InvalidScheme,
    /// Query or fragment data is present and therefore not part of the admitted session resource.
    QueryOrFragmentForbidden,
    /// The authority is absent, credential-bearing, ambiguous, or malformed.
    InvalidAuthority,
    /// The authority identifies a non-loopback host.
    NonLoopbackHost,
    /// The port is absent, zero, out of range, or not canonically serialized.
    InvalidPort,
    /// The path is not exactly one `/session/<session id>` resource.
    InvalidSessionResource,
    /// The session id is not an admitted canonical W3C/ChromeDriver representation.
    InvalidSessionId,
}

impl fmt::Display for WebDriverBiDiWebSocketEndpointAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyEndpoint => "WebDriver BiDi WebSocket endpoint is empty",
            Self::EndpointTooLong => "WebDriver BiDi WebSocket endpoint exceeds the safety budget",
            Self::InvalidEndpointText => {
                "WebDriver BiDi WebSocket endpoint text is not canonical ASCII"
            }
            Self::InvalidScheme => "WebDriver BiDi WebSocket endpoint scheme is not ws or wss",
            Self::QueryOrFragmentForbidden => {
                "WebDriver BiDi WebSocket endpoint query or fragment is forbidden"
            }
            Self::InvalidAuthority => "WebDriver BiDi WebSocket endpoint authority is invalid",
            Self::NonLoopbackHost => "WebDriver BiDi WebSocket endpoint host is not loopback",
            Self::InvalidPort => "WebDriver BiDi WebSocket endpoint port is invalid",
            Self::InvalidSessionResource => {
                "WebDriver BiDi WebSocket endpoint session resource is invalid"
            }
            Self::InvalidSessionId => "WebDriver BiDi WebSocket endpoint session id is invalid",
        };
        f.write_str(message)
    }
}

impl std::error::Error for WebDriverBiDiWebSocketEndpointAdmissionError {}
