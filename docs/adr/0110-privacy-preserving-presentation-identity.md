# ADR 0110: Privacy-preserving presentation identity

- **Status:** Proposed
- **Date:** 2026-08-27

## Context

Pages can combine screen, viewport, pixel ratio, processor count, language,
time-zone, graphics, font, media, and network observations into a persistent
browser fingerprint. Copying values from the host leaks ambient device
authority. Independently randomizing fields can instead create contradictory
identities and a smaller anonymity set. Camoufox demonstrates native browser
fingerprint injection, but its anti-detect and access-control-evasion goals do
not define OriginWeave policy.

## Decision drivers

- Reduce host-derived fingerprint entropy without creating contradictory field
  combinations.
- Keep browser authority independent from model output and page content.
- Produce deterministic, credential-free evidence for replay and audit.
- Avoid claiming browser-level protection before a real Chromium adapter proves
  every supported surface.

## Options considered

- **Expose host values:** rejected because it leaks ambient device identity.
- **Randomize fields independently:** rejected because contradictory
  combinations can be more identifying.
- **Use bounded, coherent presentation classes:** selected for the pure kernel;
  population-weighted classes remain unavailable without cited evidence.
- **Copy Camoufox anti-detect behavior:** rejected because bypass and
  circumvention are outside OriginWeave's authority model.

## Decision

OriginWeave will own a Rust presentation-identity contract behind narrow,
versioned Chromium adapters. A profile is stable for its governed lifecycle,
uses standardized or explicitly validated values, and binds its canonical
fields to a credential-free SHA-256 evidence identifier. The first supported
named time-zone profile is `UTC`; it has no daylight-saving transition, so
`Intl.DateTimeFormat().resolvedOptions().timeZone` and `Date` offsets cannot
contradict one another.

The adapter must apply every supported surface before page script executes,
must not fall back to host values for a claimed surface, and must preserve the
actual Chromium engine/platform family. Unsupported surfaces fail closed or
remain explicitly ambient and unreleased. The seed, if used for lifecycle
selection, is trusted control-plane material and never enters page, model, log,
or evidence context.

OriginWeave does not use presentation identity to solve CAPTCHA, impersonate a
target person or device, rotate residential routes, defeat bot-management, or
circumvent access controls. Such a challenge is recorded as blocked/degraded.

## Consequences

The pure `originweave-fingerprint` kernel can be independently tested, but it
does not make stealth or anti-detection a shipped browser capability. Release
evidence requires a pinned real-Chromium test covering every claimed active and
passive surface, lifecycle stability, no host fallback, digest binding, and
challenge non-circumvention. Region-specific profiles require cited population
evidence and named-time-zone/DST correctness; no arbitrary weights or
independent Cartesian sampling are permitted.

## Failure and degraded behavior

Construction rejects values outside the enumerated screen, viewport, and
processor classes or combinations whose viewport exceeds the screen. A future
adapter must fail closed for any surface it claims to control; unimplemented
surfaces remain ambient and unreleased.

## Security, privacy, and governance impact

Seeds remain trusted control-plane material and cannot enter page, model, log,
or evidence context. The digest is an integrity identifier, not authentication
or authorization. Presentation identity never grants origin, transport,
extension, secret, or action authority.

## Tests and acceptance evidence

Unit and integration tests cover deterministic derivation, independent seed
results, enumerated construction, cross-field consistency, standardized UTC
identity, canonical digest validation, and malformed input rejection. Browser
acceptance remains blocked on pinned real-Chromium pre-script injection and
host-fallback evidence.

## Migration and rollback

The crate has no shipped Chromium caller or persisted schema. Rollback removes
the workspace member and documentation before release. Once an adapter or
stored profile exists, any class or canonical-serialization change requires a
versioned migration and compatibility evidence.

## References

See [`../doctoring.md`](../doctoring.md#browser-fingerprinting-and-presentation-identity).
