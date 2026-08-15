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
- Prevent a later secret-import failure from silently leaving earlier imports without a cleanup attempt.

## Assumptions and authority boundaries

- Profile text is untrusted input and is not proof of source authenticity, gateway identity, route authority, key ownership, or user authorization.
- `originweave-vpn-profile` owns syntax validation, resource bounds, normalization, replacement of raw credentials with opaque references, and compensating cleanup orchestration for failed normalization.
- Destination/network policy remains authoritative before any remote connection. A parsed endpoint or traffic selector is descriptive intent, not permission.
- Platform adapters remain responsible for operating-system installation, interface creation, route and DNS mutation, IKE/IPsec negotiation, certificate handling, rollback, audit, and recovery.
- The caller-supplied `VpnSecretImporter` is a trusted boundary. Entry points perform a side-effect-free validation pass before using the real importer, journal every successful opaque reference, and call `discard_secret` for those references in reverse order if a later import or construction step fails.
- Persistent importer implementations must override `discard_secret` or provide equivalent transactional behavior behind that hook. The default returns `SecretCleanupFailed`, so absent cleanup authority cannot be reported as successful rollback.
- Compensating rollback is not an atomic secret-store transaction. A `SecretCleanupFailed` result means at least one cleanup action could not be proven and requires trusted-store reconciliation.
- The 253-byte limit applied to IKE identity text and EAP usernames is an OriginWeave resource bound for this interchange contract, not a claim that RFC 7296 imposes one universal protocol maximum on every identity type.

## Options considered

### Execute or pass through wg-quick profiles unchanged

Rejected. Hook and route directives would make untrusted profile text an execution/host-network authority and would intermingle secrets with operational configuration.

### Store raw credentials in normalized structs

Rejected. It would enlarge the secret exposure surface and make accidental logging, serialization, or model-context leakage materially easier.

### Let each operating-system adapter parse arbitrary vendor formats

Deferred. Platform-native importers may be valuable later, but the common control plane first needs a small typed contract so privileged adapters cannot silently redefine credential, route, or proposal policy.

### Leave partial-import cleanup entirely outside the trait

Rejected. A caller could receive only `SecretImportFailed` while earlier successful imports remained unreferenced, and the normalizer would have no way to prove that compensating cleanup was attempted.

### Normalize bounded provider-neutral intent and keep privilege elsewhere

Selected. It gives later adapters a reusable typed input while preserving existing OriginWeave policy and privilege boundaries. The importer trait includes an explicit cleanup hook so the normalizer can journal and reverse a partial import sequence.

## Decision

Add an independently reusable `originweave-vpn-profile` Rust crate with three public entry points:

1. `import_wireguard_profile` accepts a bounded WireGuard/wg-quick-style text profile.
2. `parse_ikev2_profile` accepts a strict provider-neutral `[IKEv2]` text profile.
3. `parse_vpn_profile` dispatches only when the first effective section unambiguously identifies one of those profile families.

Raw WireGuard `PrivateKey` and `PresharedKey` values, IKEv2 preshared keys, and IKEv2 EAP passwords cross one caller-supplied `VpnSecretImporter` boundary and are replaced by bounded opaque `SecretReference` values. Normalized profile types never retain those raw credentials.

The entry points first execute a complete side-effect-free validation pass. During the real import pass, each successful `SecretReference` is journaled. If a later import or profile-construction step fails, the normalizer calls `VpnSecretImporter::discard_secret` for every journaled reference in reverse order. If every cleanup succeeds, the original typed failure is returned; if any cleanup fails, `ProfileError::SecretCleanupFailed` takes precedence so incomplete rollback cannot be mistaken for a clean rejection.

