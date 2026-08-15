# VPN profile support: primary-source doctoring

This note records the external standards boundary used by `originweave-vpn-profile`. It is evidence for design review, not a claim that OriginWeave already establishes a VPN tunnel or installs an operating-system profile.

## WireGuard profile observations

WireGuard's official quick-start documentation describes `wg setconf` configuration and `wg-quick` as an automation layer around ordinary interface and route setup. The official project description also separates WireGuard's cryptographic tunnel configuration from key distribution and ordinary route management. OriginWeave therefore treats interface/peer connectivity data as descriptive intent while refusing to inherit shell or route authority from profile text.

For imported WireGuard/wg-quick-style text, the crate accepts bounded interface and peer connectivity fields and completes a side-effect-free validation pass before raw `PrivateKey` or `PresharedKey` material reaches the caller-supplied trusted secret importer. During the real import pass, OriginWeave journals every returned opaque reference. If a later import or profile-construction step fails, it invokes `discard_secret` for those references in reverse order. Successful reverse-order rollback preserves the original typed failure; any cleanup failure returns `SecretCleanupFailed` so incomplete cleanup cannot be mistaken for a clean rejection. The parser rejects `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, and `Table`. The normalized result cannot execute those directives and does not itself create interfaces or routes.

Primary sources:

WireGuard. (n.d.). *Quick start*. https://www.wireguard.com/quickstart/

WireGuard. (n.d.). *WireGuard: Fast, modern, secure VPN tunnel*. https://www.wireguard.com/

## IKEv2 profile observations

RFC 7296 defines IKEv2 exchanges, authentication, and traffic selectors. Traffic selectors are negotiated by the peers and can be narrowed by a responder; a profile therefore describes proposed policy rather than proving that a future IPsec SA will have exactly those selectors.

RFC 4555 defines MOBIKE as an IKEv2 mobility and multihoming extension. Enabling MOBIKE in a normalized profile is only permission for a future IKE implementation or platform adapter to negotiate the extension; parsing the profile does not implement mobility or alter a network path.

RFC 7383 defines encrypted IKEv2 message fragmentation after negotiation. The normalized `Fragmentation` flag likewise records adapter intent and does not imply that the parser fragments packets.

The first OriginWeave IKEv2 profile format is intentionally provider-neutral and strict. It carries gateway and IKE identities, authentication mode, one modern allow-listed proposal, bounded traffic selectors, MOBIKE/fragmentation booleans, and liveness/rekey timing. As with WireGuard, a complete side-effect-free validation pass precedes caller-visible secret import. Raw PSK and EAP password material is replaced by opaque references, successful references are journaled, and later failure triggers reverse-order rollback through `discard_secret`. Vendor-native Apple, Windows, NetworkManager, and strongSwan file formats are not silently treated as equivalent.

Primary sources:

Eronen, P. (Ed.). (2006). *IKEv2 mobility and multihoming protocol (MOBIKE)* (RFC 4555). RFC Editor. https://doi.org/10.17487/RFC4555

Kaufman, C., Hoffman, P., Nir, Y., Eronen, P., & Kivinen, T. (2014). *Internet key exchange protocol version 2 (IKEv2)* (RFC 7296). RFC Editor. https://doi.org/10.17487/RFC7296

Smyslov, V. (2014). *Internet key exchange protocol version 2 (IKEv2) message fragmentation* (RFC 7383). RFC Editor. https://doi.org/10.17487/RFC7383

## OriginWeave authority boundary

The parser owns only bounded syntax and semantic validation, normalization, secret replacement, and compensating cleanup orchestration. `discard_secret` is a cleanup hook over opaque references, not proof of an atomic secret-store transaction; `SecretCleanupFailed` requires trusted-store reconciliation. A future privileged adapter remains responsible for operating-system installation, route/DNS changes, tunnel creation, IKE/IPsec negotiation, certificate handling, and teardown. Before any remote connection, OriginWeave destination/network policy must independently authorize the gateway; a value appearing in a VPN profile is never itself network authority.
