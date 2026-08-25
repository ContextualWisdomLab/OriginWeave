# ADR 0017: Enterprise maker-checker approval lifecycle

- Status: Proposed
- Date: 2026-08-23
- Supersedes: none
- Superseded by: none

## Context

OriginWeave already binds approval policy to an immutable `ApprovalScope` containing the action kind, canonical target origin, and complete canonical action-intent digest. Enterprise operation additionally needs a maker-checker lifecycle that can express who requested a bounded approval, who independently decided it, when that decision is valid, how many uses it permits, and when it becomes terminal.

A lifecycle counter alone is insufficient if successful consumption returns ordinary reusable approval evidence. `ApprovalEvidence` is intentionally a reusable policy-context value for other authority sources; returning it directly from a bounded enterprise request would allow a caller to retain or clone that evidence and evaluate the same approved scope again after the lifecycle has consumed its configured use count or expired. That would separate the recorded lifecycle state from effective execution authority.

This decision extends, but does not replace, the Accepted agent-safety model in ADR 0002. It defines a branch-local proposed enterprise authority primitive. Protected-main source and live repository policy remain authoritative until this proposal is reviewed and integrated.

## Decision drivers

- Bind every delegated enterprise approval to the exact immutable action/origin/intent identity that will be evaluated.
- Enforce separation of duties between the requesting maker and deciding checker.
- Make expiry and transition ordering deterministic under a trusted control-plane clock.
- Make denial, withdrawal, expiry, exhaustion, and revocation fail-closed terminal states.
- Enforce the configured bounded-use count at the same authority boundary that produces executable policy authority.
- Prevent a successfully consumed use from becoming replayable merely because surrounding policy context or generic approval evidence is cloneable.
- Revalidate approval lifetime immediately before policy evaluation so a pre-expiry consume cannot authorize after the deadline.
- Keep R5 legal consent non-delegable.
- Avoid introducing authentication, persistence, signing, workflow, release, or ambient authority into the policy crate.

## Assumptions and authority boundaries

`ApprovalPrincipalRef` is an opaque `(issuer, subject)` tuple supplied by an already trusted identity boundary. This crate validates only bounded canonical representation and does not authenticate principals, merge identities by mutable attributes such as email address, or discover tenant membership.

Before calling `EnterpriseApprovalRequest::approve` or `EnterpriseApprovalRequest::deny`, the trusted identity or workflow boundary must verify that the proposed checker has the required checker role, belongs to the request's authoritative tenant, and is authorized for the exact approval scope. Those lifecycle methods enforce requester/checker identity separation and state/time invariants only; they do not establish checker eligibility, tenant membership, or policy scope by themselves.

All lifecycle timestamps are supplied by a trusted control-plane clock. Model output, page content, browser content, or other untrusted inputs must not supply authoritative lifecycle time. Accepted transitions require non-decreasing trusted time; the expiry deadline is exclusive. A consumed approval use retains its consumption time and the same exclusive expiry deadline so the consuming policy evaluation can revalidate trusted time immediately before introducing approval evidence.

The lifecycle does not persist state, acquire clocks, deliver approvals, render UI, sign evidence, resolve external identity, grant release authority, or authorize any action by itself. Normal `originweave-policy` capability, origin, mode, purpose, robots, secret, and risk gates still apply.

## Options considered

### Return reusable `ApprovalEvidence` from `consume`

Rejected. Even when the lifecycle request itself is non-cloneable, a caller could retain or clone the returned evidence and reuse effective approval after lifecycle exhaustion. The accounting state and executable authority would no longer be coupled.

### Store approval evidence permanently in the caller's `PolicyContext`

Rejected. `PolicyContext` is a reusable policy input and is cloneable by design. Mutating it with enterprise approval evidence would make the bounded enterprise use replayable and would implicitly widen the lifetime of authority.

### Return a linear, non-cloneable approval-use value

Selected. A successful lifecycle consumption produces exactly one `EnterpriseApprovalUse`. Its policy-evaluation operation consumes `self`, requires current trusted time, rejects time rollback or expiry, injects the exact approved scope only into a private cloned context for that one evaluation, and delegates to the ordinary fail-closed evaluator.

## Decision

`EnterpriseApprovalRequest` is non-cloneable and owns the mutable lifecycle accounting state. It is created for exactly one immutable `ApprovalScope`, requester, trusted validity window, and nonzero `max_uses`. R5 `LegalConsent` is rejected at construction.

A pending request may be approved or denied only by a principal distinct from the maker. The maker alone may withdraw a pending request. An approved request may be revoked only by the checker that approved it. State validation occurs before transition-specific mutation; trusted transition time must not move backward; and a transition at or after the exclusive expiry deadline moves the live request to `Expired` and fails closed.

`consume` is permitted only from `Approved`, before expiry, and for an exactly equal `ApprovalScope`. A scope mismatch does not spend a use. A successful consume increments lifecycle accounting immediately and returns a non-cloneable `EnterpriseApprovalUse` that retains the exact scope, consumption time, and exclusive expiry deadline. The request becomes `Consumed` when the configured use count is exhausted.

