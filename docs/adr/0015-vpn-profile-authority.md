# ADR 0015: Keep VPN profile parsing secret-safe and non-privileged

- Status: Proposed
- Date: 2026-08-14
- Decision owners: OriginWeave maintainers

## Context

OriginWeave needs to ingest WireGuard and IKEv2 connectivity profiles without turning profile text into host execution, route mutation, credential disclosure, or implicit network authorization. The repository's existing destination, transport, TLS, policy, and evidence boundaries remain the authorities for any later connection attempt.

WireGuard configuration commonly carries an interface private key, optional peer preshared keys, peer endpoints, and allowed IPs. `wg-quick` additionally recognizes operational directives such as `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, and `Table`; those directives can change host state and therefore exceed a parser's authority. WireGuard itself deliberately leaves key distribution and ordinary route management to other layers.

IKEv2 defines authenticated key exchange and traffic selectors. MOBIKE and IKE fragmentation are separately standardized extensions. A portable profile parser should normalize only the intent needed by a privileged adapter and must not claim to implement IKE negotiation, IPsec, route installation, or operating-system VPN provisioning.

## Decision

Add an independently reusable `originweave-vpn-profile` Rust crate with three public entry points:

1. `import_wireguard_profile` accepts a bounded WireGuard/wg-quick-style text profile.
2. `parse_ikev2_profile` accepts a strict provider-neutral `[IKEv2]` text profile.
3. `parse_vpn_profile` dispatches only when the first effective section unambiguously identifies one of those profile families.

Raw WireGuard `PrivateKey` and `PresharedKey` values, IKEv2 preshared keys, and IKEv2 EAP passwords cross one caller-supplied `VpnSecretImporter` boundary and are immediately replaced by bounded opaque `SecretReference` values. Normalized profile types never retain those raw credentials.

The WireGuard parser rejects `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, `Table`, and every unknown key. It never invokes a command or mutates routing. Endpoint and allowed-IP data remain descriptive intent; they do not authorize a destination or route.

The provider-neutral IKEv2 parser accepts only bounded identity/authentication fields, a small modern proposal allow-list, bounded traffic selectors, MOBIKE, IKE fragmentation, and liveness/rekey timing. It rejects unknown keys and contradictory authentication material. Platform adapters must still apply destination policy and platform-specific privilege checks before installation or connection.

This slice does **not** implement WireGuard tunneling, IKEv2 exchanges, IPsec, kernel interfaces, routing, DNS changes, OS profile installation, certificate enrollment, or automatic connection. Those are separate privileged adapter decisions.

## Consequences

### Positive

- Raw VPN credentials do not enter normalized profile state or model-visible evidence.
- Profile text cannot gain shell or route authority through wg-quick hooks.
- WireGuard and IKEv2 share one reusable, testable control-plane normalization boundary.
- Future Linux, macOS, Windows, Android, or iOS adapters can consume the same typed intent while remaining independently privileged and policy-bound.

### Negative

- The strict IKEv2 format is OriginWeave's provider-neutral interchange contract, not an assertion that vendor-specific Apple, NetworkManager, strongSwan, or Windows profile files are interchangeable.
- Unknown or vendor-specific fields fail closed and require an explicit future adapter or ADR.
- The parser intentionally cannot reproduce wg-quick scripts or implicit route-table behavior.

## Alternatives considered

### Execute or pass through wg-quick profiles unchanged

Rejected. Hook and route directives would make untrusted profile text an execution/host-network authority and would intermingle secrets with operational configuration.

### Store raw credentials in normalized structs

Rejected. It would enlarge the secret exposure surface and make accidental logging, serialization, or model-context leakage materially easier.

### Let each operating-system adapter parse arbitrary vendor formats

Deferred. Platform-native importers may be valuable later, but the common control plane first needs a small typed contract so privileged adapters cannot silently redefine credential, route, or proposal policy.

## Verification

The repository contract requires the crate, the secret-import boundary, explicit WireGuard hook/route rejection, an IKEv2 parser, this ADR, and primary-source doctoring. Rust unit tests must cover successful WireGuard and IKEv2 normalization plus malformed, hostile, oversized, conflicting-authentication, secret-import, item-bound, and unsupported-authority cases. Repository-wide formatting, check, tests, Clippy, rustdoc, security scans, and exact owned production coverage remain required on the exact PR head.

## References

Eronen, P. (Ed.). (2006). *IKEv2 mobility and multihoming protocol (MOBIKE)* (RFC 4555). RFC Editor. https://doi.org/10.17487/RFC4555

Kaufman, C., Hoffman, P., Nir, Y., Eronen, P., & Kivinen, T. (2014). *Internet key exchange protocol version 2 (IKEv2)* (RFC 7296). RFC Editor. https://doi.org/10.17487/RFC7296

Smyslov, V. (2014). *Internet key exchange protocol version 2 (IKEv2) message fragmentation* (RFC 7383). RFC Editor. https://doi.org/10.17487/RFC7383

WireGuard. (n.d.). *Quick start*. https://www.wireguard.com/quickstart/
