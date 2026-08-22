# ADR 0015: Release manifest identity boundary

- Status: Proposed
- Date: 2026-08-23

## Context

Issue #201 requires commercial OriginWeave releases to bind an exact OriginWeave source identity, Chromium revision, release channel, and the artifacts buyers install or verify. That larger release lifecycle will later add signing, SBOM/SLSA provenance, updater trust, rollback, platform support, and operational acceptance. Those authorities do not yet exist in the current source tree.

A smaller durable boundary is nevertheless required before packaging work can safely compose: one deterministic, bounded manifest identity for a release candidate. Without an explicit contract, equivalent release inventories can be represented differently, case-insensitive target filesystems can collapse distinct names, and host-specific device namespaces can reinterpret an apparent artifact leaf name.

## Decision drivers

- One release-candidate identity must not depend on caller ordering.
- Artifact references must remain leaf identities rather than filesystem paths.
- The same manifest must be unambiguous on supported case-sensitive and case-insensitive platforms.
- Manifest construction must remain inert metadata admission and must not grant signing, publication, installation, update, rollback, or release authority.
- Inputs must be bounded and fail closed before later packaging, signing, or updater layers consume them.

## Assumptions and authority boundaries

- Source identity is a full 40-character lowercase Git commit SHA.
- Chromium identity is a bounded canonical ASCII release token; this ADR does not claim that the token alone authenticates Chromium bytes.
- A release channel is explicit (`Stable`, `Beta`, or `Development`) metadata, not authorization to publish or promote a release.
- Every admitted artifact carries a bounded ASCII leaf name and an exact lowercase `sha256:` digest.
- The manifest contains no private signing material, credentials, secrets, installer authority, network authority, or update authority.
- Later release systems must independently authenticate the build, signer, provenance, platform package, updater metadata, and operational acceptance evidence.

## Options considered

### Caller-order manifest with host-local filenames

Rejected. Caller ordering makes identity representation unstable, and host-local filename rules can produce collisions or device-name reinterpretation on another supported platform.

### Canonical manifest that normalizes stored spelling

Rejected for this slice. Destructively rewriting admitted artifact spelling would make the manifest differ from the exact release artifact identity that packaging and verification need to preserve.

### Canonical admission with preserved spelling and collision guards

Selected. Preserve the admitted artifact spelling, sort artifacts deterministically by that spelling, and separately reject names whose ASCII-case-folded identities collide or whose basenames are reserved Win32 device names.

## Decision

OriginWeave release-manifest admission is a deterministic, bounded, fail-closed identity contract:

1. `source_commit` must be exactly 40 lowercase hexadecimal digits.
2. `chromium_revision` must be a non-empty bounded canonical ASCII token.
3. `channel` must be an explicit `ReleaseChannel` variant.
4. The artifact inventory must be non-empty and contain at most 64 entries.
5. Each artifact name must be a bounded ASCII leaf name containing only alphanumerics, `.`, `_`, and `-`; it cannot contain path separators, traversal-like `..`, leading punctuation, or trailing punctuation.
6. Artifact basenames `CON`, `PRN`, `AUX`, `NUL`, `COM1` through `COM9`, and `LPT1` through `LPT9` are rejected case-insensitively, including when followed by an extension. The artifact grammar already rejects the non-ASCII superscript-digit Win32 aliases.
7. Artifact names must be unique under ASCII case folding while their original admitted spelling is retained.
8. Each artifact digest must be exactly `sha256:` followed by 64 lowercase hexadecimal digits.
9. Admitted artifacts are stored in deterministic name order.
10. Validation errors remain typed, deterministic standard Rust errors.

Constructing or possessing a valid manifest does **not** authenticate an artifact, prove reproducibility, prove provenance, verify a signature, establish a signing identity, authorize a release channel, publish software, install software, update software, roll software back, or satisfy release acceptance.

## Consequences

The release candidate receives one bounded artifact-identity representation that is stable across caller order and avoids known case-insensitive and Win32 device-name collisions. Packaging, signing, provenance, and update layers can compose on top of this contract without inheriting ambient authority from it.

The contract is intentionally narrower than issue #201's final release manifest. Additional fields such as toolchain identity, adapter versions, build environment, dependency locks, signing identity, timestamp, SBOM/provenance references, and platform package identity remain future reviewed work rather than being inferred from this primitive.

## Failure and degraded behavior

Malformed, ambiguous, duplicate, unbounded, or empty identity evidence fails closed before a manifest is produced. There is no fallback that silently rewrites an invalid name, accepts an alternate digest representation, drops an artifact, or substitutes another release channel.

A caller that cannot provide canonical evidence does not receive a release manifest. That failure is not converted into permission to sign, publish, install, or update through another path.

## Security / privacy / governance impact

The boundary reduces path-confusion and cross-platform filename ambiguity without introducing credentials or protected values. It does not alter GitHub governance, reviewer authority, release signing authority, or protected-main policy. Scheduled development agents remain unable to merge, tag, publish, sign, or change release authority.

## Tests and acceptance evidence

The owning `originweave-core` tests must cover valid deterministic ordering, exact digests, identifier bounds, malformed names, path/traversal-like names, case-only collisions, Win32 reserved device basenames with and without extensions, neighboring admissible names, exact inventory bounds, duplicate names, channel access, and deterministic standard error contracts.

Owned production function, line, region, and branch coverage remains exactly 100% on the unchanged reviewed head. CI/security/scanner evidence is exact-head evidence only; predecessor or model-only evidence cannot satisfy acceptance.

## Migration and rollback

This is a new admission primitive with no protected-main persisted manifest migration. If the contract proves incompatible before acceptance, the Proposed ADR and owning feature branch can be revised or withdrawn without granting legacy inputs grandfathered authority.

After acceptance and external release artifacts depend on this schema, any incompatible identity change requires an explicit versioning/migration decision rather than silent parser broadening.

## Open follow-ups

- Complete issue #201's signed cross-platform distribution and updater trust architecture.
- Bind toolchain, adapter, build-environment, dependency-lock, SBOM/SLSA provenance, signing, timestamp, and platform-package evidence through separately reviewed contracts.
- Define manifest serialization/versioning before any durable external release-manifest format is promised.
- Add signing-key rotation, compromise response, update rollback/freeze protection, and platform release acceptance without transferring authority from this metadata type.

## Supersession / reversal conditions

Supersede this ADR when a versioned external release-manifest specification replaces the in-process identity primitive, or when supported-platform packaging requires a stronger canonical filename identity. Reversal must preserve fail-closed artifact identity and cannot make possession of metadata equivalent to release authority.

## References

Microsoft. (n.d.). *Naming files, paths, and namespaces*. Microsoft Learn. Retrieved August 23, 2026, from https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
