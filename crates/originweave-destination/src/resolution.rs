use std::collections::BTreeSet;
use std::fmt;
use std::net::IpAddr;
use std::time::Duration;

use originweave_core::Origin;

use crate::{AddressClass, ClassifiedAddress, classify_address};

/// The largest resolver answer accepted by one resolution snapshot.
pub const MAX_RESOLUTION_ADDRESS_COUNT: usize = 256;

/// The largest freshness interval accepted for one resolution approval.
///
/// This is an OriginWeave product safety budget, not a DNS protocol validity
/// rule. Callers may choose any smaller non-zero interval appropriate to their
/// resolver and network adapter.
pub const MAX_RESOLUTION_VALIDITY: Duration = Duration::from_secs(30);

/// A fail-closed allow-list of destination address classes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationPolicy {
    allowed_classes: BTreeSet<AddressClass>,
}

impl DestinationPolicy {
    /// Create the default web policy, which permits only public destinations.
    #[must_use]
    pub fn public_web() -> Self {
        Self::from_allowed_classes([AddressClass::Public])
    }

    /// Create a policy from explicitly permitted address classes.
    #[must_use]
    pub fn from_allowed_classes(allowed_classes: impl IntoIterator<Item = AddressClass>) -> Self {
        Self {
            allowed_classes: allowed_classes.into_iter().collect(),
        }
    }

    /// Return the explicitly permitted address classes.
    #[must_use]
    pub const fn allowed_classes(&self) -> &BTreeSet<AddressClass> {
        &self.allowed_classes
    }

    /// Return whether one address class is explicitly permitted.
    #[must_use]
    pub fn allows(&self, address_class: AddressClass) -> bool {
        self.allowed_classes.contains(&address_class)
    }

    /// Classify and validate one resolved address.
    pub fn validate_address(&self, address: IpAddr) -> Result<ClassifiedAddress, DestinationError> {
        let classified = classify_address(address);
        if self.allows(classified.address_class()) {
            Ok(classified)
        } else {
            Err(DestinationError::AddressClassDenied {
                address: classified.original_address(),
                address_class: classified.address_class(),
            })
        }
    }
}

impl Default for DestinationPolicy {
    fn default() -> Self {
        Self::public_web()
    }
}

/// A deterministic reason that destination authorization failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationError {
    /// DNS or the adapter supplied no resolved address.
    EmptyResolution,
    /// The resolver answer exceeded [`MAX_RESOLUTION_ADDRESS_COUNT`].
    ResolutionAddressLimitExceeded {
        /// The largest accepted address count.
        maximum_count: usize,
    },
    /// A resolved address belongs to a class not permitted by policy.
    AddressClassDenied {
        /// The address supplied by the resolver or adapter.
        address: IpAddr,
        /// The denied security class.
        address_class: AddressClass,
    },
    /// The special `localhost` name resolved outside loopback address space.
    LocalhostResolutionNotLoopback {
        /// The address supplied by the resolver or adapter.
        address: IpAddr,
        /// The security class assigned to the address.
        address_class: AddressClass,
    },
    /// A literal-IP origin resolved to a different canonical address.
    LiteralOriginAddressMismatch {
        /// The canonical address encoded directly in the logical origin.
        origin_address: IpAddr,
        /// The canonical address supplied by the resolver or adapter.
        resolved_address: IpAddr,
    },
    /// The connection candidate was absent from the pinned address set.
    UnapprovedConnectionAddress {
        /// The candidate address supplied for connection.
        address: IpAddr,
    },
    /// A refreshed DNS answer introduced an address outside the pinned set.
    ResolutionSetExpanded {
        /// The newly introduced canonical address.
        address: IpAddr,
    },
    /// A freshness interval was zero or exceeded [`MAX_RESOLUTION_VALIDITY`].
    InvalidResolutionValidity {
        /// The rejected freshness interval.
        validity: Duration,
        /// The largest accepted freshness interval.
        maximum_validity: Duration,
    },
    /// Adding the freshness interval to the approval time overflowed.
    ResolutionValidityOverflow {
        /// The trusted monotonic time at which the answer was approved.
        approved_at: Duration,
        /// The requested freshness interval.
        validity: Duration,
    },
    /// A caller supplied a monotonic time earlier than the recorded approval.
    ResolutionUseBeforeApproval {
        /// The recorded approval time.
        approved_at: Duration,
        /// The caller-supplied current time.
        current_time: Duration,
    },
    /// A bounded resolution approval reached its exclusive validity deadline.
    ResolutionApprovalExpired {
        /// The exclusive upper bound of the approval interval.
        valid_until: Duration,
        /// The caller-supplied current time.
        current_time: Duration,
    },
}

