# WebDriver BiDi screen-area planning traceability

## Problem

The runtime-qualified WebDriver BiDi adapter plans reversible viewport/device-pixel-ratio and time-zone overrides, while the 3 September 2026 Working Draft also defines `emulation.setScreenSettingsOverride`. The screen operation is wider and more destructive than its width/height payload initially suggests.

WebDriver BiDi applies one `screenArea` rectangle to both the web-exposed total screen area and the web-exposed available screen area. OriginWeave `ScreenMetrics` currently models width, height, and color depth, but not `screen.availWidth` or `screen.availHeight`. Automatically deriving the command from `ScreenMetrics` inside the reusable profile plan would therefore mutate a page-observable fingerprint surface that the profile neither selected nor digest-bound. Color depth remains independently uncontrolled.

A second authority defect remains even when the operation is separated from the profile-derived plan. The standard stores one override per target browsing context. Setting a rectangle replaces that target's current override; `screenArea: null` removes the target from the override map. The standard does not restore a predecessor value. A validated browsing-context identifier therefore identifies where a mutation would occur but does not prove that OriginWeave owns the state being replaced or cleared.

A third reachability defect became executable after the ownership witness was introduced. The adapter intentionally had no production mint path for `WebDriverBidiScreenAreaOwnership` but still retained public explicit screen-area planner helpers. Exact-head CI `34419810636` ran on a GitHub-hosted Ubuntu 24.04 runner: Python repository contracts, formatting, and locked workspace tests passed; exact production coverage passed; strict Clippy failed because both explicit planner functions were dead production code. Keeping those helpers with a lint waiver would advertise executable authority that the canonical Browser Session owner cannot yet provide.

## Constraints

- Keep browser-domain truth in OriginWeave; WebDriver BiDi remains an adapter, not policy authority.
- Preserve the runtime-qualified 3 September 2026 Working Draft pin. Publication freshness is owned separately by `webdriver-bidi-publication-current.md`.
- Reuse validated presentation value objects rather than reopen raw width/height validation in the adapter.
- Do not treat a browsing-context identifier as mutation authority.
- A reusable browsing context may automatically plan only observables represented by the explicit presentation contract and paired with non-destructive cleanup.
- Screen-area mutation requires an exclusive/disposable Browser Session context or equivalent ownership proof before the command can be materialized.
- Do not retain dead public planner helpers or suppress strict Clippy while the ownership mint path is absent.
- Do not add media-feature cleanup, ambient-host fallback, live protocol I/O, command-ACK success semantics, or Chromium-specific authority here.

## Alternatives

1. **Insert screen settings into the reusable profile-derived plan.** Rejected. The apply operation changes the currently unmodelled available-screen rectangle, and the nullable reset does not restore a predecessor override.
2. **Mark `PresentationSurface::Screen` supported after planning width/height.** Rejected because color depth remains page-observable and uncontrolled, and available-screen geometry is absent from the profile.
3. **Carry full `ScreenMetrics` in the command payload.** Rejected because the command would contain color depth, which the protocol operation does not apply, while still failing to name the available-screen side effect.
4. **Expose context-only explicit Set/Reset commands.** Rejected after review. A context identifier does not establish ownership; setting can replace another owner's override and resetting can erase it without restoration.
5. **Remove the standard capability entirely.** Rejected. The protocol operation is useful and can be represented safely without making it ambient authority.
6. **Keep public explicit planners that accept an opaque witness before any production witness-mint path exists.** Rejected by exact-head Clippy RED. No legal production caller can reach them, so they are dead API rather than useful capability.
7. **Retain the typed screen-area value, ownership witness, and Set/Reset command vocabulary, but expose no screen-area planner until Browser Session supplies the mint transition and consumer path.** Selected. Protocol semantics remain explicit while executable authority stays with the lifecycle owner.
8. **Expand `PresentationProfile` immediately with available-screen dimensions.** Deferred. That changes the canonical fingerprint schema, replay digest, consistency rules, fixtures, and buyer evidence and needs its own test-first change.

## Decision

`originweave-bidi` retains `WebDriverBidiScreenArea` as the validated width/height projection, retains opaque `WebDriverBidiScreenAreaOwnership`, and retains explicit `SetScreenArea` / `ResetScreenArea` command intent. Both command variants carry the ownership witness rather than a raw `WebDriverBidiBrowsingContext`.

`WebDriverBidiScreenAreaOwnership` contains the exact validated browsing context but intentionally has no public constructor. The adapter therefore cannot mint its own proof from a context identifier. A future Browser Session integration may create the witness only after proving an exclusive/disposable lifecycle or an equivalent ownership transition.

There is no public explicit screen-area planner while that mint path is absent. The planner/transport consumer must be introduced together with the reviewed Browser Session ownership transition so strict Clippy and runtime evidence prove a real canonical call path. No `allow(dead_code)`/`expect(dead_code)` exception is used.

The ordinary `plan_standard_presentation_commands` and `plan_standard_presentation_cleanup` remain limited to viewport/DPR and time zone. The complete capability map continues to omit `PresentationSurface::Screen`, so `require_complete_presentation_profile()` still returns `MissingSurface(Screen)` until a reviewed owner models available-screen geometry, controls color depth, and proves the runtime application/cleanup lifecycle.

## Evidence and acceptance

PR #310 review identified two distinct findings. The first was the unmodelled available-screen side effect, repaired by keeping screen-area mutation out of the profile-derived reusable plan. The later exact-head review identified the ownership gap: a context-only `ResetScreenArea` could remove another owner's active override because `screenArea: null` deletes the target's override-map entry rather than restoring a prior value.

The first #311 ownership-witness implementation then exposed a third, executable finding. Run `34419810636` on exact `f1380ab8e091964ccbdd576d933cf19d696c3791` assigned hosted runners and executed repository code. `Rust contracts` job `102692565837` passed Python contracts, formatting, and the complete locked workspace tests before strict Clippy rejected `plan_explicit_screen_area_override` and `plan_explicit_screen_area_cleanup` as dead code. `Production coverage` job `102692565938` passed measurement, diagnostics publication, and exact enforcement. This is a source RED, not a queue or coverage failure.

The successor contract therefore requires:

- `WebDriverBidiScreenArea` to remain the typed width/height representation derived from validated screen metrics;
- an opaque `WebDriverBidiScreenAreaOwnership` carrying the exact context with no public mint constructor in the adapter;
- `SetScreenArea` and `ResetScreenArea` to carry that ownership witness rather than a raw context identifier;
- no public explicit screen-area planner until the Browser Session ownership mint path and consuming integration exist;
- no screen-area mutation in the reusable profile-derived plan while available-screen geometry is unmodelled;
- no media-feature reset;
- no color-depth field in the screen-area value object; and
- continued fail-closed complete Screen admission.

The initial successor RED briefly over-constrained the repair by requiring removal of all screen-area command intents. That remains unnecessary: the typed protocol vocabulary can stay dormant without exposing a callable dead planner or widening mutation authority.

Hosted exact-head repository checks, 100% owned-production coverage, security checks, central required workflows, and realistic pinned-Chromium acceptance remain separate evidence. A command intent or acknowledgement is never substituted for apply → page-observed post-condition → interaction/outcome → owned cleanup/destruction → post-cleanup observation.

## References

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* [Working Draft; runtime-qualified OriginWeave adapter pin]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
