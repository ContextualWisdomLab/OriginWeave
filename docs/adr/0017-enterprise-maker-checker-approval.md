# ADR 0017: Enterprise maker-checker approval lifecycle

- Status: Proposed
- Date: 2026-08-23
- Supersedes: none
- Superseded by: none

## Context

OriginWeave already binds approval policy to an immutable `ApprovalScope` containing the action kind, canonical target origin, and complete canonical action-intent digest. Enterprise operation additionally needs a maker-checker lifecycle that can express who requested a bounded approval, who independently decided it, when that decision is valid, how many uses it permits, and when it becomes terminal.

A lifecycle counter alone is insufficient if successful consumption returns ordinary reusable approval evidence. `ApprovalEvidence` is intentionally a reusable policy-context value for other authority sources; returning it directly from a bounded enterprise request would allow a caller to retain or clone that evidence and evaluate the same approved scope again after the lifecycle has consumed its configured use count or expired. That would separate the recorded lifecycle state from effective execution authority.

A second split can occur when an approved request issues a one-shot use and the approving checker revokes before that use is evaluated. That risk remains when the issued use is the final configured use and the live request has already entered `Consumed`: issuance exhaustion is not proof that execution finished. If the issued use is detached from later revocation state, it can remain effective even though the checker has withdrawn the delegated authority. Revocation therefore has to invalidate outstanding, not-yet-evaluated uses whether the request is still `Approved` or has become `Consumed` because all configured uses were issued.

The same detached-use problem exists when the authoritative request later observes its expiry deadline. An outstanding use still performs its own deadline check, but without sharing that observed terminal state a caller could advance the request's trusted timeline to `Expired` and then present an earlier, locally valid evaluation timestamp to the detached use. Request-observed expiry therefore must also invalidate outstanding in-process uses so backdated evaluation cannot resurrect authority after the lifecycle has already become terminal.

This decision extends, but does not replace, the Accepted agent-safety model in ADR 0002. It defines a branch-local proposed enterprise authority primitive. Protected-main source and live repository policy remain authoritative until this proposal is reviewed and integrated.

## Decision drivers

- Bind every delegated enterprise approval to the exact immutable action/origin/intent identity that will be evaluated.
- Enforce separation of duties between the requesting maker and deciding checker.
- Make expiry and transition ordering deterministic under a trusted control-plane clock.
- Make denial, withdrawal, expiry, exhaustion, and revocation fail-closed terminal states.
- Enforce the configured bounded-use count at the same authority boundary that produces executable policy authority.
- Prevent a successfully consumed use from becoming replayable merely because surrounding policy context or generic approval evidence is cloneable.
- Revalidate approval lifetime immediately before policy evaluation so a pre-expiry consume cannot authorize after the deadline.
- Invalidate an outstanding one-shot use when its approving checker revokes before evaluation begins, including after the final configured use has been issued.
- Invalidate outstanding in-process uses when the authoritative request observes expiry, even if a later evaluator supplies a backdated timestamp that is individually after that use's consumption time and before the retained deadline.
- Keep R5 legal consent non-delegable.
- Avoid introducing authentication, persistence, signing, workflow, release, or ambient authority into the policy crate.

## Assumptions and authority boundaries

`ApprovalPrincipalRef` is an opaque `(issuer, subject)` tuple supplied by an already trusted identity boundary. This crate validates only a bounded canonical representation and does not authenticate principals, merge identities by mutable attributes such as email address, or discover tenant membership. The canonical representation rejects control characters and the Unicode Standard Annex #9 `Bidi_Control` set (directional marks, embeddings, overrides, and isolates) so a logically distinct principal reference cannot rely on hidden directional formatting to present misleading issuer/subject text in operator or audit surfaces. Other Unicode remains opaque; this crate does not perform identity normalization or confusable folding.

