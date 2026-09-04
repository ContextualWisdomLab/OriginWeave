use originweave_core::OriginWeaveProtocolVersion;

#[test]
fn protocol_version_can_be_constructed_from_runtime_values() {
    let major = std::hint::black_box(0_u16);
    let minor = std::hint::black_box(1_u16);
    let version = OriginWeaveProtocolVersion::new(major, minor);

    assert_eq!(version.major(), 0);
    assert_eq!(version.minor(), 1);
    assert_eq!(version.to_string(), "originweave/0.1");
}
