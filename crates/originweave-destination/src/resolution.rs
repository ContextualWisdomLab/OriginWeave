use std::collections::BTreeSet;
use std::fmt;
use std::net::IpAddr;

use originweave_core::Origin;

use crate::{AddressClass, ClassifiedAddress, classify_address};

/// The largest resolver answer accepted by one resolution snapshot.
pub const MAX_RESOLUTION_ADDRESS_COUNT: usize = 256;

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