The WireGuard parser rejects `PreUp`, `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, `Table`, and every unknown key. It never invokes a command or mutates routing. WireGuard interface `Address` and peer `AllowedIPs` accept either explicit CIDR or a bare IP host shorthand; bare IPv4/IPv6 values normalize deterministically to `/32` or `/128`. Provider-neutral IKEv2 traffic selectors remain explicit CIDR, and no parsed network data authorizes a destination or route.

The provider-neutral IKEv2 parser accepts only bounded gateway, identity, username, authentication, proposal, traffic-selector, extension, and liveness/rekey fields. Remote identity, local identity, and EAP username are each capped at 253 bytes and reject control characters. Missing `Mobike` and `Fragmentation` values normalize to `false`; an adapter may negotiate either extension only after explicit profile opt-in. Unknown keys and contradictory authentication material fail closed.

This slice does **not** implement WireGuard tunneling, IKEv2 exchanges, IPsec, kernel interfaces, routing, DNS changes, OS profile installation, certificate enrollment, or automatic connection. Those are separate privileged adapter decisions.

## Consequences

### Positive

- Raw VPN credentials do not enter normalized profile state or model-visible evidence.
- Profile text cannot gain shell or route authority through wg-quick hooks.
- WireGuard and IKEv2 share one reusable, testable control-plane normalization boundary.
- Optional IKEv2 extensions require explicit permission instead of being silently enabled by omission.
- Failed multi-secret normalization attempts trigger an explicit reverse-order cleanup sequence.
- Missing or failed cleanup is represented by `SecretCleanupFailed` rather than hidden behind the original import error.
- Future Linux, macOS, Windows, Android, or iOS adapters can consume the same typed intent while remaining independently privileged and policy-bound.

### Negative

- The strict IKEv2 format is OriginWeave's provider-neutral interchange contract, not an assertion that vendor-specific Apple, NetworkManager, strongSwan, or Windows profile files are interchangeable.
- Unknown or vendor-specific fields fail closed and require an explicit future adapter or ADR.
- The parser intentionally cannot reproduce wg-quick scripts or implicit route-table behavior.
- Provider-neutral IKEv2 traffic selectors require explicit CIDR; WireGuard bare-IP shorthand is accepted only as a host route and does not grant routing authority.
- `discard_secret` is compensating cleanup rather than an atomic commit/abort protocol. Importers still own the correctness and durability of each discard operation.
- A cleanup failure masks the original parsing/import error with `SecretCleanupFailed` because incomplete cleanup is the more urgent operational state.

## Failure and degraded behavior

- Oversized, malformed, duplicate, unsupported, contradictory, or invalid values return typed `ProfileError` values and do not produce normalized connectivity intent.
- Complete structural and semantic validation happens before the caller's real importer is invoked, so a profile rejected during validation performs no caller-visible secret import.
- If the trusted importer fails after one or more successful imports, every journaled reference is offered to `discard_secret` in reverse order.
- When all compensating cleanup calls succeed, the original `SecretImportFailed` or later typed construction error is returned.
- When any compensating cleanup call fails, normalization returns `SecretCleanupFailed`; the trusted secret store must be treated as requiring reconciliation, and no normalized profile is returned.
- Missing optional MOBIKE or fragmentation fields do not grant negotiation authority; both remain disabled.
- No parser error causes fallback to command execution, vendor-native import, automatic route mutation, or a more permissive profile family.

## Security / privacy / governance impact

- The raw-secret enum deliberately does not derive `Debug`; normalized output contains only opaque references.
- The crate must never execute profile-provided commands or treat route/DNS fields as host authority.
- Secret values must not be copied into diagnostics, review/model context, provenance records, or public evidence.
- Persistent secret stores must implement `discard_secret` or an equivalent transaction behind that method; relying on the fail-closed default makes a partial import return `SecretCleanupFailed`.
- Cleanup receives opaque references only and must not require the raw secret to be re-exposed.
- A later secret broker must resolve opaque references only at immediate authorized use and under the repository's sensitive-data authority boundary.
- This ADR remains **Proposed** on the active feature branch. Its presence and implementation are not protected-main or shipped truth until a policy-compliant integration occurs; even then, lifecycle promotion requires an explicit ADR status change.

## Tests and acceptance evidence

The repository contract requires the crate, secret-import and cleanup hooks, explicit WireGuard hook/route rejection, an IKEv2 parser, this ADR, primary-source doctoring, and documentation that matches the implemented rollback boundary. Rust and Python contracts cover successful WireGuard/IKEv2 normalization plus malformed, hostile, oversized, conflicting-authentication, secret-import, item-bound, endpoint, key-shape, identity-length/control-character, unsupported-authority, reverse-order cleanup, and cleanup-failure cases. Tests also require omitted MOBIKE/fragmentation to normalize disabled and profiles rejected during the first pass to make zero calls to the caller importer.

Acceptance for a changed exact head requires canonical formatting, locked workspace checks/tests, strict Clippy, rustdoc, exact owned-production function/line/region/branch coverage, required security/SAST workflows, resolved current review threads, and whatever independent approval live GitHub policy requires. Queued, skipped, predecessor-head, synthetic-merge-only, model-only, or status-only evidence is not a passing substitute.

## Migration and rollback

This crate introduces a new normalization authority and no protected-main persistence migration. Callers adopting it must treat `SecretReference` as opaque, implement the cleanup hook for persistent imports, and must not depend on permissive extension defaults. If integration must be rolled back before downstream adapters depend on the API, revert the feature lane as a unit. After downstream adoption, supersede this ADR and provide an explicit API migration rather than silently widening accepted authority or changing credential semantics.

## Open follow-ups

- Evaluate a true atomic batch/transaction importer that can replace compensating per-reference cleanup when a secret backend supports prepare/commit/abort semantics.
- Define an operator reconciliation receipt for `SecretCleanupFailed` before connecting this crate to a durable production secret store.
- Define privileged platform-adapter ADRs before any tunnel installation, route/DNS mutation, certificate enrollment, or automatic connection behavior is added.
- Add source-authentication and provenance requirements if profiles become remotely distributed or centrally managed.
- Evaluate vendor-native profile adapters independently; do not silently reinterpret them as the provider-neutral format.

## Supersession / reversal conditions

Supersede or reverse this decision if a later accepted architecture provides a stronger atomic credential transaction boundary, changes VPN profile authority ownership, requires a different portable interchange model, or proves that a platform-native parser can preserve the same fail-closed privilege and secret boundaries without ambiguity. Any superseding decision must retain explicit destination authorization, secret non-disclosure, rollback/recovery ownership, and evidence of hostile-input behavior.

## References

Eronen, P. (Ed.). (2006). *IKEv2 mobility and multihoming protocol (MOBIKE)* (RFC 4555). RFC Editor. https://doi.org/10.17487/RFC4555

Kaufman, C., Hoffman, P., Nir, Y., Eronen, P., & Kivinen, T. (2014). *Internet key exchange protocol version 2 (IKEv2)* (RFC 7296). RFC Editor. https://doi.org/10.17487/RFC7296

Smyslov, V. (2014). *Internet key exchange protocol version 2 (IKEv2) message fragmentation* (RFC 7383). RFC Editor. https://doi.org/10.17487/RFC7383

WireGuard. (n.d.). *Quick start*. https://www.wireguard.com/quickstart/
