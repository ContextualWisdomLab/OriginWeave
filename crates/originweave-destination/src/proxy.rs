//! Explicit proxy and PAC route authority without network side effects.
//!
//! This module decides only whether a preselected route is authorized. It does
//! not resolve names, evaluate PAC JavaScript, open sockets, authenticate to a
//! proxy, issue CONNECT, or authenticate the final target. Proxy and final
//! destinations still require their own resolved-destination, transport, and TLS
//! authority boundaries.

use std::collections::BTreeSet;
use std::fmt;

use originweave_core::Origin;

/// Maximum byte length accepted for one proxy server identifier.
pub const MAX_PROXY_SERVER_IDENTIFIER_BYTES: usize = 512;
/// Maximum number of explicit proxy servers retained by one route policy.
pub const MAX_PROXY_SERVER_COUNT: usize = 32;
/// Maximum number of PAC source origins retained by one route policy.
pub const MAX_PAC_ORIGIN_COUNT: usize = 16;

/// Chromium-compatible protocol used to communicate with one proxy server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProxyServerScheme {
    /// Cleartext HTTP proxy transport.
    Http,
    /// TLS-protected HTTP proxy transport.
    Https,
    /// SOCKS version 4 proxy transport.
    Socks4,
    /// SOCKS version 5 proxy transport.
    Socks5,
    /// QUIC proxy transport.
    Quic,
}

impl ProxyServerScheme {
    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks4 => "socks4",
            Self::Socks5 => "socks5",
            Self::Quic => "quic",
        }
    }

    const fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https | Self::Quic => 443,
            Self::Socks4 | Self::Socks5 => 1080,
        }
    }
}

/// Canonical identity of one explicitly configured Chromium proxy server.
///
/// Unlike [`Origin`], a proxy-server identity may legitimately use ordinary
/// cleartext HTTP, SOCKS4, SOCKS5, or QUIC. The parser therefore reuses the
/// Origin authority validator without inheriting the web-origin rule that
/// remote HTTP origins are forbidden.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProxyServer {
    canonical: String,
    scheme: ProxyServerScheme,
}

impl ProxyServer {
    /// Parse a Chromium proxy server identifier.
    ///
    /// Supported explicit schemes are HTTP, HTTPS, SOCKS4, SOCKS5 (`socks` is
    /// accepted as Chromium's SOCKS5 alias), and QUIC. When the scheme is
    /// omitted, Chromium-compatible URI-form input defaults to HTTP. Credentials,
    /// paths, queries, fragments, zero or invalid ports, malformed authorities,
    /// oversized identifiers, and browser-special numeric host spellings fail
    /// closed before canonicalization.
    pub fn parse(input: &str) -> Result<Self, ProxyServerError> {
        if input.len() > MAX_PROXY_SERVER_IDENTIFIER_BYTES
            || input.trim() != input
            || input
                .chars()
                .any(|character| character.is_control() || character.is_whitespace())
        {
            return Err(ProxyServerError::InvalidIdentifier);
        }

        let (scheme, authority) = if let Some((raw_scheme, authority)) = input.split_once("://") {
            let scheme = match raw_scheme.to_ascii_lowercase().as_str() {
                "http" => ProxyServerScheme::Http,
                "https" => ProxyServerScheme::Https,
                "socks4" => ProxyServerScheme::Socks4,
                "socks" | "socks5" => ProxyServerScheme::Socks5,
                "quic" => ProxyServerScheme::Quic,
                _ => return Err(ProxyServerError::InvalidIdentifier),
            };
            (scheme, authority)
        } else {
            (ProxyServerScheme::Http, input)
        };
        if authority.is_empty() {
            return Err(ProxyServerError::InvalidIdentifier);
        }

        let port = explicit_port(authority)?;
        let validation_input = format!("https://{authority}");
        let validated = Origin::parse(&validation_input)
            .map_err(|_error| ProxyServerError::InvalidIdentifier)?;
        let host = if validated.host().contains(':') {
            format!("[{}]", validated.host())
        } else {
            validated.host().to_owned()
        };
        let canonical = match port {
            Some(port_number) if port_number != scheme.default_port() => {
                format!("{}://{host}:{port_number}", scheme.canonical_name())
            }
            _ => format!("{}://{host}", scheme.canonical_name()),
        };

        Ok(Self { canonical, scheme })
    }

    /// Return the canonical URI-form proxy server identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Return the proxy protocol encoded by this server identifier.
    #[must_use]
    pub const fn scheme(&self) -> ProxyServerScheme {
        self.scheme
    }
}

impl fmt::Display for ProxyServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Reason a proxy server identifier could not enter the routing boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyServerError {
    /// The identifier was malformed, ambiguous, credential-bearing, or used an unsupported scheme.
    InvalidIdentifier,
}

