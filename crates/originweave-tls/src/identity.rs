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
        let Some(authority) = origin.as_str().strip_prefix("https://") else {
            return Err(TlsError::OriginRequiresHttps {
                origin: origin.clone(),
            });
        };

        let host = if let Some(bracketed) = authority.strip_prefix('[') {
            let Some((host, suffix)) = bracketed.split_once(']') else {
                return Err(TlsError::InvalidReferenceIdentity {
                    origin: origin.clone(),
                });
            };
            if !suffix.is_empty()
                && (!suffix.starts_with(':')
                    || suffix.len() == 1
                    || !suffix[1..].bytes().all(|byte| byte.is_ascii_digit()))
            {
                return Err(TlsError::InvalidReferenceIdentity {
                    origin: origin.clone(),
                });
            }
            host
        } else if let Some((candidate_host, candidate_port)) = authority.rsplit_once(':') {
            if candidate_port.is_empty()
                || !candidate_port.bytes().all(|byte| byte.is_ascii_digit())
            {
                authority
            } else {
                candidate_host
            }
        } else {
            authority
        };

        if host.is_empty() {
            return Err(TlsError::InvalidReferenceIdentity {
                origin: origin.clone(),
            });
        }
        if let Ok(address) = host.parse::<IpAddr>() {
            return Ok(Self::Ip(address));
        }
        ServerName::try_from(host.to_owned()).map_err(|_error| {
            TlsError::InvalidReferenceIdentity {
                origin: origin.clone(),
            }
        })?;
        Ok(Self::Dns(host.to_owned()))
    }

    pub(crate) fn server_name(&self) -> Result<ServerName<'static>, TlsError> {
        match self {
            Self::Dns(name) => ServerName::try_from(name.clone())
                .map_err(|_error| TlsError::InvalidReferenceIdentity {
                    origin: Origin::parse("https://invalid.example")
                        .unwrap_or_else(|_error| unreachable!()),
                }),
            Self::Ip(address) => Ok(ServerName::IpAddress((*address).into())),
        }
    }

    pub(crate) const fn uses_sni(&self) -> bool {
        matches!(self, Self::Dns(_))
    }
}
