# ADR 0113: Cross-surface platform coherence

- **Status:** Proposed
- **Date:** 2026-08-27

## Context

A page can reconcile several surfaces into one browser identity: the static
presentation profile (ADR 0110), the JavaScript `navigator.platform` token,
and the UA Client Hints platform object (ADR 0112). If an adapter presents a
`Windows` presentation profile but a `macOS` UA Client Hints platform, the
contradiction is itself a reidentification signal and negates the privacy
benefit of bounding each surface independently. Camoufox-style stealth
requires cross-surface coherence: every observable surface must describe the
same platform, or the union of surfaces leaks more than any single surface.

## Decision drivers

- Guarantee the presentation platform, its UA token, and the UA Client Hints
  platform always agree.
- Keep the mapping deterministic and enumerated so an adapter cannot widen it.
- Fail closed on any mismatch; never read the host.
- Produce a single source of truth for the platform-to-hints mapping.

## Options considered

- **Let the adapter choose hints independently:** rejected because the
  platform surfaces could contradict.
- **Duplicate the mapping in each module:** rejected because a reviewer could
  not prove the maps agree.
- **Bind the hints platform to the presentation platform in one method and a
  fail-closed coherence check:** selected.

## Decision

`PresentationPlatform::hints_platform` is the single source of truth mapping
each presentation platform to its canonical UA Client Hints platform
(`Windows` -> `Windows`, `MacOS` -> `macOS`, `Linux` -> `Linux`).
`require_hints_coherence` rejects any `UaClientHints` whose platform differs
from the canonical mapping for the presented platform. The existing
`user_agent_token` mapping completes the triad, so a page reconciling
`navigator.platform`, `userAgentData.platform`, and the profile cannot observe
a cross-surface contradiction.

## Consequences

Adapters that call `require_hints_coherence` before presenting a profile prove
platform agreement as a checked precondition. This remains a pure
control-plane contract; it does not install a browser or override real
surfaces.

## Failure and degraded behavior

Any hints platform other than the canonical mapping for the presented
platform returns `CoherenceError::HintsPlatformMismatch`.

## Security, privacy, and governance impact

The coherence check is identity evidence only and grants no origin, transport,
extension, secret, or action authority. It makes the platform triad auditable
and non-contradictory.

## Tests and acceptance evidence

`profile_coherence_surface.rs` covers every accepted mapping, every mismatch
across platforms, the canonical `hints_platform` and `user_agent_token`
mappings, the deterministic error text, and a round-trip coherence check.
The workspace coverage gate enforces 100% functions, lines, regions, and
branches. Browser acceptance remains out of scope.

## Migration and rollback

The new method and function are additive. Rollback removes them and the tests
without schema changes or digest impact.

## Open follow-ups

- A real pinned-Chromium adapter must prove the triad is applied before page
  script and that no ambient host value leaks.
- Subsequent cross-surface coherence (for example viewport-to-screen or
  language-to-platform) can reuse the same fail-closed pattern.

## References

See [`../doctoring.md`](../doctoring.md#browser-fingerprinting-and-presentation-identity).