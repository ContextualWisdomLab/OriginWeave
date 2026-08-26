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

## References

See [`../doctoring.md`](../doctoring.md#browser-fingerprinting-and-presentation-identity).
