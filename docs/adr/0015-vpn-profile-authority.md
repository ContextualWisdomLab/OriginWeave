# ADR 0015: Keep VPN profile parsing secret-safe and non-privileged

- Status: Proposed
- Date: 2026-08-14
- Decision owners: OriginWeave maintainers

## Context

OriginWeave needs to ingest WireGuard and IKEv2 connectivity profiles without turning profile text into host execution, route mutation, credential disclosure, or implicit network authorization. The repository's existing destination, transport, TLS, policy, and evidence boundaries remain the authorities for any later connection attempt.

WireGuard configuration commonly carries an interface private key, optional peer preshared keys, peer endpoints, and allowed IPs. `wg-quick` additionally recognizes operational directives such as `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, and `Table`; those directives can change host state and therefore exceed a parser's authority. WireGuard itself deliberately leaves key distribution and ordinary route management to other layers.

IKEv2 defines authenticated key exchange and traffic selectors. MOBIKE and IKE fragmentation are separately standardized extensions. A portable profile parser should normalize only the intent needed by a privileged adapter and must not claim to implement IKE negotiation, IPsec, route installation, or operating-system VPN provisioning.

## Decision drivers

- Prevent raw VPN credentials from entering normalized state, logs, diagnostics, or model-visible evidence.
- Keep untrusted profile text from gaining shell, route, DNS, socket, or tunnel authority.
- Provide one bounded reusable Rust normalization contract for later privileged platform adapters.
- Fail closed on unknown, contradictory, oversized, malformed, or authority-bearing profile fields.
- Keep optional IKEv2 negotiation extensions explicit rather than granting them by omission.
- Preserve exact testability, deterministic error behavior, and repository-wide coverage/security gates.

## Assumptions and authority boundaries

- Profile text is untrusted input and is not proof of source authenticity, gateway identity, route authority, key ownership, or user authorization.
- `originweave-vpn-profile` owns syntax validation, resource bounds, normalization, and replacement of raw credentials with opaque references only.
- Destination/network policy remains authoritative before any remote connection. A parsed endpoint or traffic selector is descriptive intent, not permission.
- Platform adapters remain responsible for operating-system installation, interface creation, route and DNS mutation, IKE/IPsec negotiation, certificate handling, rollback, audit, and recovery.
- The caller-supplied `VpnSecretImporter` is a trusted boundary. Entry points perform a side-effect-free validation pass before using the real importer, but a later importer failure can occur after earlier imports succeeded. Until disposal is represented in the trait, the importer implementation owns cleanup of partial imports through transactional staging, expiry, or an equivalent mechanism.
- The 253-byte limit applied to IKE identity text and EAP usernames is an OriginWeave resource bound for this interchange contract, not a claim that RFC 7296 imposes one universal protocol maximum on every identity type.

## Options considered

### Execute or pass through wg-quick profiles unchanged

Rejected. Hook and route directives would make untrusted profile text an execution/host-network authority and would intermingle secrets with operational configuration.

### Store raw credentials in normalized structs

Rejected. It would enlarge the secret exposure surface and make accidental logging, serialization, or model-context leakage materially easier.

### Let each operating-system adapter parse arbitrary vendor formats

Deferred. Platform-native importers may be valuable later, but the common control plane first needs a small typed contract so privileged adapters cannot silently redefine credential, route, or proposal policy.

### Normalize bounded provider-neutral intent and keep privilege elsewhere

Selected. It gives later adapters a reusable typed input while preserving existing OriginWeave policy and privilege boundaries.

## Decision

Add an independently reusable `originweave-vpn-profile` Rust crate with three public entry points:

1. `import_wireguard_profile` accepts a bounded WireGuard/wg-quick-style text profile.
2. `parse_ikev2_profile` accepts a strict provider-neutral `[IKEv2]` text profile.
3. `parse_vpn_profile` dispatches only when the first effective section unambiguously identifies one of those profile families.

Raw WireGuard `PrivateKey` and `PresharedKey` values, IKEv2 preshared keys, and IKEv2 EAP passwords cross one caller-supplied `VpnSecretImporter` boundary and are replaced by bounded opaque `SecretReference` values. Normalized profile types never retain those raw credentials.

The WireGuard parser rejects `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, `Table`, and every unknown key. It never invokes a command or mutates routing. WireGuard interface `Address` and peer `AllowedIPs` accept either explicit CIDR or a bare IP host shorthand; bare IPv4/IPv6 values normalize deterministically to `/32` or `/128`. Provider-neutral IKEv2 traffic selectors remain explicit CIDR, and no parsed network data authorizes a destination or route.

The provider-neutral IKEv2 parser accepts only bounded gateway, identity, username, authentication, proposal, traffic-selector, extension, and liveness/rekey fields. Remote identity, local identity, and EAP username are each capped at 253 bytes and reject control characters. Missing `Mobike` and `Fragmentation` values normalize to `false`; an adapter may negotiate either extension only after explicit profile opt-in. Unknown keys and contradictory authentication material fail closed.

This slice does **not** implement WireGuard tunneling, IKEv2 exchanges, IPsec, kernel interfaces, routing, DNS changes, OS profile installation, certificate enrollment, or automatic connection. Those are separate privileged adapter decisions.

## Consequences

