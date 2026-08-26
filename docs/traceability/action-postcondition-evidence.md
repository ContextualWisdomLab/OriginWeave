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

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Test-only head `d2580305f05aba93d10b5342ec1886d601c6752e` was based directly on the protected-main baseline and intentionally required a checked-in `tests/fixtures/agent_task_basic/index.html` before that fixture existed. CI run `31445088008`, Rust contracts job `93637443229`, checked out that exact head and failed with three `FileNotFoundError` results for the missing fixture, establishing the intended fail-first boundary.

Exact head `0888fe3a6ef6da547a37fd075733cc73dc52b2ab` adds the smallest controlled fixture satisfying the contract: a labelled semantic field, submit control, deterministic `idle` → `submitted` observable state change carrying only synthetic text, one explicitly hidden/untrusted prompt-injection marker, and no password/OTP/API-key/secret collection surface.

On that unchanged exact head, CI run `31445201739` succeeds; Rust contracts job `93637824750` passes repository contracts, formatting, locked workspace check, full tests, strict Clippy and rustdoc; Production coverage job `93637824824` passes exact owned production function/line/region/branch enforcement; Security Scan run `31445201774`, SAST Semgrep run `31445201669` and CodeRabbit exact-head status succeed. GitHub reports the PR mergeable and Ready for review with no formal reviews or inline review threads currently returned.

This remains controlled test infrastructure rather than browser-execution evidence. The fixture itself does not establish WebDriver BiDi/CDP transport, Chromium semantic extraction, policy dispatch, native input, post-condition provenance, profile teardown or process attribution.

### PR #70 — pinned Chrome execution of the controlled Agent Task fixture

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #70 reuses the existing pinned Chrome for Testing workflow and executes the #65 fixture through loopback ChromeDriver with extensions disabled and a fresh temporary profile. Each bounded trial performs real WebDriver clear/type/click operations, observes the `submitted` state and synthetic value through element endpoints, verifies that submission preserves the loaded URL, and proves that the temporary profile is removed after teardown. The runner emits credential-free repeatability evidence and fails the lane when any trial or post-condition is incomplete.

This is real WebDriver evidence for a controlled local fixture, not a product browser adapter. It does not establish WebDriver BiDi/CDP authority translation, OriginWeave semantic observation or node handles, policy-authorized typed action dispatch, trusted browser-process attribution, or protected-main product runtime completion.

### PR #71 — browser-computed semantic role/name evidence before action

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #71 extends the pinned-Chrome fixture lane by reading WebDriver's browser-computed role and accessible name for the controlled input and submit button before sending input or clicking. The exact expected values are `textbox` / `Task text` and `button` / `Submit task`; the repeatability gate requires both semantic checks in every successful trial.

This is bounded browser-computed evidence for a synthetic test target, not the OriginWeave semantic observation adapter. CSS locators remain test-harness selectors, and the lane does not create OriginWeave node handles, source-channel provenance, policy authority, or permission to execute page-advertised actions.

### PR #72 — bounded Agent Task resource evidence

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

PR #72 records browser-process RSS, semantic-observation bytes, action latency, and total task duration while the pinned-Chrome fixture runs. The measurements are bounded, positive observations from the trusted ChromeDriver process identifier and the controlled semantic payload; they make the real fixture's resource and timing evidence inspectable without introducing a new telemetry subsystem.

This is resource evidence for the active test harness, not process-set attribution or a product resource adapter. It does not discover Chromium children, prove task ownership or ancestry, walk cgroups, sample GPU/VRAM, or export durable product telemetry.

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
controlled fixture success -/> proof of an OriginWeave product browser runtime
```

PR #64 now rejects a caller-supplied observation timestamp that predates caller-supplied dispatch time, but the type cannot independently prove the clock source, that a real browser actually dispatched the action, that the supplied provenance belongs to the claimed browser target/node, or that the observed state was caused by that action. PR #70 proves real Chromium execution against the controlled fixture, PR #71 adds browser-computed role/name evidence, and PR #72 adds bounded resource evidence, but their test-harness CSS locators, direct WebDriver calls, and fixture-scoped measurements are not the OriginWeave adapter/runtime composition required under issue #28.

## 5. Active prerequisite graph for issue #28

The first real Chromium vertical slice remains distributed across bounded active prerequisites rather than one shipped runtime:

- PR #40 — protocol/browser identifiers → OriginWeave session/context/origin/document/node authority;
- PR #52 — bounded semantic node observation with explicit source-channel provenance;
- PR #57 — typed semantic-node query contract;
- PR #58 — authority-bound semantic node action target;
- PR #49 — ephemeral compatibility-profile lifecycle regression stacked on #43;
- PR #51 — bounded browser-task telemetry plus one explicitly supplied Linux PID `VmRSS` sampler; Chromium process discovery/process-set attribution remains outside that slice;
- PR #64 — verified and caller-timestamp-ordered post-condition action-outcome evidence; and
- PR #65 — controlled hostile local Agent Task workflow fixture; and
- PR #70 — real WebDriver execution of that fixture on pinned Chrome, without claiming a product browser adapter; and
- PR #71 — browser-computed role/name evidence before controlled action, without claiming a product semantic observer; and
- PR #72 — bounded browser-process RSS, semantic-observation byte, latency, and task-duration resource evidence, without claiming process-set attribution or a product resource adapter.

These active PRs are non-shipped evidence. PR #70/#71/#72 prove bounded browser-level, semantic, and resource evidence, but the active set does not itself compose WebDriver BiDi/CDP transport, OriginWeave authority translation, trusted Chromium process attribution, policy-authorized real input dispatch, causal post-condition observation, or deterministic end-to-end teardown/recovery into one protected-main runtime.

## 6. Remaining issue #28 boundary

This dossier does **not** close issue #28. Material remaining work includes:

- a production Agent Task runtime path that composes pinned stock Chromium with OriginWeave authority, rather than only the controlled #70 fixture and extension compatibility fixtures;
- isolated Agent Task profile/context lifecycle and cleanup in the production vertical path;
- versioned WebDriver BiDi adapter plus explicitly bounded CDP observation fallback where needed;
- real semantic observation feeding typed query and policy-authorized typed action;
- real browser input dispatch followed by post-dispatch observation of the declared condition;
- hostile/stale/cross-session/cross-context/cross-origin/prompt-injection/secret-leak/crash/oversize regressions;
- deterministic failure/recovery evidence and task teardown;
- Chromium process discovery/process-set attribution composed into resource telemetry; and
- protected-main integration plus fresh acceptance before any active-PR capability becomes shipped truth.

## 7. Documentation fitness consequence

The ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PR #64 narrows a typed evidence gap, PR #65 supplies the controlled fixture, and PR #70/#71 supply real WebDriver and browser-computed semantic evidence for that fixture. Neither introduces a new trust domain, deployed component, persistence owner, database schema, or independent architecture decision, so a new ADR or physical ERD entity would overstate the implementation. Detailed real-Chromium dispatch/post-condition sequence diagrams should be reconciled when the executable adapter chain stabilizes rather than manufacturing as-built detail before that runtime exists.
