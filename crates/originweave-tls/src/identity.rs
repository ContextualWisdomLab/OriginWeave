use std::net::IpAddr;

use originweave_core::Origin;
use rustls::pki_types::ServerName;

use crate::TlsError;

/// The RFC 9525 reference identity derived from one canonical HTTPS origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsReferenceIdentity {
    /// A DNS reference identity that permits a matching SNI value.
    Dns(String),
    /// A literal IPv4 or IPv6 reference identity that sends no SNI value.
    Ip(IpAddr),
}

impl TlsReferenceIdentity {
    /// Derive a TLS reference identity from a canonical OriginWeave origin.
    pub fn from_origin(origin: &Origin) -> Result<Self, TlsError> {
        if origin.scheme() != "https" {
            return Err(TlsError::OriginRequiresHttps {
                origin: origin.clone(),
            });
        }
        if let Ok(address) = origin.host().parse::<IpAddr>() {
            return Ok(Self::Ip(address));
        }
        Ok(Self::Dns(origin.host().to_owned()))
    }

    #[inline(never)]
    pub(crate) fn server_name(&self, origin: &Origin) -> Result<ServerName<'static>, TlsError> {
        match self {
            Self::Dns(name) => match ServerName::try_from(name.clone()) {
                Ok(server_name) => Ok(server_name),
                Err(_error) => Err(TlsError::InvalidReferenceIdentity {
                    origin: origin.clone(),
                }),
            },
            Self::Ip(address) => Ok(ServerName::IpAddress((*address).into())),
        }
    }

    pub(crate) const fn uses_sni(&self) -> bool {
        matches!(self, Self::Dns(_))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn explicit_dns_variants_validate_before_becoming_server_names() {
        let origin = Origin::parse("https://example.com").expect("HTTPS origin");
        let valid = TlsReferenceIdentity::Dns("example.com".to_owned());
        assert!(matches!(valid.server_name(&origin), Ok(ServerName::DnsName(_))));

        let invalid = TlsReferenceIdentity::Dns("contains space".to_owned());
        assert!(matches!(
            invalid.server_name(&origin),
            Err(TlsError::InvalidReferenceIdentity { .. })
        ));
    }

    #[test]
    fn sni_is_used_only_for_dns_identity() {
        assert!(TlsReferenceIdentity::Dns("example.com".to_owned()).uses_sni());
        assert!(!TlsReferenceIdentity::Ip(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)).uses_sni());
    }
}
