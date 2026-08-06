use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// The security-relevant class of one resolved network destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddressClass {
    /// A globally reachable unicast destination not covered by a denied class.
    Public,
    /// The protocol's unspecified address.
    Unspecified,
    /// A loopback destination local to the browser host.
    Loopback,
    /// IPv4 private-use or IPv6 unique-local address space.
    PrivateNetwork,
    /// Carrier-grade NAT or another shared address space.
    SharedNetwork,
    /// Link-local address space.
    LinkLocal,
    /// A well-known cloud or container metadata-service destination.
    MetadataService,
    /// Address space reserved for documentation and examples.
    Documentation,
    /// Address space reserved for benchmarking.
    Benchmarking,
    /// A multicast destination.
    Multicast,
    /// The limited IPv4 broadcast destination.
    Broadcast,
    /// An IPv4-in-IPv6 or deprecated transition destination.
    Transition,
    /// Address space reserved by an Internet protocol or registry.
    ProtocolReserved,
}

/// A resolved address together with its canonical form and policy class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassifiedAddress {
    original_address: IpAddr,
    canonical_address: IpAddr,
    address_class: AddressClass,
}

impl ClassifiedAddress {
    /// Return the address supplied by the resolver or network adapter.
    #[must_use]
    pub const fn original_address(&self) -> IpAddr {
        self.original_address
    }

    /// Return the address used for pinning and comparison.
    ///
    /// IPv4-mapped IPv6 values are reduced to their canonical IPv4 value.
    #[must_use]
    pub const fn canonical_address(&self) -> IpAddr {
        self.canonical_address
    }

    /// Return the security-relevant destination class.
    #[must_use]
    pub const fn address_class(&self) -> AddressClass {
        self.address_class
    }
}

/// Classify one resolved address without performing network I/O.
#[must_use]
pub fn classify_address(address: IpAddr) -> ClassifiedAddress {
    let canonical_address = match address {
        IpAddr::V4(_) => address,
        IpAddr::V6(ipv6_address) => match ipv6_address.to_ipv4_mapped() {
            Some(ipv4_address) => IpAddr::V4(ipv4_address),
            None => address,
        },
    };
    let address_class = match canonical_address {
        IpAddr::V4(ipv4_address) => classify_ipv4(ipv4_address),
        IpAddr::V6(ipv6_address) => classify_ipv6(ipv6_address),
    };
    ClassifiedAddress {
        original_address: address,
        canonical_address,
        address_class,
    }
}

fn classify_ipv4(address: Ipv4Addr) -> AddressClass {
    match address.octets() {
        [0, 0, 0, 0] => AddressClass::Unspecified,
        [169, 254, 169, 254] | [169, 254, 170, 2] | [100, 100, 100, 200] => {
            AddressClass::MetadataService
        }
        [127, _, _, _] => AddressClass::Loopback,
        [10, _, _, _] | [172, 16..=31, _, _] | [192, 168, _, _] => {
            AddressClass::PrivateNetwork
        }
        [100, 64..=127, _, _] => AddressClass::SharedNetwork,
        [169, 254, _, _] => AddressClass::LinkLocal,
        [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _] => {
            AddressClass::Documentation
        }
        [198, 18..=19, _, _] => AddressClass::Benchmarking,
        [192, 88, 99, _] => AddressClass::Transition,
        [224..=239, _, _, _] => AddressClass::Multicast,
        [255, 255, 255, 255] => AddressClass::Broadcast,
        [192, 0, 0, 9..=10] => AddressClass::Public,
        [0, _, _, _] | [192, 0, 0, _] | [240..=255, _, _, _] => {
            AddressClass::ProtocolReserved
        }
        _ => AddressClass::Public,
    }
}

fn classify_ipv6(address: Ipv6Addr) -> AddressClass {
    match address.segments() {
        [0, 0, 0, 0, 0, 0, 0, 0] => AddressClass::Unspecified,
        [0, 0, 0, 0, 0, 0, 0, 1] => AddressClass::Loopback,
        [0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254] => AddressClass::MetadataService,
        [0, 0, 0, 0, 0, 0, _, _]
        | [0x0064, 0xff9b, 0, 0, 0, 0, _, _]
        | [0x0064, 0xff9b, 1, _, _, _, _, _]
        | [0x2001, 0, _, _, _, _, _, _]
        | [0x2002, _, _, _, _, _, _, _] => AddressClass::Transition,
        [first, _, _, _, _, _, _, _] if first & 0xff00 == 0xff00 => {
            AddressClass::Multicast
        }
        [first, _, _, _, _, _, _, _] if first & 0xfe00 == 0xfc00 => {
            AddressClass::PrivateNetwork
        }
        [first, _, _, _, _, _, _, _] if first & 0xffc0 == 0xfe80 => {
            AddressClass::LinkLocal
        }
        [0x2001, 0x0db8, _, _, _, _, _, _] => AddressClass::Documentation,
        [first, _, _, _, _, _, _, _] if first & 0xfff0 == 0x3ff0 => {
            AddressClass::Documentation
        }
        [0x2001, 2, 0, _, _, _, _, _] => AddressClass::Benchmarking,
        [0x2001, 1, 0, 0, 0, 0, 0, 1..=3]
        | [0x2001, 3, _, _, _, _, _, _]
        | [0x2001, 4, 0x0112, _, _, _, _, _]
        | [0x2001, 0x0020..=0x002f, _, _, _, _, _, _]
        | [0x2001, 0x0030..=0x003f, _, _, _, _, _, _] => AddressClass::Public,
        [0x2001, second, _, _, _, _, _, _] if second <= 0x01ff => {
            AddressClass::ProtocolReserved
        }
        [first, _, _, _, _, _, _, _] if first & 0xe000 == 0x2000 => {
            AddressClass::Public
        }
        _ => AddressClass::ProtocolReserved,
    }
}
