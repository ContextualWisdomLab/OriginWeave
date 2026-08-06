#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use originweave_destination::{AddressClass, classify_address};

fn assert_class(input: &str, expected: AddressClass) {
    let address = input.parse::<IpAddr>().expect("test address must parse");
    assert_eq!(classify_address(address).address_class(), expected, "{input}");
}

#[test]
fn ipv4_special_purpose_ranges_are_classified_fail_closed() {
    let cases = [
        ("0.0.0.0", AddressClass::Unspecified),
        ("169.254.169.254", AddressClass::MetadataService),
        ("169.254.170.2", AddressClass::MetadataService),
        ("100.100.100.200", AddressClass::MetadataService),
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
        ("2001:200::1", AddressClass::Public),
    ];
    for (input, expected) in cases {
        assert_class(input, expected);
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

    let public = Ipv4Addr::new(8, 8, 4, 4);
    let direct = classify_address(IpAddr::V4(public));
    assert_eq!(direct.original_address(), IpAddr::V4(public));
    assert_eq!(direct.canonical_address(), IpAddr::V4(public));
    assert_eq!(direct.address_class(), AddressClass::Public);
}