`EnterpriseApprovalUse::evaluate_at(self, request, context, now_epoch_seconds)` consumes the approval-use value. It first rejects trusted time earlier than the recorded consumption time with `NonMonotonicTime` and rejects evaluation at or after the retained exclusive deadline with `Expired`. Only then does it clone the supplied policy context privately, install `ApprovalEvidence::UserConfirmed` for the retained exact scope in that private copy, and delegate to the normal deterministic policy evaluator. The caller's reusable context is not upgraded. The approval use is burned regardless of whether evaluation returns a policy decision or fails the evaluation-time validity checks.

No public API converts `EnterpriseApprovalUse` back into reusable `ApprovalEvidence`, exposes its retained scope for later reinjection, or implements `Clone`/`Copy` for it. There is no untimed evaluation entry point that can bypass the retained expiry boundary.

## Consequences

Enterprise callers receive a capability-like one-shot policy input rather than reusable approval evidence. This aligns effective execution authority with lifecycle accounting: each successful consumption can authorize at most one still-valid policy evaluation, and a denied or expired evaluation cannot be retried by replaying the same consumed value.

Callers that previously expected `consume` to return `ApprovalEvidence` must instead pass the returned `EnterpriseApprovalUse` directly to its consuming `evaluate_at` method together with the intended request, ordinary policy context, and trusted current epoch seconds.

The policy crate remains deterministic and I/O-free. Authentication, clock acquisition, durable state, distributed concurrency control, operator workflows, signatures, and tenant authority remain outside this ADR.

## Failure and degraded behavior

The lifecycle fails closed on invalid validity windows, zero use limits, non-delegable actions, invalid state transitions, trusted-time regression, self-approval, requester/checker role mismatch, exact-scope mismatch, and expiry. The consumed-use evaluation repeats the trusted-time regression and expiry checks before it can introduce approval evidence.

Within the live `EnterpriseApprovalRequest` instance, a successful consume spends that use even if downstream policy evaluation denies the action or the resulting one-shot value later fails its evaluation-time validity check. This deliberately prefers loss of a delegated use over replay ambiguity. A caller needing another attempt must obtain another bounded lifecycle use through the authoritative request state rather than recover authority from a failed evaluation.

If process failure occurs after `consume` but before the one-shot evaluation completes, the in-memory request has advanced, but this crate does not persist that state across restart. Crash-safe replay prevention requires an external durable control plane that atomically preserves authoritative consumption state and recovery evidence. It must not be approximated by making the approval use cloneable or replayable.

## Security / privacy / governance impact

The decision narrows enterprise approval authority by coupling each configured use to one non-replayable, still-valid evaluation attempt. It prevents cloning of lifecycle state or consumed execution authority from bypassing `max_uses`, expiry, terminal-state, or revocation semantics, and prevents a token created immediately before expiry from being exercised after its approval deadline.

The decision does not put credentials, secrets, mutable identity attributes, or raw identity-provider tokens into model context. Principal references remain opaque. Legal consent remains non-delegable. Existing origin, capability, secret-broker, and risk gates are unchanged and continue to fail closed independently of enterprise approval.

## Tests and acceptance evidence

The owning PR must retain realistic executable evidence for:

- distinct maker/checker approval of an exact immutable scope;
- rejection of self-approval, role mismatch, scope mutation, expiry, clock regression, and invalid terminal transitions;
- exact bounded multi-use accounting;
- a single configured use yielding exactly one policy evaluation and rejecting subsequent lifecycle consumption;
- a policy denial burning the already consumed one-shot use;
- evaluation at the retained expiry deadline and trusted-time rollback after consumption both failing closed before approval evidence is applied;
- compile-time proof that `EnterpriseApprovalRequest` and `EnterpriseApprovalUse` are not cloneable; and
- exact-head repository contracts, Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, security scanning where applicable, and exact owned-production function/line/region/branch coverage.

Historical or predecessor-head results do not establish acceptance for a changed head.

## Migration and rollback

Call sites must migrate from storing or passing raw enterprise-produced `ApprovalEvidence` to consuming `EnterpriseApprovalUse::evaluate_at` with trusted current time. No persistence migration is introduced by this branch.

A rollback must revert the lifecycle/use API coherently. Reintroducing a direct `consume -> ApprovalEvidence` path, adding `Clone`/`Copy` to lifecycle accounting or consumed-use types, restoring an untimed evaluation path, or mutating a reusable caller policy context with enterprise approval evidence is not an acceptable partial rollback because it reopens replay or post-expiry authority.

## Open follow-ups

Issue #202 remains the owner for the broader enterprise control plane, including trusted principal authentication, tenant identity, durable state, workflow delivery, operator UI, signed/auditable evidence, and crash-safe/distributed consumption semantics. Those additions must preserve the exact-scope, separation-of-duties, monotonic-time, terminal-state, evaluation-time expiry, and one-shot-use invariants defined here.

## Supersession / reversal conditions

Supersede this ADR if OriginWeave adopts a different formally bounded authority object that can prove, under concurrency and crash recovery, that one enterprise approval use cannot authorize more policy evaluations than the authoritative lifecycle permits. Any replacement must retain or strengthen exact intent binding, maker-checker separation, trusted-time ordering, fail-closed terminal states, evaluation-time expiry, R5 non-delegability, and replay resistance.

## References

- [ADR 0002: Agent safety kernel](0002-agent-safety-kernel.md).
- OriginWeave issue #202, enterprise policy and approval control-plane completion criteria.
