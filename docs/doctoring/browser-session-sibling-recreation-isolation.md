# Browser Session sibling recreation isolation

Status: test-first acceptance design stacked on PR #318. This document does not describe protected-main runtime behavior.

## Problem

Browser Session navigation authority is scoped to an exact owned context generation rather than to a raw `BrowsingContextId`. PR #318 already requires an unused re-establishment opportunity in context A to survive proven destruction of sibling context B, and separately requires destroy/recreate reuse of a raw context id to create a newer ownership generation. Those two invariants must also compose.

A cleanup implementation that stores re-establishment eligibility in aggregate-global state can pass each isolated contract yet still lose A's unused opportunity when B is recreated. A different faulty implementation can let the newly created B generation inherit stale commit state or can revoke B's fresh authority when A later re-establishes. Raw context-id reuse makes this an ABA-sensitive authority boundary rather than ordinary collection cleanup.

A second failure mode appears after B-new is accepted under the reused raw id. An implementation can validate an old witness by raw `BrowsingContextId` and whichever navigation state happens to exist for that raw id, instead of by exact ownership generation and current navigation witness. While B-new is idle, stale B-old terminal or download evidence can therefore manufacture eligibility or damage B-new's fresh authority. After B-new starts navigating, the same defect can close B-new with B-old evidence or consume/reset B-new's first `navigationCommitted` slot.

A third failure mode appears only after B-new has completed one valid navigation and re-established authority. An implementation can retire the predecessor-generation tombstone or raw-id generation discriminator after that successful cycle. B-old evidence then remains stale during B-new's first navigation but becomes eligible for accidental remapping when B-new starts a second navigation. This is a long-lived-session defect: a recreated context can look correct at admission and first settlement while becoming vulnerable only on a later navigation cycle.

A fourth failure mode is capability resurrection rather than witness remapping. A stale predecessor replay can leave the newest B-new authority apparently correct while reactivating an older B-new capability that was already consumed by navigation 1 or navigation 2. Checking only the current projection or the immediately previous authority is insufficient: every capability superseded within the recreated ownership generation must stay permanently non-executable across later predecessor replay.

## Acceptance decision

For each qualified A closure family—complete-positive settlement, `navigationAborted`, `navigationFailed`, and `downloadWillBegin`—the first hostile sequence is:

`A earns unused eligibility → B-old starts and records navigationCommitted → proven destroy(B-old) → recreate B-new with the same raw BrowsingContextId and a newer BrowserContextEpoch → immediately re-establish(A) → prove B-new authority is executable → reject delayed commit(B-old) → prove A-fresh authority remains executable and B-new remains current`.

The ordering is intentional. A must re-establish immediately after B-new creation, before any later protocol observation or authority operation can compensate for cleanup that incorrectly erased A's eligibility. B-new is exercised immediately after A re-establishes, before stale predecessor evidence can compensate for an A transition that incorrectly revoked the sibling. After stale B-old evidence is rejected, B-new must still be the current generation and A's fresh authority must still execute.

The second hostile matrix makes the generation boundary observable across every B-new navigation phase:

`B-old start + commit → destroy(B-old) → recreate B-new → [B-new idle before navigation | B-new pending before first commit | B-new already committed] → replay one B-old {navigationCommitted, complete-positive, navigationAborted, navigationFailed, downloadWillBegin} event → prove B-new phase-local authority/commit/witness state unchanged → complete B-new only with B-new evidence → re-establish(B-new) → replay the same B-old event again`.

Generation validation must happen before consulting raw context identity, current pending state, commit-progress state, terminal eligibility, presentation authority, or the aggregate epoch allocator. In the idle branch, all five B-old replay families must fail while B-new keeps its complete fresh creation authority executable and has no navigation-derived re-establishment eligibility; B-new must still be able to start its own navigation afterward. In the pre-commit branch, B-new's own first `navigationCommitted` must remain available after every stale B-old replay. In the committed branch, its duplicate commit must remain stale after every replay. No branch may expose re-establishment until B-new closes with its own current witness. B-old replay remains stale after B-new has re-established and may not reactivate retained authority, manufacture another eligibility, spend a hidden epoch, or revoke any independent sibling authority.

The third hostile matrix extends the same generation boundary across repeated navigation cycles in B-new:

`B-old start + commit → destroy(B-old) → recreate B-new → B-new navigation 1 commit + close + re-establish → B-new navigation 2 [pending before first commit | already committed] → replay one B-old {navigationCommitted, complete-positive, navigationAborted, navigationFailed, downloadWillBegin} event → prove navigation 2 phase unchanged → close navigation 2 only with its own witness → re-establish again → replay the same B-old event once more`.

