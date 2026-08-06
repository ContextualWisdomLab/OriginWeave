use std::collections::BTreeSet;

use originweave_core::Origin;

use crate::ResolutionSnapshot;

/// The largest redirect chain accepted by the destination kernel.
pub const MAX_REDIRECT_HOPS: u8 = 20;

/// A lowercase SHA-256 digest of one complete canonical redirect target URI.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RedirectTargetDigest {
    canonical: String,
}

impl RedirectTargetDigest {
    /// Parse `sha256:` followed by exactly 64 lowercase hexadecimal digits.
    pub fn parse(input: &str) -> Result<Self, RedirectTargetDigestError> {
        let Some(hexadecimal) = input.strip_prefix("sha256:") else {
            return Err(RedirectTargetDigestError::InvalidFormat);
        };
        if hexadecimal.len() != 64
            || !hexadecimal
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(RedirectTargetDigestError::InvalidFormat);
        }
        Ok(Self {
            canonical: input.to_owned(),
        })
    }

    /// Return the canonical redirect-target digest.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }
}

/// A validation error for a redirect-target digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectTargetDigestError {
    /// The value was not a canonical lowercase SHA-256 identifier.
    InvalidFormat,
}

/// A deterministic reason that redirect authorization failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedirectError {
    /// The configured maximum was zero or exceeded [`MAX_REDIRECT_HOPS`].
    InvalidMaximumHops {
        /// The rejected maximum hop count.
        maximum_hops: u8,
    },
    /// The redirect chain reached its configured maximum.
    RedirectLimitExceeded,
    /// The target origin was not present in the explicit read-origin grant.
    OriginNotGranted {
        /// The ungranted target origin.
        origin: Origin,
    },
    /// The supplied resolution snapshot belongs to another origin.
    ResolutionOriginMismatch {
        /// The redirect target origin.
        target_origin: Origin,
        /// The origin bound to the resolution snapshot.
        resolution_origin: Origin,
    },
    /// An HTTPS request attempted to redirect to HTTP.
    InsecureSchemeDowngrade {
        /// The secure source origin.
        source_origin: Origin,
        /// The insecure target origin.
        target_origin: Origin,
    },
    /// The complete canonical target URI already appeared in the chain.
    RedirectCycle {
        /// The repeated target digest.
        target_digest: RedirectTargetDigest,
    },
}

/// Stateful redirect authorization for one bounded navigation chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectGuard {
    current_origin: Origin,
    maximum_hops: u8,
    hop_count: u8,
    seen_targets: BTreeSet<RedirectTargetDigest>,
}

impl RedirectGuard {
    /// Start a redirect chain at one approved origin and canonical target URI.
    pub fn new(
        initial_origin: Origin,
        initial_target_digest: RedirectTargetDigest,
        maximum_hops: u8,
    ) -> Result<Self, RedirectError> {
        if maximum_hops == 0 || maximum_hops > MAX_REDIRECT_HOPS {
            return Err(RedirectError::InvalidMaximumHops { maximum_hops });
        }
        Ok(Self {
            current_origin: initial_origin,
            maximum_hops,
            hop_count: 0,
            seen_targets: BTreeSet::from([initial_target_digest]),
        })
    }

    /// Return the origin that currently owns the redirect chain.
    #[must_use]
    pub const fn current_origin(&self) -> &Origin {
        &self.current_origin
    }

    /// Return the number of authorized redirects already consumed.
    #[must_use]
    pub const fn hop_count(&self) -> u8 {
        self.hop_count
    }

    /// Return the configured maximum redirect count.
    #[must_use]
    pub const fn maximum_hops(&self) -> u8 {
        self.maximum_hops
    }

    /// Authorize the next redirect after origin and DNS policy evaluation.
    pub fn authorize_redirect(
        &mut self,
        target_origin: Origin,
        target_digest: RedirectTargetDigest,
        target_resolution: &ResolutionSnapshot,
        readable_origins: &BTreeSet<Origin>,
    ) -> Result<RedirectEvidence, RedirectError> {
        if self.hop_count >= self.maximum_hops {
            return Err(RedirectError::RedirectLimitExceeded);
        }
        if !readable_origins.contains(&target_origin) {
            return Err(RedirectError::OriginNotGranted {
                origin: target_origin,
            });
        }
        if target_resolution.origin() != &target_origin {
            return Err(RedirectError::ResolutionOriginMismatch {
                target_origin,
                resolution_origin: target_resolution.origin().clone(),
            });
        }
        if is_https(&self.current_origin) && !is_https(&target_origin) {
            return Err(RedirectError::InsecureSchemeDowngrade {
                source_origin: self.current_origin.clone(),
                target_origin,
            });
        }
        if self.seen_targets.contains(&target_digest) {
            return Err(RedirectError::RedirectCycle { target_digest });
        }

        let evidence = RedirectEvidence {
            hop_number: self.hop_count + 1,
            source_origin: self.current_origin.clone(),
            target_origin: target_origin.clone(),
            target_digest: target_digest.clone(),
            approved_address_count: target_resolution.addresses().len(),
        };
        self.hop_count += 1;
        self.current_origin = target_origin;
        self.seen_targets.insert(target_digest);
        Ok(evidence)
    }
}

fn is_https(origin: &Origin) -> bool {
    origin.as_str().starts_with("https://")
}

/// Credential-free evidence for one approved redirect hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedirectEvidence {
    hop_number: u8,
    source_origin: Origin,
    target_origin: Origin,
    target_digest: RedirectTargetDigest,
    approved_address_count: usize,
}

impl RedirectEvidence {
    /// Return the one-based redirect hop number.
    #[must_use]
    pub const fn hop_number(&self) -> u8 {
        self.hop_number
    }

    /// Return the origin that emitted the redirect.
    #[must_use]
    pub const fn source_origin(&self) -> &Origin {
        &self.source_origin
    }

    /// Return the separately authorized redirect target origin.
    #[must_use]
    pub const fn target_origin(&self) -> &Origin {
        &self.target_origin
    }

    /// Return the digest of the complete canonical target URI.
    #[must_use]
    pub const fn target_digest(&self) -> &RedirectTargetDigest {
        &self.target_digest
    }

    /// Return the number of addresses approved for the redirect target.
    #[must_use]
    pub const fn approved_address_count(&self) -> usize {
        self.approved_address_count
    }
}