impl fmt::Display for ProxyServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid proxy server identifier")
    }
}

impl std::error::Error for ProxyServerError {}

/// A route selected by a trusted adapter before network I/O begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyRoute {
    /// Connect directly to the final target.
    Direct,
    /// Connect through one explicitly selected proxy server.
    ExplicitProxy {
        /// Canonical identity of the selected proxy server.
        proxy_server: ProxyServer,
    },
    /// A separately authorized PAC source selected a direct route.
    PacDirect {
        /// Canonical origin from which the PAC policy was obtained.
        pac_origin: Origin,
    },
    /// A separately authorized PAC source selected an explicit proxy.
    PacProxy {
        /// Canonical origin from which the PAC policy was obtained.
        pac_origin: Origin,
        /// Canonical identity of the proxy server selected by the PAC result.
        proxy_server: ProxyServer,
    },
}

/// Stable route classification retained in authorization evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyRouteKind {
    /// A direct route with no proxy or PAC source.
    Direct,
    /// An explicitly configured proxy route.
    ExplicitProxy,
    /// A PAC source selected `DIRECT`.
    PacDirect,
    /// A PAC source selected an explicit proxy.
    PacProxy,
}

/// Fail-closed authority for direct, proxy, and PAC-selected routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRoutePolicy {
    allow_direct: bool,
    proxy_servers: BTreeSet<ProxyServer>,
    pac_origins: BTreeSet<Origin>,
}

impl ProxyRoutePolicy {
    /// Construct the default route policy, which permits only direct routing.
    #[must_use]
    pub fn direct_only() -> Self {
        Self {
            allow_direct: true,
            proxy_servers: BTreeSet::new(),
            pac_origins: BTreeSet::new(),
        }
    }

    /// Construct an explicit bounded route policy from canonical authorities.
    ///
    /// Proxy servers and PAC sources are accepted only as already validated
    /// [`ProxyServer`] and [`Origin`] values. This policy does not grant
    /// destination, TCP, or TLS authority to any listed server or origin.
    pub fn new(
        allow_direct: bool,
        proxy_servers: Vec<ProxyServer>,
        pac_origins: Vec<Origin>,
    ) -> Result<Self, ProxyRouteError> {
        Ok(Self {
            allow_direct,
            proxy_servers: collect_proxy_servers(proxy_servers)?,
            pac_origins: collect_pac_origins(pac_origins)?,
        })
    }

    /// Return whether this policy explicitly authorizes direct routing.
    #[must_use]
    pub const fn allows_direct(&self) -> bool {
        self.allow_direct
    }

    /// Authorize one already selected route for the exact target origin.
    ///
    /// The returned evidence records the routing decision only. A caller must
    /// independently authorize and authenticate every network destination that
    /// the selected route will actually use.
    pub fn authorize(
        &self,
        target_origin: &Origin,
        route: &ProxyRoute,
    ) -> Result<ProxyRouteEvidence, ProxyRouteError> {
        match route {
            ProxyRoute::Direct => {
                self.require_direct()?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::Direct,
                    None,
                    None,
                ))
            }
            ProxyRoute::ExplicitProxy { proxy_server } => {
                self.require_proxy(proxy_server)?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::ExplicitProxy,
                    Some(proxy_server),
                    None,
                ))
            }
            ProxyRoute::PacDirect { pac_origin } => {
                self.require_pac(pac_origin)?;
                self.require_direct()?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::PacDirect,
                    None,
                    Some(pac_origin),
                ))
            }
            ProxyRoute::PacProxy {
                pac_origin,
                proxy_server,
            } => {
                self.require_pac(pac_origin)?;
                self.require_proxy(proxy_server)?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::PacProxy,
                    Some(proxy_server),
                    Some(pac_origin),
                ))
            }
        }
    }

    fn require_direct(&self) -> Result<(), ProxyRouteError> {
        if self.allow_direct {
            Ok(())
        } else {
            Err(ProxyRouteError::DirectRouteDenied)
        }
    }

    fn require_proxy(&self, server: &ProxyServer) -> Result<(), ProxyRouteError> {
        if self.proxy_servers.contains(server) {
            Ok(())
        } else {
            Err(ProxyRouteError::ProxyServerDenied {
                server: server.clone(),
            })
        }
    }

    fn require_pac(&self, origin: &Origin) -> Result<(), ProxyRouteError> {
        if self.pac_origins.contains(origin) {
            Ok(())
        } else {
            Err(ProxyRouteError::PacOriginDenied {
                origin: origin.clone(),
            })
        }
    }
}

impl Default for ProxyRoutePolicy {
    fn default() -> Self {
        Self::direct_only()
    }
}

/// Immutable credential-free evidence for one route authorization decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRouteEvidence {
    target_origin: Origin,
    route_kind: ProxyRouteKind,
    proxy_server: Option<ProxyServer>,
    pac_origin: Option<Origin>,
}