impl fmt::Display for DestinationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyResolution => formatter.write_str("resolver answer is empty"),
            Self::ResolutionAddressLimitExceeded { maximum_count } => write!(
                formatter,
                "resolver answer exceeds the maximum of {maximum_count} addresses",
            ),
            Self::AddressClassDenied {
                address,
                address_class,
            } => write!(
                formatter,
                "destination address {address} is denied as {address_class:?}",
            ),
            Self::LocalhostResolutionNotLoopback {
                address,
                address_class,
            } => write!(
                formatter,
                "localhost resolved to non-loopback address {address} classified as {address_class:?}",
            ),
            Self::LiteralOriginAddressMismatch {
                origin_address,
                resolved_address,
            } => write!(
                formatter,
                "literal origin address {origin_address} does not match resolved address {resolved_address}",
            ),
            Self::UnapprovedConnectionAddress { address } => write!(
                formatter,
                "connection address {address} is not in the approved resolution snapshot",
            ),
            Self::ResolutionSetExpanded { address } => write!(
                formatter,
                "refreshed DNS answer introduced unapproved address {address}",
            ),
            Self::InvalidResolutionValidity {
                validity,
                maximum_validity,
            } => write!(
                formatter,
                "resolution validity {validity:?} is outside 1ns..={maximum_validity:?}",
            ),
            Self::ResolutionValidityOverflow {
                approved_at,
                validity,
            } => write!(
                formatter,
                "resolution validity {validity:?} overflows approval time {approved_at:?}",
            ),
            Self::ResolutionUseBeforeApproval {
                approved_at,
                current_time,
            } => write!(
                formatter,
                "resolution use time {current_time:?} precedes approval time {approved_at:?}",
            ),
            Self::ResolutionApprovalExpired {
                valid_until,
                current_time,
            } => write!(
                formatter,
                "resolution approval expired at {valid_until:?}; current time is {current_time:?}",
            ),
        }
    }
}

impl std::error::Error for DestinationError {}

/// An approved, origin-bound, canonical DNS resolution snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionSnapshot {
    origin: Origin,
    addresses: BTreeSet<IpAddr>,
}

impl ResolutionSnapshot {
    /// Validate and pin one non-empty, bounded set of resolved addresses.
    pub fn approve(
        origin: Origin,
        addresses: impl IntoIterator<Item = IpAddr>,
        policy: &DestinationPolicy,
    ) -> Result<Self, DestinationError> {
        let addresses: Vec<IpAddr> = addresses
            .into_iter()
            .take(MAX_RESOLUTION_ADDRESS_COUNT + 1)
            .collect();
        Self::approve_slice(origin, &addresses, policy)
    }

    fn approve_slice(
        origin: Origin,
        addresses: &[IpAddr],
        policy: &DestinationPolicy,
    ) -> Result<Self, DestinationError> {
        if addresses.is_empty() {
            return Err(DestinationError::EmptyResolution);
        }
        if addresses.len() > MAX_RESOLUTION_ADDRESS_COUNT {
            return Err(DestinationError::ResolutionAddressLimitExceeded {
                maximum_count: MAX_RESOLUTION_ADDRESS_COUNT,
            });
        }

        let origin_constraint = classify_origin_host(&origin);
        let mut approved_addresses = BTreeSet::new();
        for address in addresses {
            let classified = policy.validate_address(*address)?;
            validate_origin_binding(origin_constraint, classified)?;
            approved_addresses.insert(classified.canonical_address());
        }
        Ok(Self {
            origin,
            addresses: approved_addresses,
        })
    }

    /// Return the logical origin whose DNS answer was approved.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the canonical addresses pinned for this resolution snapshot.
    #[must_use]
    pub const fn addresses(&self) -> &BTreeSet<IpAddr> {
        &self.addresses
    }

    /// Authorize one concrete connection address against the pinned set.
    pub fn authorize_connection(
        &self,
        address: IpAddr,
    ) -> Result<ConnectionEvidence, DestinationError> {
        let classified = classify_address(address);
        if !self.addresses.contains(&classified.canonical_address()) {
            return Err(DestinationError::UnapprovedConnectionAddress { address });
        }
        Ok(ConnectionEvidence {
            origin: self.origin.clone(),
            requested_address: address,
            canonical_address: classified.canonical_address(),
            address_class: classified.address_class(),
        })
    }

