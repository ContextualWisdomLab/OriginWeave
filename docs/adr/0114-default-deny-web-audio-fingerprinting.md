# ADR 0114: Default-deny Web Audio fingerprinting

- Status: Proposed
- Date: 2026-08-27
- Supersedes: None
- Superseded by: None

## Context

A page does not need audible playback to use the Web Audio API as a fingerprinting surface. It can create an oscillator or other deterministic graph, inspect analyser or processor output, render an `OfflineAudioContext`, and combine implementation-specific numerical differences with other browser signals. Setting the final gain to zero only prevents sound from reaching the user; it does not prevent the browser from performing the measurements.

ADR 0111 standardizes a reported Web Audio sample rate as part of a bounded presentation identity. That reduces one ambient signal but does not prevent a page from constructing analyser, processor, compressor, oscillator, offline-rendering, or worklet graphs. A privacy profile therefore requires an explicit authority decision before any page script captures the native Web Audio constructors.

## Decision drivers

- Prevent silent Web Audio computation from becoming an ambient re-identification channel.
- Apply the decision before page JavaScript, including in child frames.
- Preserve ordinary `<audio>` and `<video>` playback.
- Support a bounded exact-origin exception for trusted applications that legitimately require Web Audio.
- Avoid random output perturbation that could become a new stable distinguisher.
- Produce deterministic, credential-free policy and browser evidence.
- Keep privacy enforcement separate from CAPTCHA, consent, bot-management, or access-control evasion.

## Assumptions and authority boundaries

- OriginWeave Agent and Crawler profiles are isolated managed profiles; the user's unrestricted Human profile is not silently rewritten by this policy.
- `originweave_core::Origin` is the sole parser and canonicalizer for an allowed origin.
- Exact origin means scheme, canonical host, and effective port. A grant does not include sibling subdomains or non-default ports.
- The checked-in Manifest V3 extension is a reviewed enforcement asset. The Rust policy renders a deterministic copy by replacing one fixed allowlist marker.
- Page content cannot add an origin grant or weaken the guard.
- This ADR governs Web Audio construction authority, not microphone permission, media-element playback, or operating-system audio routing.

## Options considered

### Normalize only the sample rate

Rejected as insufficient. It leaves timing, analyser, processor, compressor, oscillator, offline-rendering, and worklet behavior available to page scripts.

### Inject random noise into Web Audio output

Rejected. Independent or session-specific noise can create another distinguisher, makes scientific reproducibility difficult, and still permits resource consumption by hidden graphs.

### Block all media playback

Rejected as overbroad. Standard media elements are needed for ordinary audio and video playback and do not require exposing the Web Audio graph API.

### Default-deny Web Audio constructors with exact-origin grants

Selected. It removes the high-entropy computation surface from managed privacy profiles, preserves media elements, and makes compatibility exceptions explicit and auditable.

## Decision

OriginWeave Agent and Crawler privacy profiles must apply a Manifest V3 content script with all of the following properties:

```text
run_at: document_start
world: MAIN
all_frames: true
match_about_blank: true
match_origin_as_fallback: true
```

Unless the current canonical origin appears in the trusted profile's bounded allowlist, the script replaces these global construction entry points before page scripts execute:

```text
AudioContext
webkitAudioContext
OfflineAudioContext
webkitOfflineAudioContext
AudioWorkletNode
```

Each blocked constructor throws a `DOMException` named `NotAllowedError` with a fixed non-secret message. The replacement properties are non-writable and non-configurable for the lifetime of that document.

The control-plane policy:

- defaults to an empty allowlist;
- deduplicates canonical origins;
- permits no more than 128 unique exact origins;
- evaluates membership without wildcard or subdomain expansion;
- sorts origins deterministically through `BTreeSet`;
- renders the reviewed guard by replacing exactly one source marker; and
- emits the stable denial reason `web_audio_fingerprinting_no_explicit_origin_grant`.

The checked-in extension uses the empty allowlist and is therefore default-deny. A future trusted profile builder may materialize a policy-specific extension artifact from the Rust-rendered script, bind its digest to session evidence, and load only that artifact.

## Consequences

### Benefits

- Silent oscillator/analyser and offline-rendering fingerprints cannot start in the managed default profile.
- Top-level documents and child frames share the same default-deny boundary.
- Legitimate Web Audio applications have a narrow, reviewable compatibility path.
- Ordinary media elements remain available.
- The same source asset is used by the Rust renderer and the real-browser test.

### Costs

- Web games, conferencing tools, music applications, visualizers, and accessibility tools that require Web Audio will fail until their exact origin is granted.
- A content-script guard is a distribution-layer enforcement mechanism; a future Chromium policy integration may provide a stronger engine-native boundary.
- The broad match patterns are intentional for an isolated privacy profile and must not be copied into an unrelated extension distribution.

## Failure and degraded behavior

- Missing or malformed policy material fails to the checked-in empty allowlist.
- Too many unique grants fail closed before a browser artifact is created.
- A non-matching scheme, subdomain, or port remains blocked.
- If the real-browser lane cannot prove first-script and child-frame enforcement on the pinned Chromium build, the feature remains draft and cannot be claimed as released.
- A site broken by the guard requires an explicit exact-origin grant; disabling privacy globally is not the fallback.

## Security / privacy / governance impact

- The guard performs no network request, storage access, model call, secret read, or host fingerprint read.
- Page text, scripts, and WebMCP output are untrusted observations and cannot grant Web Audio.
- The policy does not evade access control or impersonate another device.
- Audit evidence records the origin, policy revision/digest, decision, and reason code, never page audio samples or user secrets.
- Enterprise administrators must use maker-checker governance for organization-wide grants once that administration plane ships.

## Tests and acceptance evidence

- Rust integration tests prove default denial, exact-origin behavior, canonical duplicate collapse, the 128-origin bound, deterministic ordering, complete constructor coverage, and stable errors.
- Python repository contracts prove the Manifest V3 timing/world/frame properties, source binding, no ambient extension permissions, pinned runner, and workflow artifact.
- A pinned Chrome for Testing 150.0.7871.129 lane runs three isolated trials and requires the first top-document script and first child-frame script to observe `NotAllowedError` from every available construction entry point.
- Existing repository gates continue to require exact production function, line, region, and branch coverage plus public documentation.

## Migration and rollback

The change is additive and stacked on ADRs 0110–0113. Rollback removes the privacy extension, policy module, and exact-origin grants together. Retaining grants while removing enforcement is prohibited because it would leave misleading policy evidence. A rollback must restore the prior claim that Web Audio is only normalized, not blocked, and must update the product-gap baseline.

## Open follow-ups

- Bind a rendered guard digest to the isolated browser-profile launch record.
- Add user and enterprise administration surfaces for temporary and durable exact-origin grants.
- Add expiry, reviewer, purpose, and application identity to durable organization grants.
- Prove behavior for cross-origin iframes and opaque-origin frames in the complete browser distribution.
- Evaluate engine-native Chromium permission/policy integration as a future replacement for the distribution extension.
- Add resource-budget evidence showing that blocked graphs do not consume hidden audio-processing capacity.

## Supersession / reversal conditions

Revisit this decision when Chromium provides a stable engine-native Web Audio permission or anti-fingerprinting control that can be policy-managed before page script with equal or stronger all-frame guarantees. Any replacement must preserve default denial, exact-origin exceptions, deterministic evidence, ordinary media playback, and compatibility rollback.

## References

Google Chrome Developers. (2023). *Content scripts*. https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts

World Wide Web Consortium. (2024). *Web Audio API 1.1*. https://www.w3.org/TR/webaudio-1.1/