impl ProxyRouteEvidence {
    fn new(
        target_origin: &Origin,
        route_kind: ProxyRouteKind,
        proxy_server: Option<&ProxyServer>,
        pac_origin: Option<&Origin>,
    ) -> Self {
        Self {
            target_origin: target_origin.clone(),
            route_kind,
            proxy_server: proxy_server.cloned(),
            pac_origin: pac_origin.cloned(),
        }
    }

    /// Return the exact canonical target origin for which routing was approved.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the authorized route classification.
    #[must_use]
    pub const fn route_kind(&self) -> ProxyRouteKind {
        self.route_kind
    }

    /// Return the exact selected proxy server when the route uses a proxy.
    #[must_use]
    pub const fn proxy_server(&self) -> Option<&ProxyServer> {
        self.proxy_server.as_ref()
    }

    /// Return the exact PAC source origin when PAC selected the route.
    #[must_use]
    pub const fn pac_origin(&self) -> Option<&Origin> {
        self.pac_origin.as_ref()
    }
}

/// Reason an explicit direct, proxy, or PAC route could not be authorized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyRouteError {
    /// The selected route requires direct routing but the policy denies it.
    DirectRouteDenied,
    /// The selected proxy server was not in the exact canonical allow-list.
    ProxyServerDenied {
        /// Canonical proxy server that was denied.
        server: ProxyServer,
    },
    /// The PAC source was not in the exact canonical PAC-source allow-list.
    PacOriginDenied {
        /// Canonical PAC source origin that was denied.
        origin: Origin,
    },
    /// More proxy servers were supplied than the policy permits.
    TooManyProxyServers {
        /// Number of supplied servers observed before rejecting construction.
        count: usize,
        /// Maximum number of proxy servers accepted by the policy.
        maximum: usize,
    },
    /// More PAC source origins were supplied than the policy permits.
    TooManyPacOrigins {
        /// Number of supplied origins observed before rejecting construction.
        count: usize,
        /// Maximum number of PAC source origins accepted by the policy.
        maximum: usize,
    },
}

impl fmt::Display for ProxyRouteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectRouteDenied => formatter.write_str("direct proxy route is not authorized"),
            Self::ProxyServerDenied { server } => {
                write!(formatter, "proxy server is not authorized: {server}")
            }
            Self::PacOriginDenied { origin } => {
                write!(formatter, "PAC origin is not authorized: {origin}")
            }
            Self::TooManyProxyServers { count, maximum } => write!(
                formatter,
                "proxy server policy has {count} entries; maximum is {maximum}"
            ),
            Self::TooManyPacOrigins { count, maximum } => write!(
                formatter,
                "PAC origin policy has {count} entries; maximum is {maximum}"
            ),
        }
    }
}

impl std::error::Error for ProxyRouteError {}

fn explicit_port(authority: &str) -> Result<Option<u16>, ProxyServerError> {
    let port_text = if authority.starts_with('[') {
        let close_index = authority
            .find(']')
            .ok_or(ProxyServerError::InvalidIdentifier)?;
        let remainder = &authority[close_index + 1..];
        if remainder.is_empty() {
            return Ok(None);
        }
        remainder
            .strip_prefix(':')
            .ok_or(ProxyServerError::InvalidIdentifier)?
    } else {
        let Some((_host, port)) = authority.rsplit_once(':') else {
            return Ok(None);
        };
        port
    };

    let port = port_text
        .parse::<u16>()
        .map_err(|_error| ProxyServerError::InvalidIdentifier)?;
    if port == 0 {
        return Err(ProxyServerError::InvalidIdentifier);
    }
    Ok(Some(port))
}

fn collect_proxy_servers(
    servers: Vec<ProxyServer>,
) -> Result<BTreeSet<ProxyServer>, ProxyRouteError> {
    let mut collected = BTreeSet::new();
    let mut count = 0usize;
    for server in servers {
        count += 1;
        if count > MAX_PROXY_SERVER_COUNT {
            return Err(ProxyRouteError::TooManyProxyServers {
                count,
                maximum: MAX_PROXY_SERVER_COUNT,
            });
        }
        collected.insert(server);
    }
    Ok(collected)
}

fn collect_pac_origins(origins: Vec<Origin>) -> Result<BTreeSet<Origin>, ProxyRouteError> {
    let mut collected = BTreeSet::new();
    let mut count = 0usize;
    for origin in origins {
        count += 1;
        if count > MAX_PAC_ORIGIN_COUNT {
            return Err(ProxyRouteError::TooManyPacOrigins {
                count,
                maximum: MAX_PAC_ORIGIN_COUNT,
            });
        }
        collected.insert(origin);
    }
    Ok(collected)
}