Before calling `EnterpriseApprovalRequest::approve` or `EnterpriseApprovalRequest::deny`, the trusted identity or workflow boundary must verify that the proposed checker has the required checker role, belongs to the request's authoritative tenant, is authorized for the exact approval scope, and resolves to a distinct canonical human or workload actor from the maker. Exact `(issuer, subject)` inequality inside this crate is not sufficient separation-of-duties evidence when one real actor can hold aliases or multiple federated identities; canonical actor correlation and alias/account-link governance belong to that trusted boundary. Those lifecycle methods enforce requester/checker tuple separation and state/time invariants only; they do not establish actor uniqueness, checker eligibility, tenant membership, or policy scope by themselves.

All lifecycle timestamps are supplied by a trusted control-plane clock. Model output, page content, browser content, or other untrusted inputs must not supply authoritative lifecycle time. Accepted transitions require non-decreasing trusted time; the expiry deadline is exclusive. A consumed approval use retains its consumption time and the same exclusive expiry deadline so the consuming policy evaluation can revalidate trusted time immediately before introducing approval evidence.

The live request and its issued uses also share a one-way in-memory terminal invalidation signal. A successful checker revocation records `Revoked` before the request enters `Revoked`; any request transition that observes the exclusive deadline records `Expired` before the request enters `Expired`. An issued use checks that shared terminal signal before introducing approval evidence. This is process-local coordination only. It does not provide durable revocation or expiry propagation, distributed consensus, crash recovery, or cross-process invalidation.

The lifecycle does not persist state, acquire clocks, deliver approvals, render UI, sign evidence, resolve external identity, grant release authority, or authorize any action by itself. Normal `originweave-policy` capability, origin, mode, purpose, robots, secret, and risk gates still apply.

## Options considered

### Return reusable `ApprovalEvidence` from `consume`

Rejected. Even when the lifecycle request itself is non-cloneable, a caller could retain or clone the returned evidence and reuse effective approval after lifecycle exhaustion. The accounting state and executable authority would no longer be coupled.

### Store approval evidence permanently in the caller's `PolicyContext`

Rejected. `PolicyContext` is a reusable policy input and is cloneable by design. Mutating it with enterprise approval evidence would make the bounded enterprise use replayable and would implicitly widen the lifetime of authority.

### Return a linear, non-cloneable approval-use value

Selected. A successful lifecycle consumption produces exactly one `EnterpriseApprovalUse`. Its policy-evaluation operation consumes `self`, requires current trusted time, rejects any request whose action/origin/intent differs from the retained exact scope before exposing lifecycle or time state, then rejects time rollback, direct deadline expiry, or a shared terminal expiry/revocation observed by the issuing request before evaluation begins. Only a still-valid exact-scope use injects approval into a private cloned context for that one evaluation and delegates to the ordinary fail-closed evaluator.

## Decision

`EnterpriseApprovalRequest` is non-cloneable and owns the mutable lifecycle accounting state. It is created for exactly one immutable `ApprovalScope`, requester, trusted validity window, and nonzero `max_uses`. R5 `LegalConsent` is rejected at construction.

A pending request may be approved or denied only by a principal distinct from the maker. The maker alone may withdraw a pending request. After approval, the exact approving checker may revoke while the request is `Approved` or after all configured uses have been issued and the request is `Consumed`. State validation occurs before transition-specific mutation; trusted transition time must not move backward; and a transition at or after the exclusive expiry deadline moves the live request to `Expired` and fails closed. A revocation after `Consumed` invalidates any issued use that has not yet begun its evaluation-time validity check; it does not retroactively undo policy evaluations completed before revocation.

`consume` is permitted only from `Approved`, before expiry, and for an exactly equal `ApprovalScope`. A scope mismatch does not spend a use. A successful consume increments lifecycle accounting immediately and returns a non-cloneable `EnterpriseApprovalUse` that retains the exact scope, consumption time, exclusive expiry deadline, and a shared one-way terminal invalidation signal. The request becomes `Consumed` when the configured use count is exhausted. If a later consume attempt observes the expiry deadline while the request is still `Approved`, it records shared `Expired` invalidation before entering `Expired`; outstanding uses from the same live request then fail closed even if their evaluator supplies an earlier timestamp.

