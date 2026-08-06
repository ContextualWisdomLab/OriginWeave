#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use originweave_destination::{AddressClass, classify_address};

fn assert_class(input: &str, expected: AddressClass) {
    let address = input.parse::<IpAddr>().expect("test address must parse");
    assert_eq!(
        classify_address(address).address_class(),
        expected,
        "{input}"
    );
}

#[test]
fn ipv4_special_purpose_ranges_are_classified_fail_closed() {
    let cases = [
        ("0.0.0.0", AddressClass::Unspecified),
        ("100.100.100.200", AddressClass::MetadataService),
        ("168.63.129.16", AddressClass::MetadataService),
        ("169.254.169.254", AddressClass::MetadataService),
        ("169.254.170.2", AddressClass::MetadataService),
        ("169.254.170.23", AddressClass::MetadataService),
        ("127.0.0.1", AddressClass::Loopback),
        ("127.255.255.255", AddressClass::Loopback),
        ("10.0.0.1", AddressClass::PrivateNetwork),
        ("172.16.0.1", AddressClass::PrivateNetwork),
        ("172.31.255.255", AddressClass::PrivateNetwork),
        ("192.168.1.1", AddressClass::PrivateNetwork),
        ("100.64.0.1", AddressClass::SharedNetwork),
        ("100.127.255.254", AddressClass::SharedNetwork),
        ("169.254.1.1", AddressClass::LinkLocal),
        ("192.0.2.1", AddressClass::Documentation),
        ("198.51.100.1", AddressClass::Documentation),
        ("203.0.113.1", AddressClass::Documentation),
        ("198.18.0.1", AddressClass::Benchmarking),
        ("198.19.255.254", AddressClass::Benchmarking),
        ("192.88.99.1", AddressClass::Transition),
        ("224.0.0.1", AddressClass::Multicast),
        ("239.255.255.255", AddressClass::Multicast),
        ("255.255.255.255", AddressClass::Broadcast),
        ("0.1.2.3", AddressClass::ProtocolReserved),
        ("192.0.0.8", AddressClass::ProtocolReserved),
        ("192.0.0.170", AddressClass::ProtocolReserved),
        ("240.0.0.1", AddressClass::ProtocolReserved),
        ("255.0.0.1", AddressClass::ProtocolReserved),
        ("8.8.8.8", AddressClass::Public),
        ("172.32.0.1", AddressClass::Public),
        ("100.128.0.1", AddressClass::Public),
        ("192.0.0.9", AddressClass::Public),
        ("192.0.0.10", AddressClass::Public),
        ("192.0.1.1", AddressClass::Public),
        ("192.31.196.1", AddressClass::Public),
        ("192.52.193.1", AddressClass::Public),
        ("192.175.48.1", AddressClass::Public),
    ];
    for (input, expected) in cases {
        assert_class(input, expected);
    }
}

#[test]
fn ipv6_special_purpose_ranges_are_classified_fail_closed() {
    let cases = [
        ("::", AddressClass::Unspecified),
        ("::1", AddressClass::Loopback),
        ("fd00:ec2::23", AddressClass::MetadataService),
        ("fd00:ec2::254", AddressClass::MetadataService),
        ("::2", AddressClass::Transition),
        ("64:ff9b::808:808", AddressClass::Transition),
        ("64:ff9b:1::1", AddressClass::Transition),
        ("2001::1", AddressClass::Transition),
        ("2002:c000:0204::1", AddressClass::Transition),
        ("ff02::1", AddressClass::Multicast),
        ("fc00::1", AddressClass::PrivateNetwork),
        ("fd12:3456::1", AddressClass::PrivateNetwork),
        ("fe80::1", AddressClass::LinkLocal),
        ("febf::1", AddressClass::LinkLocal),
        ("2001:db8::1", AddressClass::Documentation),
        ("3fff::1", AddressClass::Documentation),
        ("2001:2::1", AddressClass::Benchmarking),
        ("2001:1::4", AddressClass::ProtocolReserved),
        ("2001:1ff::1", AddressClass::ProtocolReserved),
        ("100::1", AddressClass::ProtocolReserved),
        ("5f00::1", AddressClass::ProtocolReserved),
        ("1fff::1", AddressClass::ProtocolReserved),
        ("4000::1", AddressClass::ProtocolReserved),
        ("2606:4700:4700::1111", AddressClass::Public),
        ("2001:1::1", AddressClass::Public),
        ("2001:1::2", AddressClass::Public),
        ("2001:1::3", AddressClass::Public),
        ("2001:3::1", AddressClass::Public),
        ("2001:4:112::1", AddressClass::Public),
        ("2001:20::1", AddressClass::Public),
        ("2001:2f::1", AddressClass::Public),
        ("2001:30::1", AddressClass::Public),
        ("2001:3f::1", AddressClass::Public),
    ];
    for (input, expected) in cases {
        assert_class(input, expected);
    }
}

