#![allow(clippy::expect_used)]

use std::error::Error;
use std::str::FromStr;

use originweave_core::{OriginWeaveProtocolVersion, OriginWeaveProtocolVersionParseError};

#[test]
fn canonical_protocol_versions_parse_and_round_trip() -> Result<(), Box<dyn Error>> {
    let current = OriginWeaveProtocolVersion::from_str("originweave/0.1")?;
    assert_eq!(current, OriginWeaveProtocolVersion::new(0, 1));
    assert_eq!(current.to_string(), "originweave/0.1");

    let maximum = OriginWeaveProtocolVersion::from_str("originweave/65535.65535")?;
    assert_eq!(maximum, OriginWeaveProtocolVersion::new(u16::MAX, u16::MAX));
    assert_eq!(maximum.to_string(), "originweave/65535.65535");
    Ok(())
}

#[test]
fn malformed_or_noncanonical_protocol_versions_fail_closed() {
    let malformed = [
        "",
        "originweave/",
        "originweave/0",
        "originweave/0.",
        "originweave/.1",
        "originweave/0.1.0",
        "OriginWeave/0.1",
        "originweave/00.1",
        "originweave/0.01",
        "originweave/+0.1",
        "originweave/0.+1",
        "originweave/-0.1",
        "originweave/0.-1",
        "originweave/65536.1",
        "originweave/0.65536",
        " originweave/0.1",
        "originweave/0.1 ",
        "originweave/０.１",
    ];

    for value in malformed {
        assert_eq!(
            OriginWeaveProtocolVersion::from_str(value),
            Err(OriginWeaveProtocolVersionParseError::InvalidFormat)
        );
    }
}

#[test]
fn protocol_version_parse_error_is_stable_and_source_free() {
    let error = OriginWeaveProtocolVersionParseError::InvalidFormat;
    assert_eq!(
        error.to_string(),
        "OriginWeave protocol version must use canonical originweave/<major>.<minor> syntax"
    );
    assert!(error.source().is_none());
}