`EnterpriseApprovalUse::evaluate_at(self, request, context, now_epoch_seconds)` consumes the approval-use value. It first reconstructs the exact `ApprovalScope` from the supplied request's action, canonical target origin, and immutable action-intent digest and returns `ScopeMismatch` if that scope differs from the retained approved scope. This scope check intentionally precedes lifecycle and trusted-time checks so an unrelated request cannot use the token to infer expiry or terminal state. For an exact-scope request, evaluation rejects trusted time earlier than the recorded consumption time with `NonMonotonicTime`, rejects evaluation at or after the retained exclusive deadline with `Expired`, and then checks the issuing request's shared terminal invalidation. An observed `Expired` invalidation returns `Expired`; an observed `Revoked` invalidation returns `InvalidState(Revoked)`. Only then does it clone the supplied policy context privately, install `ApprovalEvidence::UserConfirmed` for the retained exact scope in that private copy, and delegate to the normal deterministic policy evaluator. The caller's reusable context is not upgraded. The approval use is burned regardless of whether evaluation returns a policy decision or fails scope, time, expiry, or terminal-invalidation validation.

The terminal invalidation signal is intentionally one-way and process-local. Once set it cannot be cleared. An outstanding use that begins its validity check after request-observed expiry or checker revocation fails closed according to the first shared terminal condition recorded by the live request. An evaluation that has already passed that validity check is considered in flight; stronger cross-process or transactional cancellation semantics belong to the durable enterprise control plane under issue #202.

No public API converts `EnterpriseApprovalUse` back into reusable `ApprovalEvidence`, exposes its retained scope for later reinjection, or implements `Clone`/`Copy` for it. There is no untimed evaluation entry point that can bypass the retained scope, expiry, or terminal-invalidation boundary.

## Consequences

Enterprise callers receive a capability-like one-shot policy input rather than reusable approval evidence. This aligns effective execution authority with lifecycle accounting: each successful consumption can authorize at most one still-valid, exact-scope policy evaluation, and a scope mismatch, policy denial, expiry, or revocation cannot be retried by replaying the same consumed value.

Callers that previously expected `consume` to return `ApprovalEvidence` must instead pass the returned `EnterpriseApprovalUse` directly to its consuming `evaluate_at` method together with the intended request, ordinary policy context, and trusted current epoch seconds.

The policy crate remains deterministic and I/O-free. Authentication, clock acquisition, durable state, distributed concurrency control, operator workflows, signatures, and tenant authority remain outside this ADR.

## Failure and degraded behavior

The lifecycle fails closed on invalid validity windows, zero use limits, non-delegable actions, invalid state transitions, trusted-time regression, self-approval, requester mismatch, decision-actor mismatch, exact-scope mismatch, and expiry. Checker-role, tenant-membership, actor-uniqueness, and business-authorization failures must already have failed closed at the trusted identity/workflow boundary before an approval or denial enters this lifecycle. The consumed-use evaluation repeats exact request-scope binding before trusted-time regression, direct expiry, and shared terminal invalidation checks so a mismatched request neither gains authority nor learns lifecycle/time state.

Within the live `EnterpriseApprovalRequest` instance, a successful consume spends that use even if downstream policy evaluation denies the action or the resulting one-shot value later fails its evaluation-time validity check. This deliberately prefers loss of a delegated use over replay ambiguity. A caller needing another attempt must obtain another bounded lifecycle use through the authoritative request state rather than recover authority from a failed evaluation.

If process failure occurs after `consume` but before the one-shot evaluation completes, the in-memory request has advanced, but this crate does not persist that state or its terminal invalidation signal across restart. Crash-safe replay, expiry propagation, and revocation prevention require an external durable control plane that atomically preserves authoritative consumption/expiry/revocation state and recovery evidence. It must not be approximated by making the approval use cloneable or replayable.

## Security / privacy / governance impact

The decision narrows enterprise approval authority by coupling each configured use to one non-replayable, exact-scope, still-valid evaluation attempt. It prevents cloning of lifecycle state or consumed execution authority from bypassing `max_uses`, exact scope, expiry, terminal-state, or revocation semantics; prevents an unrelated low-risk request from bypassing scope binding; prevents a mismatched request from learning expiry/terminal state before receiving `ScopeMismatch`; prevents a token created immediately before expiry from being exercised after its approval deadline; prevents an already-issued but not-yet-evaluated token from surviving a successful checker revocation in the same live process even when that token was the final configured use; and prevents a caller from resurrecting an outstanding token with a backdated timestamp after the live request has already observed expiry. Principal references additionally reject Unicode `Bidi_Control` formatting characters so invisible direction overrides or isolates cannot create a misleading displayed identity while retaining a different exact `(issuer, subject)` tuple.

