# Action Post-Condition Evidence Traceability

- **Documentation status:** Active-PR evidence dossier
- **Canonical owner:** PR #44 (`docs: reconcile architecture documentation fitness`)
- **Protected-main baseline:** `67af7c87589edc2039545af335c95064d9b8391c`
- **Capability maturity:** **PARTIAL**
- **Governing decisions:** Accepted ADR 0003 plus Proposed ADR 0106 preserve provenance-native evidence and separation of action execution from verification.

## 1. Why this dossier exists

OriginWeave's protected-main API contract already defines a durable product rule: returning from a browser command is not equivalent to successful action completion. A state-changing action becomes successful only after the declared or derived post-condition is observed and verified. Protected main also provides generic credential-safe provenance with explicit verification state, but that design rule was not yet represented by a reusable typed action-outcome evidence object.

This dossier records the active implementation evidence that narrows that gap. It does not promote active pull requests to protected-main shipped truth and it does not claim that a real Chromium adapter already observes the post-condition after dispatch.

## 2. Protected-main design and implementation boundary

Protected `main` already provides:

- typed `ActionKind` and immutable `ActionIntentDigest` values;
- canonical `Origin` authority values;
- credential-safe `ProvenanceRecord` with explicit `VerificationResult`;
- API/TRD requirements that state-changing success waits for an observed post-condition; and
- provenance architecture that keeps observation, policy, execution, and verification as distinct authorities.

The generic value primitives are **IMPLEMENTED_ON_PROTECTED_MAIN**. The complete action dispatch → observation → independent verification → successful outcome chain remains **PARTIAL** because protected main does not yet contain the real Chromium runtime that composes them end to end.

## 3. Active executable evidence

### PR #64 — verified, temporally ordered post-condition becomes typed action-outcome evidence

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `2c45411ed9aa0eecca2d06c85659db9f4bb85e4d` adds `VerifiedActionOutcomeEvidence` in the existing credential-safe evidence crate. It binds:

1. the exact typed `ActionKind`;
2. canonical target `Origin`;
3. complete immutable `ActionIntentDigest`;
4. a bounded first-slice `PostConditionKind` (`UrlChanged`, `NodeStateChanged`, `DialogStateChanged`, or `NetworkMutationObserved`);
5. caller-supplied action-dispatch and post-condition-observation timestamps that must come from one monotonic clock domain; and
6. the exact `ProvenanceRecord` used as the post-condition proof.

Construction fails closed unless the supplied provenance has `VerificationResult::Verified`. Both `Unverified` and `Rejected` observations are rejected as `PostConditionNotVerified`. An observation timestamp earlier than dispatch is rejected as `PostConditionPredatesDispatch`; equal ticks remain valid for coarse monotonic clocks.

On this exact head, CI run `31441848670`, Security Scan run `31441848649`, SAST Semgrep run `31441848615`, exact owned production function/line/region/branch coverage, strict Clippy, rustdoc and CodeRabbit exact-head status are successful. GitHub reports the PR mergeable and Ready for review; no formal reviews or inline review threads are currently returned.

### PR #65 — controlled hostile local workflow fixture

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR` only after its fail-first contract is satisfied; current state is **PARTIAL / RED-IN-PROGRESS**.

Test-only head `d2580305f05aba93d10b5342ec1886d601c6752e` is based directly on the protected-main baseline and requires a checked-in `tests/fixtures/agent_task_basic/index.html` that does not yet exist. The contract intentionally requires a labelled semantic field, submit control, deterministic `idle` → `submitted` observable state change, one explicitly hidden/untrusted prompt-injection marker, and no password/OTP/API-key/secret collection surface.

The missing fixture is deliberate fail-first evidence, not a shipped compatibility claim. The lane remains Draft until the exact RED is observed, the smallest controlled fixture is added, and exact-head verification succeeds.

## 4. Non-transitive success semantics

The intended first-slice chain is:

```text
typed action intent
-> policy-authorized dispatch
-> real browser input/event
-> observed bounded post-condition
-> independently verified provenance
-> temporally ordered VerifiedActionOutcomeEvidence
```

The active PR implements only the final typed evidence boundary. The following implications are explicitly invalid:

```text
command return -/> successful action completion
protocol acknowledgement -/> successful action completion
Unverified -/> successful action completion
Rejected -/> successful action completion
caller-supplied timestamp ordering -/> proof of trusted clock provenance
VerifiedActionOutcomeEvidence type existence -/> proof of real Chromium execution
```

PR #64 now rejects a caller-supplied observation timestamp that predates caller-supplied dispatch time, but the type cannot independently prove the clock source, that a real browser actually dispatched the action, that the supplied provenance belongs to the claimed browser target/node, or that the observed state was caused by that action. Those claims remain the responsibility of the real adapter/runtime composition under issue #28.

## 5. Active prerequisite graph for issue #28

The first real Chromium vertical slice remains distributed across bounded active prerequisites rather than one shipped runtime:

- PR #40 — protocol/browser identifiers → OriginWeave session/context/origin/document/node authority;
- PR #52 — bounded semantic node observation with explicit source-channel provenance;
- PR #57 — typed semantic-node query contract;
- PR #58 — authority-bound semantic node action target;
- PR #49 — ephemeral compatibility-profile lifecycle regression stacked on #43;
- PR #51 — bounded browser-task telemetry plus one explicitly supplied Linux PID `VmRSS` sampler; Chromium process discovery/process-set attribution remains outside that slice;
- PR #64 — verified and caller-timestamp-ordered post-condition action-outcome evidence; and
- PR #65 — controlled hostile local Agent Task workflow fixture currently in fail-first Draft state.

These active PRs are non-shipped evidence. They do not themselves compose WebDriver BiDi/CDP transport, trusted Chromium process attribution, policy-authorized real input dispatch, causal post-condition observation, or deterministic end-to-end teardown/recovery into one protected-main runtime.

## 6. Remaining issue #28 boundary

This dossier does **not** close issue #28. Material remaining work includes:

- pinned stock Chromium exercised as one reproducible end-to-end Agent Task runtime path, not only extension compatibility fixtures;
- isolated Agent Task profile/context lifecycle and cleanup in the production vertical path;
- versioned WebDriver BiDi adapter plus explicitly bounded CDP observation fallback where needed;
- real semantic observation feeding typed query and policy-authorized typed action;
- real browser input dispatch followed by post-dispatch observation of the declared condition;
- hostile/stale/cross-session/cross-context/cross-origin/prompt-injection/secret-leak/crash/oversize regressions;
- deterministic failure/recovery evidence and task teardown;
- Chromium process discovery/process-set attribution composed into resource telemetry; and
- protected-main integration plus fresh acceptance before any active-PR capability becomes shipped truth.

## 7. Documentation fitness consequence

The ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PR #64 narrows a typed evidence gap already governed by existing provenance/action-success decisions, while PR #65 supplies controlled test infrastructure for the eventual real-browser proof. Neither introduces a new trust domain, deployed component, persistence owner, database schema, or independent architecture decision, so a new ADR or physical ERD entity would overstate the implementation. Detailed real-Chromium dispatch/post-condition sequence diagrams should be reconciled when the executable adapter chain stabilizes rather than manufacturing as-built detail before that runtime exists.