#[test]
fn ipv6_public_class_is_limited_to_current_iana_allocations() {
    for allocated in [
        "2001:200::1",
        "2001:400::1",
        "2001:600::1",
        "2001:800::1",
        "2001:c00::1",
        "2001:e00::1",
        "2001:1200::1",
        "2001:1400::1",
        "2001:1800::1",
        "2001:1a00::1",
        "2001:1c00::1",
        "2001:2000::1",
        "2001:4000::1",
        "2001:4200::1",
        "2001:4400::1",
        "2001:4600::1",
        "2001:4800::1",
        "2001:4a00::1",
        "2001:4c00::1",
        "2001:5000::1",
        "2001:8000::1",
        "2001:a000::1",
        "2001:b000::1",
        "2003::1",
        "2400::1",
        "2410::1",
        "2600::1",
        "2610::1",
        "2620::1",
        "2630::1",
        "2800::1",
        "2a00::1",
        "2a10::1",
        "2c00::1",
    ] {
        assert_class(allocated, AddressClass::Public);
    }

    for allocated_boundary in [
        "2001:3ff:ffff::1",
        "2001:bff:ffff::1",
        "2001:13ff:ffff::1",
        "2001:1fff:ffff::1",
        "2001:3fff:ffff::1",
        "2001:4dff:ffff::1",
        "2001:5fff:ffff::1",
        "2001:9fff:ffff::1",
        "2001:afff:ffff::1",
        "2001:bfff:ffff::1",
        "2003:3fff::1",
        "240f:ffff::1",
        "241f:ffff::1",
        "260f:ffff::1",
        "2610:1ff::1",
        "2620:1ff::1",
        "263f:ffff::1",
        "280f:ffff::1",
        "2a0f:ffff::1",
        "2a1f:ffff::1",
        "2c0f:ffff::1",
    ] {
        assert_class(allocated_boundary, AddressClass::Public);
    }

    for reserved in [
        "2001:1000::1",
        "2001:4e00::1",
        "2001:6000::1",
        "2001:c000::1",
        "2003:4000::1",
        "2004::1",
        "2200::1",
        "2420::1",
        "2500::1",
        "2610:200::1",
        "2620:200::1",
        "2640::1",
        "2700::1",
        "2810::1",
        "2a20::1",
        "2b00::1",
        "2c10::1",
        "2d00::1",
        "3000::1",
        "3ffe::1",
    ] {
        assert_class(reserved, AddressClass::ProtocolReserved);
    }
}

#[test]
fn ipv4_mapped_ipv6_is_canonicalized_before_policy_classification() {
    let original = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x7f00, 1);
    let classified = classify_address(IpAddr::V6(original));

    assert_eq!(classified.original_address(), IpAddr::V6(original));
    assert_eq!(
        classified.canonical_address(),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    );
    assert_eq!(classified.address_class(), AddressClass::Loopback);

    let mapped_metadata = Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xa9fe, 0xa9fe);
    let metadata = classify_address(IpAddr::V6(mapped_metadata));
    assert_eq!(
        metadata.canonical_address(),
        IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254))
    );
    assert_eq!(metadata.address_class(), AddressClass::MetadataService);

    let public = Ipv4Addr::new(8, 8, 4, 4);
    let direct = classify_address(IpAddr::V4(public));
    assert_eq!(direct.original_address(), IpAddr::V4(public));
    assert_eq!(direct.canonical_address(), IpAddr::V4(public));
    assert_eq!(direct.address_class(), AddressClass::Public);
}
