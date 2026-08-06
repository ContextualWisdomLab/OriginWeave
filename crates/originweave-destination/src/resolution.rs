use std::collections::BTreeSet;
use std::net::IpAddr;

use originweave_core::Origin;

use crate::{AddressClass, ClassifiedAddress, classify_address};

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
    pub fn from_allowed_classes(
        allowed_classes: impl IntoIterator<Item = AddressClass>,
    ) -> Self {
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
    pub fn validate_address(
        &self,
        address: IpAddr,
    ) -> Result<ClassifiedAddress, DestinationError> {
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
    /// A resolved address belongs to a class not permitted by policy.
    AddressClassDenied {
        /// The address supplied by the resolver or adapter.
        address: IpAddr,
        /// The denied security class.
        address_class: AddressClass,
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

/// An approved, origin-bound, canonical DNS resolution snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionSnapshot {
    origin: Origin,
    addresses: BTreeSet<IpAddr>,
}

impl ResolutionSnapshot {
    /// Validate and pin one non-empty set of resolved addresses.
    pub fn approve(
        origin: Origin,
        addresses: impl IntoIterator<Item = IpAddr>,
        policy: &DestinationPolicy,
    ) -> Result<Self, DestinationError> {
        let mut approved_addresses = BTreeSet::new();
        for address in addresses {
            let classified = policy.validate_address(address)?;
            approved_addresses.insert(classified.canonical_address());
        }
        if approved_addresses.is_empty() {
            return Err(DestinationError::EmptyResolution);
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
        let refreshed = Self::approve(self.origin.clone(), addresses, policy)?;
        if let Some(address) = refreshed.addresses.difference(&self.addresses).next() {
            return Err(DestinationError::ResolutionSetExpanded { address: *address });
        }
        Ok(refreshed)
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
