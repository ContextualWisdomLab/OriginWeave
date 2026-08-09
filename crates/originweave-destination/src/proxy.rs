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

/// Maximum number of explicit proxy origins retained by one route policy.
pub const MAX_PROXY_ORIGIN_COUNT: usize = 32;
/// Maximum number of PAC source origins retained by one route policy.
pub const MAX_PAC_ORIGIN_COUNT: usize = 16;

/// A route selected by a trusted adapter before network I/O begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyRoute {
    /// Connect directly to the final target.
    Direct,
    /// Connect through one explicitly selected proxy origin.
    ExplicitProxy {
        /// Canonical origin of the selected proxy.
        proxy_origin: Origin,
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
        /// Canonical origin of the proxy selected by the PAC result.
        proxy_origin: Origin,
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
    proxy_origins: BTreeSet<Origin>,
    pac_origins: BTreeSet<Origin>,
}

impl ProxyRoutePolicy {
    /// Construct the default route policy, which permits only direct routing.
    #[must_use]
    pub fn direct_only() -> Self {
        Self {
            allow_direct: true,
            proxy_origins: BTreeSet::new(),
            pac_origins: BTreeSet::new(),
        }
    }

    /// Construct an explicit bounded route policy from owned canonical origins.
    ///
    /// Proxy and PAC sources are accepted only as already validated canonical
    /// [`Origin`] values. This policy does not grant destination, TCP, or TLS
    /// authority to any listed origin. Owned vectors keep the constructor's
    /// executable coverage and generated code independent of caller iterator
    /// types while the policy consumes the supplied authority set.
    pub fn new(
        allow_direct: bool,
        proxy_origins: Vec<Origin>,
        pac_origins: Vec<Origin>,
    ) -> Result<Self, ProxyRouteError> {
        Ok(Self {
            allow_direct,
            proxy_origins: collect_proxy_origins(proxy_origins)?,
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
            ProxyRoute::ExplicitProxy { proxy_origin } => {
                self.require_proxy(proxy_origin)?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::ExplicitProxy,
                    Some(proxy_origin),
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
                proxy_origin,
            } => {
                self.require_pac(pac_origin)?;
                self.require_proxy(proxy_origin)?;
                Ok(ProxyRouteEvidence::new(
                    target_origin,
                    ProxyRouteKind::PacProxy,
                    Some(proxy_origin),
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

    fn require_proxy(&self, origin: &Origin) -> Result<(), ProxyRouteError> {
        if self.proxy_origins.contains(origin) {
            Ok(())
        } else {
            Err(ProxyRouteError::ProxyOriginDenied {
                origin: origin.clone(),
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
    proxy_origin: Option<Origin>,
    pac_origin: Option<Origin>,
}

impl ProxyRouteEvidence {
    fn new(
        target_origin: &Origin,
        route_kind: ProxyRouteKind,
        proxy_origin: Option<&Origin>,
        pac_origin: Option<&Origin>,
    ) -> Self {
        Self {
            target_origin: target_origin.clone(),
            route_kind,
            proxy_origin: proxy_origin.cloned(),
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

    /// Return the exact selected proxy origin when the route uses a proxy.
    #[must_use]
    pub const fn proxy_origin(&self) -> Option<&Origin> {
        self.proxy_origin.as_ref()
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
    /// The selected proxy was not in the exact canonical proxy allow-list.
    ProxyOriginDenied {
        /// Canonical proxy origin that was denied.
        origin: Origin,
    },
    /// The PAC source was not in the exact canonical PAC-source allow-list.
    PacOriginDenied {
        /// Canonical PAC source origin that was denied.
        origin: Origin,
    },
    /// More proxy origins were supplied than the policy permits.
    TooManyProxyOrigins {
        /// Number of supplied origins observed before rejecting construction.
        count: usize,
        /// Maximum number of proxy origins accepted by the policy.
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
            Self::ProxyOriginDenied { origin } => {
                write!(formatter, "proxy origin is not authorized: {origin}")
            }
            Self::PacOriginDenied { origin } => {
                write!(formatter, "PAC origin is not authorized: {origin}")
            }
            Self::TooManyProxyOrigins { count, maximum } => write!(
                formatter,
                "proxy origin policy has {count} entries; maximum is {maximum}"
            ),
            Self::TooManyPacOrigins { count, maximum } => write!(
                formatter,
                "PAC origin policy has {count} entries; maximum is {maximum}"
            ),
        }
    }
}

impl std::error::Error for ProxyRouteError {}

fn collect_proxy_origins(origins: Vec<Origin>) -> Result<BTreeSet<Origin>, ProxyRouteError> {
    let mut collected = BTreeSet::new();
    let mut count = 0usize;
    for origin in origins {
        count += 1;
        if count > MAX_PROXY_ORIGIN_COUNT {
            return Err(ProxyRouteError::TooManyProxyOrigins {
                count,
                maximum: MAX_PROXY_ORIGIN_COUNT,
            });
        }
        collected.insert(origin);
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