    /// Revalidate a fresh DNS answer without allowing the pinned set to expand.
    ///
    /// A non-empty subset is accepted so normal DNS answer contraction does not
    /// fail. Any new address is treated as a possible rebinding event.
    pub fn revalidate(
        &self,
        addresses: impl IntoIterator<Item = IpAddr>,
        policy: &DestinationPolicy,
    ) -> Result<Self, DestinationError> {
        let addresses: Vec<IpAddr> = addresses
            .into_iter()
            .take(MAX_RESOLUTION_ADDRESS_COUNT + 1)
            .collect();
        self.revalidate_slice(&addresses, policy)
    }

    fn revalidate_slice(
        &self,
        addresses: &[IpAddr],
        policy: &DestinationPolicy,
    ) -> Result<Self, DestinationError> {
        let refreshed = Self::approve_slice(self.origin.clone(), addresses, policy)?;
        if let Some(address) = refreshed.addresses.difference(&self.addresses).next() {
            return Err(DestinationError::ResolutionSetExpanded { address: *address });
        }
        Ok(refreshed)
    }
}

/// A resolution snapshot bound to one explicit trusted monotonic validity window.
///
/// The time values are opaque durations from one caller-owned monotonic clock
/// domain. This type never reads a wall clock itself. Constructing a new fresh
/// snapshot always reruns the same destination validation used by
/// [`ResolutionSnapshot`], so callers cannot renew authority without presenting
/// another policy-valid answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshResolutionSnapshot {
    snapshot: ResolutionSnapshot,
    approved_at: Duration,
    validity: Duration,
    valid_until: Duration,
}

impl FreshResolutionSnapshot {
    /// Validate addresses and bind the resulting snapshot to a bounded lifetime.
    pub fn approve(
        origin: Origin,
        addresses: impl IntoIterator<Item = IpAddr>,
        policy: &DestinationPolicy,
        approved_at: Duration,
        validity: Duration,
    ) -> Result<Self, DestinationError> {
        let snapshot = ResolutionSnapshot::approve(origin, addresses, policy)?;
        Self::from_snapshot(snapshot, approved_at, validity)
    }

    fn from_snapshot(
        snapshot: ResolutionSnapshot,
        approved_at: Duration,
        validity: Duration,
    ) -> Result<Self, DestinationError> {
        if validity.is_zero() || validity > MAX_RESOLUTION_VALIDITY {
            return Err(DestinationError::InvalidResolutionValidity {
                validity,
                maximum_validity: MAX_RESOLUTION_VALIDITY,
            });
        }
        let Some(valid_until) = approved_at.checked_add(validity) else {
            return Err(DestinationError::ResolutionValidityOverflow {
                approved_at,
                validity,
            });
        };
        Ok(Self {
            snapshot,
            approved_at,
            validity,
            valid_until,
        })
    }