A successful first B-new re-establishment is not a reason to forget B-old's ownership generation. During navigation 2, stale B-old replay must neither consume B-new's still-available first commit nor reset an already-consumed commit slot; it must not close navigation 2, reactivate navigation 1 authority, create eligibility, spend an aggregate epoch, or perform adapter I/O. Navigation 2 must mint exactly one next authority, and the authority consumed by navigation 2 must remain permanently stale after both the second re-establishment and another B-old replay. Only the second fresh authority may remain executable.

The fourth hostile check preserves cumulative revocation rather than only the current slot:

`B-old start + commit → destroy(B-old) → recreate B-new → B-new navigation 1 close + re-establish → B-new navigation 2 close + re-establish → prove both the original B-new creation authority and navigation-1 fresh authority are stale → replay one B-old family → prove both historical authorities remain stale → prove only navigation-2 fresh authority executes`.

A stale predecessor event must not resurrect any historical capability inside the recreated ownership generation even if the current projection, epoch, lifecycle state, and recovery evidence remain unchanged. Capability revocation is monotonic for the lifetime of each issued authority value: once a later navigation consumes an authority, neither same-generation progress nor predecessor-generation replay may make it executable again.

The required invariants are:

- B-old destruction and B-new creation may change only B's ownership generation and the aggregate-wide epoch allocator. They must not erase or recreate A's already-earned eligibility, mutate lifecycle/recovery state, or reactivate A's retained pre-navigation authority.
- A re-establishment remains single-use and receives the next aggregate-issued epoch after B-new creation; it must not reset or reuse an earlier epoch.
- A re-establishment must not revoke B-new's fresh creation authority. That authority is exercised before any stale predecessor event can compensate for an incorrect transition.
- Protocol evidence tied to B-old remains stale after raw-id recreation and fails before adapter I/O in every B-new navigation phase. Its rejection must not revoke or alter B-new's current generation or an independent sibling authority.
- While B-new is idle, all five B-old replay families leave B-new's complete fresh authority identity executable, create no eligibility, and cannot consume an epoch. B-new must still admit its own navigation and first commit afterward.
- If B-new has a current navigation, all five B-old progress/terminal/liveness replay families remain generation-stale before B-new's first commit, after B-new's first commit, and after B-new re-establishment. They may not consume or reset B-new's own commit-progress slot, close its witness, spend a hidden epoch, or create eligibility.
- B-old generation staleness survives any number of later valid B-new navigation/re-establishment cycles. A successful B-new re-establishment must not retire predecessor-generation correlation state needed to reject delayed evidence on a later navigation.
- Every later B-new navigation independently revokes the authority that started it, owns its own exactly-once commit-progress slot and closure witness, and mints at most one next authority. Stale B-old evidence cannot bridge from an earlier ownership generation into a later navigation cycle.
- Revocation is cumulative across B-new navigation cycles. After multiple re-establishments, every older B-new authority remains permanently `AuthorityMismatch`; predecessor replay may not resurrect the original creation authority or any intermediate fresh authority while leaving the newest authority apparently valid.
- B-new has no navigation-derived re-establishment eligibility merely because its predecessor did.
- Fresh authorities must execute against their exact current contexts. Projection equality or epoch arithmetic without executable authority is insufficient evidence.

Browser Session remains the deterministic authority owner. WebDriver BiDi navigation and browsing-context identifiers are adapter evidence only; PR #316 remains responsible for protocol correlation. Recreating or reconciling a remote browser target is not inferred from command acknowledgement.

## Evidence required before adoption

The test-first Rust acceptance in the stacked successor remains structural RED until PR #317 supplies the production navigation state machine. The exact successor must then pass repository contracts, Rust 1.97.1 `cargo fmt --all -- --check`, locked workspace tests, strict Clippy, rustdoc/API documentation, and production function/line/region/branch coverage at 100% on one exact head.

A later explicitly qualified Chromium/WebDriver BiDi lane must reproduce sibling raw-id recreation and prove browser-observed post-conditions for both the preserved sibling authority and the fresh B generation. It must replay all five B-old navigation evidence families while B-new is idle, pre-commit pending, and already committed, then repeat stale replay after B-new re-establishment. It must also start a second B-new navigation after successful first re-establishment and repeat all five B-old replay families both before and after that second navigation's first commit, followed by another replay after the second re-establishment. At that final replay point, both the original B-new creation authority and every intermediate re-established authority must still fail closed while only the newest authority remains executable. B-new's own navigation starts, first commits, closures, fresh authorities, and browser-observed post-conditions must remain attributable to B-new throughout. Command ACK alone is not acceptance evidence.

## Traceability

- Production Browser Session lifecycle owner: PR #317.
- Navigation acceptance parent: PR #318.
- WebDriver BiDi correlation owner: PR #316.
- Navigation-to-download liveness issue: #320.
- Primary lifecycle doctoring: `docs/doctoring/browser-session-navigation-lifecycle.md`.
