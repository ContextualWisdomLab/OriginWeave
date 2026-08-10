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

### PR #64 — verified post-condition becomes typed action-outcome evidence

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `98bb2efba830fb8968331b64cf16929c4005863c` adds `VerifiedActionOutcomeEvidence` in the existing credential-safe evidence crate. It binds:

1. the exact typed `ActionKind`;
2. canonical target `Origin`;
3. complete immutable `ActionIntentDigest`;
4. a bounded first-slice `PostConditionKind` (`UrlChanged`, `NodeStateChanged`, `DialogStateChanged`, or `NetworkMutationObserved`); and
5. the exact `ProvenanceRecord` used as the post-condition proof.

Construction fails closed unless the supplied provenance has `VerificationResult::Verified`. Both `Unverified` and `Rejected` observations are rejected as `PostConditionNotVerified`.

The exact head has successful CI, exact owned production function/line/region/branch coverage, strict Clippy, rustdoc, Security Scan, SAST and CodeRabbit status and is Ready for review.

## 4. Non-transitive success semantics

The intended first-slice chain is:

```text
typed action intent
-> policy-authorized dispatch
-> real browser input/event
-> observed bounded post-condition
-> independently verified provenance
-> VerifiedActionOutcomeEvidence
```

The active PR implements only the final typed evidence boundary. The following implications are explicitly invalid:

```text
command return -/> successful action completion
protocol acknowledgement -/> successful action completion
Unverified -/> successful action completion
Rejected -/> successful action completion
VerifiedActionOutcomeEvidence type existence -/> proof of real Chromium execution
```

A future adapter must establish the event ordering and observation source before constructing the evidence. The type cannot by itself prove that a real browser was dispatched, that the observation happened after the dispatch, or that the observed state was caused by that action.

## 5. Active prerequisite graph for issue #28

The first real Chromium vertical slice remains distributed across bounded active prerequisites rather than one shipped runtime:

- PR #40 — protocol/browser identifiers → OriginWeave session/context/origin/document/node authority;
- PR #52 — bounded semantic node observation with explicit source-channel provenance;
- PR #57 — typed semantic-node query contract;
- PR #58 — authority-bound semantic node action target;
- PR #51 — bounded real-adapter telemetry value for RSS/observation bytes/action latency/task duration; and
- PR #64 — verified post-condition action-outcome evidence.

These active PRs are non-shipped evidence. They do not themselves launch Chromium, implement WebDriver BiDi/CDP transport, dispatch a complete real input lifecycle, or prove deterministic teardown/recovery.

## 6. Remaining issue #28 boundary

This dossier does **not** close issue #28. Material remaining work includes:

- pinned stock Chromium exercised as a reproducible supported runtime path;
- isolated Agent Task profile/context lifecycle and cleanup in the production vertical path;
- versioned WebDriver BiDi adapter plus explicitly bounded CDP observation fallback where needed;
- real semantic observation feeding typed query and policy-authorized typed action;
- real browser input dispatch followed by post-dispatch observation of the declared condition;
- hostile/stale/cross-session/cross-context/cross-origin/prompt-injection/secret-leak/crash/oversize regressions;
- deterministic failure/recovery evidence and task teardown;
- actual adapter resource telemetry; and
- protected-main integration plus fresh acceptance before any active-PR capability becomes shipped truth.

## 7. Documentation fitness consequence

The ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PR #64 narrows a typed evidence gap already governed by existing provenance/action-success decisions. It does not introduce a new trust domain, deployment component, persistence owner, database schema, or independent architecture decision, so a new ADR or physical ERD entity would overstate the implementation. Detailed real-Chromium dispatch/post-condition sequence diagrams should be reconciled when the executable adapter chain stabilizes rather than manufacturing as-built detail before that runtime exists.