    /// Return the logical origin whose DNS answer was approved.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        self.snapshot.origin()
    }

    /// Return the canonical addresses pinned for this fresh snapshot.
    #[must_use]
    pub const fn addresses(&self) -> &BTreeSet<IpAddr> {
        self.snapshot.addresses()
    }

    /// Return the trusted monotonic approval time.
    #[must_use]
    pub const fn approved_at(&self) -> Duration {
        self.approved_at
    }

    /// Return the configured non-zero validity budget.
    #[must_use]
    pub const fn validity(&self) -> Duration {
        self.validity
    }

    /// Return the exclusive upper bound of the approval interval.
    #[must_use]
    pub const fn valid_until(&self) -> Duration {
        self.valid_until
    }

    /// Authorize one pinned address only while the freshness window is valid.
    pub fn authorize_connection(
        &self,
        address: IpAddr,
        current_time: Duration,
    ) -> Result<FreshConnectionEvidence, DestinationError> {
        self.validate_current_time(current_time)?;
        let connection = self.snapshot.authorize_connection(address)?;
        Ok(FreshConnectionEvidence {
            connection,
            resolution_approved_at: self.approved_at,
            resolution_valid_until: self.valid_until,
            authorized_at: current_time,
        })
    }

    /// Revalidate a fresh answer and renew the same bounded validity budget.
    ///
    /// `revalidated_at` must come from the same monotonic clock domain and may
    /// not precede this snapshot's approval time. Expansion of the pinned set
    /// remains fail-closed under [`ResolutionSnapshot::revalidate`].
    pub fn revalidate(
        &self,
        addresses: impl IntoIterator<Item = IpAddr>,
        policy: &DestinationPolicy,
        revalidated_at: Duration,
    ) -> Result<Self, DestinationError> {
        if revalidated_at < self.approved_at {
            return Err(DestinationError::ResolutionUseBeforeApproval {
                approved_at: self.approved_at,
                current_time: revalidated_at,
            });
        }
        let snapshot = self.snapshot.revalidate(addresses, policy)?;
        Self::from_snapshot(snapshot, revalidated_at, self.validity)
    }

    fn validate_current_time(&self, current_time: Duration) -> Result<(), DestinationError> {
        if current_time < self.approved_at {
            return Err(DestinationError::ResolutionUseBeforeApproval {
                approved_at: self.approved_at,
                current_time,
            });
        }
        if current_time >= self.valid_until {
            return Err(DestinationError::ResolutionApprovalExpired {
                valid_until: self.valid_until,
                current_time,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OriginHostConstraint {
    Domain,
    Localhost,
    Literal(IpAddr),
}

fn classify_origin_host(origin: &Origin) -> OriginHostConstraint {
    let authority = origin
        .as_str()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        bracketed.split(']').next().unwrap_or(bracketed)
    } else if let Some((host, _port)) = authority.rsplit_once(':') {
        host
    } else {
        authority
    };

    if host == "localhost" {
        OriginHostConstraint::Localhost
    } else if let Ok(address) = host.parse::<IpAddr>() {
        OriginHostConstraint::Literal(classify_address(address).canonical_address())
    } else {
        OriginHostConstraint::Domain
    }
}

fn validate_origin_binding(
    origin_constraint: OriginHostConstraint,
    classified: ClassifiedAddress,
) -> Result<(), DestinationError> {
    match origin_constraint {
        OriginHostConstraint::Domain => Ok(()),
        OriginHostConstraint::Localhost if classified.address_class() == AddressClass::Loopback => {
            Ok(())
        }
        OriginHostConstraint::Localhost => Err(DestinationError::LocalhostResolutionNotLoopback {
            address: classified.original_address(),
            address_class: classified.address_class(),
        }),
        OriginHostConstraint::Literal(origin_address)
            if origin_address == classified.canonical_address() =>
        {
            Ok(())
        }
        OriginHostConstraint::Literal(origin_address) => {
            Err(DestinationError::LiteralOriginAddressMismatch {
                origin_address,
                resolved_address: classified.canonical_address(),
            })
        }
    }
}

/// Credential-free evidence that a concrete connection address was pinned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionEvidence {
    origin: Origin,
    requested_address: IpAddr,
    canonical_address: IpAddr,
    address_class: AddressClass,
}

impl ConnectionEvidence {
    /// Return the logical origin associated with the connection.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Return the address supplied immediately before the connection attempt.
    #[must_use]
    pub const fn requested_address(&self) -> IpAddr {
        self.requested_address
    }

    /// Return the canonical address matched against the pinned set.
    #[must_use]
    pub const fn canonical_address(&self) -> IpAddr {
        self.canonical_address
    }

    /// Return the destination class recorded for the connection.
    #[must_use]
    pub const fn address_class(&self) -> AddressClass {
        self.address_class
    }
}

/// Credential-free evidence that a pinned connection address was used while fresh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshConnectionEvidence {
    connection: ConnectionEvidence,
    resolution_approved_at: Duration,
    resolution_valid_until: Duration,
    authorized_at: Duration,
}

impl FreshConnectionEvidence {
    /// Return the underlying canonical destination/connection evidence.
    #[must_use]
    pub const fn connection_evidence(&self) -> &ConnectionEvidence {
        &self.connection
    }

    /// Return the trusted monotonic time at which the answer was approved.
    #[must_use]
    pub const fn resolution_approved_at(&self) -> Duration {
        self.resolution_approved_at
    }

    /// Return the exclusive upper bound of the resolution approval interval.
    #[must_use]
    pub const fn resolution_valid_until(&self) -> Duration {
        self.resolution_valid_until
    }

    /// Return the trusted monotonic time used for this authorization decision.
    #[must_use]
    pub const fn authorized_at(&self) -> Duration {
        self.authorized_at
    }
}
