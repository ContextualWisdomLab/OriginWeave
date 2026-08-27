# ADR 0112: Bounded User-Agent Client Hints surfaces

- **Status:** Proposed
- **Date:** 2026-08-27

## Context

A user agent exposes Client Hints that carry more detail than the legacy
`User-Agent` header: brand and version lists, architecture, bitness, platform,
platform version, model, and mobileness. The legacy header incurs "quite a bit
of information packed into those strings ... form[ing] the basis for
fingerprinting schemes of all sorts" (Web Platform Incubator Community Group,
2026). An adapter that presents a static `PresentationProfile` (ADR 0110) while
letting the real UA Client Hints object leak exposes a direct, reconcilable
contradiction: a page requests high-entropy hints, compares them to the
profile, and reidentifies the host.

## Decision drivers

- Reduce the entropy a page can recover from `navigator.userAgentData` and
  the `Sec-CH-UA*` headers beyond the static profile.
- Keep every hint bounded to documented, enumerated values.
- Enforce the low-entropy rules the UA Client Hints draft itself defines
  (for example, non-mobile user agents report an empty model).
- Admit realistic Chromium brand lists, including ordinary multi-word brands
  and the punctuation used by the draft's GREASE algorithm, without widening
  the contract to arbitrary Unicode or control bytes.
- Fail closed when an adapter cannot prove a coherent hint set.
- Produce deterministic, credential-free evidence; never read the host and
  never evade an access-control or CAPTCHA gate.

## Options considered

- **Expose host hint values:** rejected because on-disk architecture, bitness,
  and model strings are re-identifying.
- **Randomize hint values per session:** rejected because W3C guidance warns
  fresh random values can be more distinguishing and are not reproducible.
- **Provide bounded enumerated classes and enforce the spec's coherence
  rules:** selected.

## Decision

OriginWeave will model UA Client Hints in the Rust fingerprint kernel using
bounded, enumerated classes plus the spec's cross-field coherence rules. This
slice adds:

- `UaBrand` — validates one non-empty brand/version pair. Brand names admit
  ASCII alphanumerics plus the separator bytes used by the WICG GREASE brand
  algorithm (`SP`, `(`, `)`, `-`, `.`, `/`, `:`, `;`, `=`, `?`, `_`), so
  values such as `Google Chrome`, `Not/A)Brand`, and `Not_A Brand` remain
  representable. Versions are non-empty dotted ASCII alphanumeric strings.
  The at-most-32-byte brand-name cap is an OriginWeave resource bound, not a
  UA Client Hints specification limit.
- `HintsArchitecture` (`x86`, `arm`) and `HintsBitness` (`32`, `64`) — bounded,
  enumerated architecture/bitness tokens.
- `HintsPlatform::normalize` — maps to `Windows`, `macOS`, `Linux` and rejects
  any other token.
- `UaClientHints::new` — requires a non-empty brand list, and requires an
  empty `model` when `mobile` is false, per the draft's processing model.

Admission checks are a control-plane contract only; they do not install a
browser or override real headers.

## Consequences

The fingerprint container gains a deterministic, testable UA-CH surface which
is purely a contract. No real browser is yet claimed: a future pinned Chromium
adapter must apply every listed hint surface before page script and prove no
ambient host value leaks. This does not make stealth or anti-detection a
shipped browser capability.

## Failure and degraded behavior

Construction rejects unknown architecture/bitness/platform tokens, over-length
brand names, empty brand names or versions, brand bytes outside the bounded
compatibility set, version bytes outside dotted ASCII alphanumeric syntax, an
empty brand list, and a non-mobile set with a non-empty model.

## Security, privacy, and governance impact

Hints are identity evidence only and grant no origin, transport, extension,
secret, or action authority. Deterministic checks make adapter claims
auditable. The admitted brand-name separators are a reviewed compatibility
set from the current WICG GREASE algorithm rather than an unbounded printable
ASCII allowance; quote, backslash, controls, and Unicode remain rejected.

## Tests and acceptance evidence

`ua_client_hints_surface.rs` exercises each surface: ordinary and realistic
Chromium/GREASE brand names, empty and invalid brand/version values, the local
brand-name length bound, every architecture/bitness/platform token and its
rejection, empty brand lists, mobile with model, and non-mobile with model.
The workspace coverage gate enforces 100% functions, lines, regions, and
branches. Browser acceptance remains out of scope.

## Migration and rollback

The new types are additive and do not change existing `PresentationProfile`
digests. Rollback removes the UA Client Hints types and tests without schema
changes.

## Open follow-ups

- A real pinned-Chromium adapter that applies the full brand/version list,
  low- and high-entropy hint set, and platform coherence before page script.
- A release-time acceptance test that cannot read the host architecture or
  bitness.

## Reference

Web Platform Incubator Community Group. (2026, February 10). *User-Agent Client Hints*
(Draft Community Group Report). https://wicg.github.io/ua-client-hints/