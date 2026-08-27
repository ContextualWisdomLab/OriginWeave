# Web Audio fingerprinting privacy basis

## Claim boundary

This record supports one bounded claim:

> In an isolated OriginWeave privacy profile, the reviewed Manifest V3 guard is injected in the page's MAIN world at `document_start` and blocks Web Audio construction by default unless the exact canonical origin is present in the rendered trusted-policy allowlist.

It does **not** claim that:

- Web Audio is the only browser fingerprinting surface;
- a content script is stronger than a future engine-native Chromium policy;
- the guard defeats bot management, CAPTCHA, consent, or access control;
- all Chrome extensions or all websites remain compatible;
- ordinary media-element playback is blocked; or
- the feature is shipped before its stack reaches protected `main` and a signed release.

## Primary-source findings

The Web Audio API privacy section identifies multiple observable sources of fingerprint entropy. It notes exposure of supported sample rates and output channel counts; timing through `AnalyserNode` or `ScriptProcessorNode`; and implementation differences in `DynamicsCompressorNode`, `OscillatorNode`, and other signal-processing behavior. This supports blocking construction rather than only muting the destination or standardizing one sample-rate field.

Chrome's Manifest V3 content-script contract provides the enforcement placement used by the fixture:

- `run_at: document_start` injects before other page scripts;
- `world: MAIN` shares the page's JavaScript environment so the page-visible constructors can be replaced;
- `all_frames`, `match_about_blank`, and `match_origin_as_fallback` extend the rule to relevant descendant frames; and
- static `matches` patterns declare the isolated profile's injection scope.

The MAIN world is intentionally powerful and therefore the guard is constant, reviewed code. It consumes no page-provided instruction, network response, extension message, storage value, or model output.

## Claim-to-implementation traceability

| Claim | Implementation | Executable evidence |
|---|---|---|
| Default denial | `WebAudioFingerprintPolicy::default` has no grants | Rust default-policy test |
| Exact-origin exception | `BTreeSet<originweave_core::Origin>` membership | scheme/subdomain/port test |
| Bounded policy | 128 unique canonical origins | boundary test with 129 origins |
| Deterministic artifact | ordered origins replace one reviewed marker | repeated rendering/order test |
| First-script enforcement | MV3 `document_start` + MAIN world | pinned-Chromium top-document probe |
| Child-frame enforcement | `all_frames` and origin-fallback settings | pinned-Chromium iframe probe |
| Complete construction boundary | online/offline/prefixed/worklet constructor list | Rust, Python, and browser probes |
| Credential-free failure | fixed `NotAllowedError` and stable reason code | Rust error/reason tests |
| No ambient authority | no extension permissions, storage, network, or runtime messaging | manifest/source repository contract |

## Validation environment

The real-browser lane pins:

```text
Chrome for Testing: 150.0.7871.129
Chromium revision: r1639810
Trials: 3 isolated browser profiles
Transport: loopback-only ChromeDriver HTTP
Fixture: loopback static HTML
```

A missing required probe, an unexpected exception, a leaked constructor, a browser-version mismatch, or fewer than three successful trials fails the workflow. Unsupported or skipped probes are not counted as passing.

## Known limitations

- The branch provides a policy kernel, reviewed extension asset, and pinned-browser fixture. Full OriginWeave browser-profile materialization and signed distribution remain separate integration work.
- The current exception model is exact origin without expiry or maker-checker metadata. Durable enterprise grants require the administration plane.
- The test covers a same-origin child frame; complete cross-origin and opaque-origin coverage remains a release-program requirement.
- Other surfaces such as Canvas, WebGL, WebRTC, fonts, storage, and behavioral signals require their own controls.

## References — APA 7th

Google Chrome Developers. (2023). *Content scripts*. https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts

World Wide Web Consortium. (2024). *Web Audio API 1.1*. https://www.w3.org/TR/webaudio-1.1/