### Positive

- Raw VPN credentials do not enter normalized profile state or model-visible evidence.
- Profile text cannot gain shell or route authority through wg-quick hooks.
- WireGuard and IKEv2 share one reusable, testable control-plane normalization boundary.
- Optional IKEv2 extensions require explicit permission instead of being silently enabled by omission.
- Future Linux, macOS, Windows, Android, or iOS adapters can consume the same typed intent while remaining independently privileged and policy-bound.

### Negative

- The strict IKEv2 format is OriginWeave's provider-neutral interchange contract, not an assertion that vendor-specific Apple, NetworkManager, strongSwan, or Windows profile files are interchangeable.
- Unknown or vendor-specific fields fail closed and require an explicit future adapter or ADR.
- The parser intentionally cannot reproduce wg-quick scripts or implicit route-table behavior.
- Provider-neutral IKEv2 traffic selectors require explicit CIDR; WireGuard bare-IP shorthand is accepted only as a host route and does not grant routing authority.
- Importer implementations must currently own cleanup of partial second-pass secret imports because the trait does not expose rollback/disposal.

## Failure and degraded behavior

- Oversized, malformed, duplicate, unsupported, contradictory, or invalid values return typed `ProfileError` values and do not produce normalized connectivity intent.
- Structural validation happens before the caller's real importer is invoked, so a profile rejected by parser validation does not import caller-visible secrets.
- If the trusted importer itself fails during the second pass, normalization fails closed with `SecretImportFailed`; earlier successful imports may exist without returned references and must be cleaned up by the importer implementation.
- Missing optional MOBIKE or fragmentation fields do not grant negotiation authority; both remain disabled.
- No parser error causes fallback to command execution, vendor-native import, automatic route mutation, or a more permissive profile family.

## Security / privacy / governance impact

- The raw-secret enum deliberately does not derive `Debug`; normalized output contains only opaque references.
- The crate must never execute profile-provided commands or treat route/DNS fields as host authority.
- Secret values must not be copied into diagnostics, review/model context, provenance records, or public evidence.
- A later secret broker must resolve opaque references only at immediate authorized use and under the repository's sensitive-data authority boundary.
- This ADR remains **Proposed** on the active feature branch. Its presence and implementation are not protected-main or shipped truth until a policy-compliant integration occurs; even then, lifecycle promotion requires an explicit ADR status change.

## Tests and acceptance evidence

The repository contract requires the crate, the secret-import boundary, explicit WireGuard hook/route rejection, an IKEv2 parser, this ADR, and primary-source doctoring. Rust and Python contracts cover successful WireGuard/IKEv2 normalization plus malformed, hostile, oversized, conflicting-authentication, secret-import, item-bound, endpoint, key-shape, identity-length/control-character, and unsupported-authority cases. Tests also require omitted MOBIKE/fragmentation to normalize disabled and rejected profiles to make zero calls to the caller importer before the second pass.

Acceptance for a changed exact head requires canonical formatting, locked workspace checks/tests, strict Clippy, rustdoc, exact owned-production function/line/region/branch coverage, required security/SAST workflows, resolved current review threads, and whatever independent approval live GitHub policy requires. Queued, skipped, predecessor-head, synthetic-merge-only, model-only, or status-only evidence is not a passing substitute.

## Migration and rollback

This crate introduces a new normalization authority and no protected-main persistence migration. Callers adopting it must treat `SecretReference` as opaque and must not depend on permissive extension defaults. If integration must be rolled back before downstream adapters depend on the API, revert the feature lane as a unit. After downstream adoption, supersede this ADR and provide an explicit API migration rather than silently widening accepted authority or changing credential semantics.

## Open follow-ups

- Evaluate an explicit atomic/batch or rollback-capable secret-import contract so cleanup is enforceable rather than solely an importer responsibility.
- Define privileged platform-adapter ADRs before any tunnel installation, route/DNS mutation, certificate enrollment, or automatic connection behavior is added.
- Add source-authentication and provenance requirements if profiles become remotely distributed or centrally managed.
- Evaluate vendor-native profile adapters independently; do not silently reinterpret them as the provider-neutral format.

## Supersession / reversal conditions

Supersede or reverse this decision if a later accepted architecture provides a stronger common credential transaction boundary, changes VPN profile authority ownership, requires a different portable interchange model, or proves that a platform-native parser can preserve the same fail-closed privilege and secret boundaries without ambiguity. Any superseding decision must retain explicit destination authorization, secret non-disclosure, rollback/recovery ownership, and evidence of hostile-input behavior.

## References

Eronen, P. (Ed.). (2006). *IKEv2 mobility and multihoming protocol (MOBIKE)* (RFC 4555). RFC Editor. https://doi.org/10.17487/RFC4555

Kaufman, C., Hoffman, P., Nir, Y., Eronen, P., & Kivinen, T. (2014). *Internet key exchange protocol version 2 (IKEv2)* (RFC 7296). RFC Editor. https://doi.org/10.17487/RFC7296

Smyslov, V. (2014). *Internet key exchange protocol version 2 (IKEv2) message fragmentation* (RFC 7383). RFC Editor. https://doi.org/10.17487/RFC7383

WireGuard. (n.d.). *Quick start*. https://www.wireguard.com/quickstart/