The decision does not put credentials, secrets, mutable identity attributes, or raw identity-provider tokens into model context. Principal references remain opaque. Legal consent remains non-delegable. Existing origin, capability, secret-broker, and risk gates are unchanged and continue to fail closed independently of enterprise approval.

## Tests and acceptance evidence

The owning PR must retain realistic executable evidence for:

- distinct maker/checker approval of an exact immutable scope;
- rejection of non-canonical principal references including control and Unicode `Bidi_Control` formatting characters;
- rejection of self-approval, requester mismatch, decision-actor mismatch, scope mutation, expiry, clock regression, and invalid terminal transitions;
- rejection of a consumed use presented to a different low-risk request before lifecycle/time state is exposed;
- exact bounded multi-use accounting;
- a single configured use yielding exactly one policy evaluation and rejecting subsequent lifecycle consumption;
- a policy denial burning the already consumed one-shot use;
- evaluation at the retained expiry deadline and trusted-time rollback after consumption both failing closed before approval evidence is applied;
- checker revocation after one use was issued from a still-live multi-use request invalidating that unexecuted use before approval evidence is applied;
- checker revocation after the final configured use was issued invalidating that still-outstanding use before approval evidence is applied;
- request-observed expiry after an earlier use was issued invalidating that outstanding use even when evaluation later supplies a backdated timestamp inside the use's original local validity window;
- expiry observed through the revocation transition invalidating an already-issued use under the same backdated-evaluation attempt;
- compile-time proof that `EnterpriseApprovalRequest` and `EnterpriseApprovalUse` are not cloneable; and
- exact-head repository contracts, Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, security scanning where applicable, and exact owned-production function/line/region/branch coverage.

Historical or predecessor-head results do not establish acceptance for a changed head.

## Migration and rollback

Call sites must migrate from storing or passing raw enterprise-produced `ApprovalEvidence` to consuming `EnterpriseApprovalUse::evaluate_at` with the exact intended request and trusted current time. No persistence migration is introduced by this branch.

A rollback must revert the lifecycle/use API coherently. Reintroducing a direct `consume -> ApprovalEvidence` path, adding `Clone`/`Copy` to lifecycle accounting or consumed-use types, restoring an untimed evaluation path, removing exact request-scope revalidation, detaching issued uses from live in-process terminal expiry/revocation invalidation, or mutating a reusable caller policy context with enterprise approval evidence is not an acceptable partial rollback because it reopens replay, scope-confusion/privacy, post-expiry, or post-revocation authority.

## Open follow-ups

Issue #202 remains the owner for the broader enterprise control plane, including trusted principal authentication, tenant identity, durable state, workflow delivery, operator UI, signed/auditable evidence, and crash-safe/distributed consumption, expiry, and revocation semantics. Those additions must preserve the exact-scope, separation-of-duties, monotonic-time, terminal-state, evaluation-time expiry, in-process outstanding-use terminal invalidation, one-shot-use, and canonical-principal-display invariants defined here.

## Supersession / reversal conditions

Supersede this ADR if OriginWeave adopts a different formally bounded authority object that can prove, under concurrency and crash recovery, that one enterprise approval use cannot authorize more policy evaluations than the authoritative lifecycle permits. Any replacement must retain or strengthen exact intent binding, maker-checker separation, trusted-time ordering, fail-closed terminal states, evaluation-time scope/expiry/revocation, R5 non-delegability, replay resistance, and principal-reference presentation safety.

## References

- [ADR 0002: Agent safety kernel](0002-agent-safety-kernel.md).
- [Research and standards doctoring](../doctoring.md), including Unicode Standard Annex #9.
- OriginWeave issue #202, enterprise policy and approval control-plane completion criteria.
