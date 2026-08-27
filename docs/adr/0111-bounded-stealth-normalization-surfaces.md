# ADR 0111: Bounded stealth-normalization surfaces

- **Status:** Proposed
- **Date:** 2026-08-27

## Context

Browser pages can observe more than the static profile fields modeled by
[ADR 0110](0110-privacy-preserving-presentation-identity.md): canvas pixel
readback, WebGL vendor and renderer strings, Web Audio sample-rate reporting,
and WebRTC interface-candidate exposure. Longitudinal fingerprint research
shows these rendered and media surfaces carry entropy sufficient to reidentify
a browser across sessions (Laperdrix, Bielova, Baudry, & Avoine, 2020; Cao,
Li, & Wijmans, 2017), so an adapter that controls only the static profile
leaks most of the identifying signal a page can measure.

The W3C Fingerprinting Guidance prefers standardized, bounded values over
independent per-session randomization, because freshly randomized values can
create new distinguishers and reduce usability (World Wide Web Consortium,
2025). Camoufox is implementation precedent for native-layer consistency, not
policy authority: OriginWeave does not claim CAPTCHA bypass, bot-management
evasion, impersonation, or access-control circumvention (see
[`docs/PRD.md`](../../docs/PRD.md), PRD-CRAWL-003).

## Decision drivers

- Reduce the entropy available to a page from render and media surfaces
  without requiring per-session randomization.
- Keep every stealth surface bound to documented, enumerated values so the
  adapter can prove coverage and a reviewer can audit the value set.
- Fail closed when an adapter cannot prove it overrides a required surface.
- Keep browser authority independent from model output and page content.
- Produce deterministic evidence identities for replay and audit.
- Never read the host, never create a peer connection, and never defeat an
  access-control gate.

## Assumptions and authority boundaries

- This ADR governs the Rust control-plane contract only. It does not select a
  default stealth profile, does not read network interfaces, and does not
  grant origin, transport, extension, secret, or action authority.
- The kernel never shadows/overrides a page's own choice to disclose or an
  access-control decision. A CAPTCHA or consent challenge is recorded as
  blocked/degraded, not solved.
- WebRTC policy is policy metadata; the kernel never acts as a peer
  connection factory.

## Options considered

- **Expose host renderer values:** rejected because the real GPU, driver, and
  audio hardware names are high-entropy reidentifiers.
- **Randomize noise per session:** rejected because W3C guidance warns fresh
  random values can be more identifying and are not reproducible.
- **Provide bounded enumerated classes and require full-surface admission:**
  selected.

## Decision

OriginWeave will model render/media stealth surfaces in the Rust fingerprint
kernel using bounded, enumerated classes and a fail-closed surface-admission
contract. This slice adds:

- `CanvasNoise` — three bounded least-significant-bit classes with a `bit_shift`
  accessor (Crisp, Smooth, Diffuse) and a strict `quantize` guard.
- `WebGlRendererToken` — canonicalization of renderer spellings to either an
  `Angle` or `Standard` bounded token; unknown spellings fail closed.
- `WebAudioRate` — normalization to 44_100 or 48_000 Hz standard rates only.
- `WebRtcInterface` — either `DirectCandidates` (the adapter deliberately
  exposes direct interface candidates) or `MDnsOnly` (candidates are
  mDNS-published), a policy statement, never a network action. The explicit
  variant naming prevents callers from mistaking direct candidate disclosure
  for a privacy-preserving enabled/disabled mode.
- `require_stealth_surfaces` — requires Canvas, WebGL, WebAudio, and WebRtc
  coverage in stable order, duplicative and order independent.

The surface admission check does not itself apply the stealth; it is a
control-plane contract a future pinned Chromium adapter must prove with a
real-browser test.

## Consequences

The fingerprint container gains a deterministic, testable stealth surface
that is purely a contract. No real browser is yet claimed: any final adapter
must apply every listed surface before page script and prove no ambient host
value leaks. This slice does not make stealth or anti-detection a shipped
browser capability.

## Failure and degraded behavior

Construction rejects unknown sample rates, unknown WebGL tokens, and unknown
noise classes with typed errors. An adapter claiming fewer than all required
surfaces fails closed with the first missing surface in contract order.

## Security, privacy, and governance impact

The surface classes are identity evidence only; they do not authenticate,
authorize, or grant. Deterministic admission checks make adapter claims
auditable.

## Tests and acceptance evidence

- `stealth_noise_surface.rs` exercises full coverage and duplicate checks for
  each surface, off-by-reorder, off-duplicate, empty lists, and every class
  value; production functions/lines/regions/branches are covered by the
  workspace coverage gate.
- `web_gl_renderer_token` canonicalization accepts known spellings and
  rejects unknown renderer strings.
- Browser acceptance remains a pinned real-Chromium pre-script injection test
  and is not claimed by this slice.

## Migration and rollback

The new surface types are additive and do not change the digest serialization
of existing `PresentationProfile`. Rollback removes the stealth surface types
and tests; no persisted schema changes are introduced.

## Open follow-ups

- A real pinned-Chromium adapter that applies every listed surface before page
  script, with no host fallback, is required before any browser-capability
  claim.
- mDNS WebRTC candidate policy requires a release-time adapter test that
  cannot disclose local interface candidates.

## Supersession / reversal conditions

This ADR is superseded if a later decision selects per-session randomization
(cohort evidence required) or defines additional renderer/audio surfaces.
It is reversed if the surface-admission contract is removed without a
replacement.

## References

See [`../doctoring.md`](../doctoring.md#browser-fingerprinting-and-presentation-identity).
