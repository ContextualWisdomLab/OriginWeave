#![allow(clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use originweave_core::Origin;
use originweave_destination::{
    AddressClass, DestinationError, DestinationPolicy, ResolutionSnapshot,
};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(a, b, c, d))
}

#[test]
fn policy_requires_explicit_address_class_authority() {
    let default_policy = DestinationPolicy::default();
    assert_eq!(default_policy, DestinationPolicy::public_web());
    assert_eq!(
        default_policy.allowed_classes(),
        &std::collections::BTreeSet::from([AddressClass::Public])
    );
    assert!(default_policy.allows(AddressClass::Public));
    assert!(!default_policy.allows(AddressClass::Loopback));
    assert_eq!(
        default_policy
            .validate_address(ipv4(8, 8, 8, 8))
            .expect("public destination")
            .address_class(),
        AddressClass::Public
    );
    assert_eq!(
        default_policy.validate_address(ipv4(10, 0, 0, 1)),
        Err(DestinationError::AddressClassDenied {
            address: ipv4(10, 0, 0, 1),
            address_class: AddressClass::PrivateNetwork,
        })
    );

    let managed = DestinationPolicy::from_allowed_classes([
        AddressClass::Public,
        AddressClass::Loopback,
        AddressClass::PrivateNetwork,
    ]);
    assert!(managed.allows(AddressClass::Loopback));
    assert!(managed.allows(AddressClass::PrivateNetwork));

    let deny_all = DestinationPolicy::from_allowed_classes(
        std::iter::empty::<AddressClass>(),
    );
    assert_eq!(
        deny_all.validate_address(ipv4(8, 8, 8, 8)),
        Err(DestinationError::AddressClassDenied {
            address: ipv4(8, 8, 8, 8),
            address_class: AddressClass::Public,
        })
    );
}

#[test]
fn approved_resolution_deduplicates_canonical_addresses_and_emits_evidence() {
    let target = origin("https://example.com");
    let public = ipv4(8, 8, 8, 8);
    let second = ipv4(1, 1, 1, 1);
    let mapped = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0x0808, 0x0808));
    let snapshot = ResolutionSnapshot::approve(
        target.clone(),
        [public, second, mapped],
        &DestinationPolicy::public_web(),
    )
    .expect("public DNS answer");

    assert_eq!(snapshot.origin(), &target);
    assert_eq!(snapshot.addresses().len(), 2);
    assert!(snapshot.addresses().contains(&public));
    assert!(snapshot.addresses().contains(&second));

    let evidence = snapshot
        .authorize_connection(mapped)
        .expect("mapped address matches canonical pin");
    assert_eq!(evidence.origin(), &target);
    assert_eq!(evidence.requested_address(), mapped);
    assert_eq!(evidence.canonical_address(), public);
    assert_eq!(evidence.address_class(), AddressClass::Public);

    assert_eq!(
        snapshot.authorize_connection(ipv4(9, 9, 9, 9)),
        Err(DestinationError::UnapprovedConnectionAddress {
            address: ipv4(9, 9, 9, 9),
        })
    );
}

#[test]
fn resolution_approval_rejects_empty_and_denied_answers() {
    let target = origin("https://example.com");
    assert_eq!(
        ResolutionSnapshot::approve(
            target.clone(),
            std::iter::empty::<IpAddr>(),
            &DestinationPolicy::public_web(),
        ),
        Err(DestinationError::EmptyResolution)
    );
    assert_eq!(
        ResolutionSnapshot::approve(
            target,
            [ipv4(169, 254, 169, 254)],
            &DestinationPolicy::public_web(),
        ),
        Err(DestinationError::AddressClassDenied {
            address: ipv4(169, 254, 169, 254),
            address_class: AddressClass::MetadataService,
        })
    );
}

#[test]
fn dns_revalidation_allows_only_non_empty_subsets_of_the_pinned_set() {
    let target = origin("https://example.com");
    let first = ipv4(8, 8, 8, 8);
    let second = ipv4(1, 1, 1, 1);
    let policy = DestinationPolicy::public_web();
    let snapshot = ResolutionSnapshot::approve(target.clone(), [first, second], &policy)
        .expect("initial resolution");

    let contracted = snapshot
        .revalidate([second], &policy)
        .expect("non-empty subset remains pinned");
    assert_eq!(contracted.origin(), &target);
    assert_eq!(contracted.addresses(), &std::collections::BTreeSet::from([second]));

    let introduced = ipv4(9, 9, 9, 9);
    assert_eq!(
        snapshot.revalidate([first, introduced], &policy),
        Err(DestinationError::ResolutionSetExpanded {
            address: introduced,
        })
    );
    assert_eq!(
        snapshot.revalidate([ipv4(127, 0, 0, 1)], &policy),
        Err(DestinationError::AddressClassDenied {
            address: ipv4(127, 0, 0, 1),
            address_class: AddressClass::Loopback,
        })
    );
    assert_eq!(
        snapshot.revalidate(std::iter::empty::<IpAddr>(), &policy),
        Err(DestinationError::EmptyResolution)
    );
}
