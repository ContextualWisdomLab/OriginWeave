# ADR 0016: BAP task lifecycle and state authority

- **Status:** Proposed
- **Date:** 2026-08-22
- **Supersedes:** None
- **Superseded by:** None

## Context

OriginWeave needs a deterministic lifecycle primitive for governed browser-agent work before durable BAP transport, persistence, idempotency, or crash recovery can be added safely. A task state is security-relevant because downstream components may use it to decide whether work may start, resume, complete, reconcile, or terminate. If adapters, persistence layers, browser drivers, or recovery code can mint state independently, OriginWeave would inherit ambient execution authority from whichever boundary supplied the most convenient state value.

The `originweave-bap` crate therefore introduces a typed in-memory state machine with monotonic transition receipts and fail-closed recovery validation. The crate deliberately owns no browser, network, model, secret, approval, persistence, tenant-authentication, or protocol authority. External protocols may project lifecycle intent into this kernel, but protocol metadata cannot bypass its transition rules or upgrade a task's authority.

## Decision drivers

- Keep task-state authority explicit and deterministic rather than distributed across protocol adapters.
- Prevent stale, unreachable, or terminal lifecycle snapshots from reopening governed work.
- Preserve a monotonic transition sequence suitable for later durable replay evidence without claiming persistence today.
- Separate lifecycle state from browser, network, secret, model, approval, and tenant authority.
- Make waiting, checkpoint, reconciliation, completion, cancellation, expiry, and dead-letter behavior typed and testable.
- Keep recovery validation fail closed when a supplied state/sequence pair cannot arise from the reviewed state machine.

## Assumptions and authority boundaries

- The lifecycle is an in-memory logical primitive; it is not a durable task repository.
- Creating or restoring a lifecycle does not authenticate a caller, tenant, browser session, document, origin, destination, secret, model, approval, or external side effect.
- A transition receipt proves only what this in-memory lifecycle instance accepted. It is not durable audit evidence until a separate authenticated persistence boundary stores it.
- Waiting for approval is a lifecycle condition, not proof that approval exists. A later approval authority must independently authenticate and authorize any decision before resumption.
- `Succeeded` is entered only after a caller asserts that its separately governed post-condition has been verified; the lifecycle does not itself verify that post-condition.
- Reconciliation and dead-letter states preserve control-flow intent only. Durable reconciliation evidence remains the responsibility of a later persistence/recovery boundary.

## Options considered

### Let each BAP or MCP adapter own its own state machine

Rejected. Adapter-local state machines would duplicate policy, make recovery semantics drift by protocol, and allow external protocol metadata to become implicit OriginWeave execution authority.

### Store task state as an unrestricted string or integer

Rejected. Untyped state admits unknown values, weakens exhaustive transition review, and makes invalid or stale recovery snapshots difficult to reject deterministically.

### Allow restored state to resume whenever the state name looks resumable

Rejected. State-only recovery loses monotonic history. A state/sequence pair that cannot be reached through the reviewed transitions must fail closed rather than becoming execution authority.

### Centralize logical lifecycle transitions in a typed Rust kernel

Selected.

## Decision

If Accepted, OriginWeave applies these lifecycle rules:

1. **One typed kernel owns logical BAP task state.** `originweave-bap` is the canonical state-transition authority for the task lifecycle represented by this contract. Protocol adapters may request transitions but do not mint lifecycle state directly.
2. **Transitions are explicit and fail closed.** The kernel accepts only reviewed event/state combinations. Invalid events preserve the existing state and sequence and return a typed error.
3. **Terminal states never reopen.** `Succeeded`, `Failed`, `Cancelled`, `Expired`, and `DeadLettered` reject later lifecycle events.
4. **Waiting and checkpoint states require explicit resumption.** Approval wait, external-input wait, and checkpoint states do not silently become running work.
5. **Reconciliation is distinct from normal suspension.** A task in `ReconciliationRequired` cannot use the ordinary resume path; it requires explicit reconciliation resolution or governed dead-letter handling.
6. **Transition sequence is monotonic and bounded.** Every accepted transition advances the sequence exactly once. Sequence exhaustion fails closed instead of wrapping.
7. **Recovery validates reachability.** A supplied state/sequence snapshot must be reachable under the same reviewed state machine. Unreachable snapshots are rejected with a typed restore error.
8. **Lifecycle state grants no ambient authority.** A `Running`, resumable, or otherwise valid lifecycle state does not authorize browser I/O, network destinations, secret resolution, model access, approvals, external protocol operations, or tenant access. Those authorities must be revalidated by their owning boundaries.
9. **Durability is a separate owner.** This contract does not claim atomic persistence, idempotency, locking, authenticated replay evidence, side-effect reconciliation, or crash-safe recovery. Later durable components must bind those concerns to lifecycle receipts without weakening this state authority.
10. **External protocol state is projected, not inherited.** BAP, MCP, WebDriver BiDi, CDP, or other adapters may translate reviewed external events into typed lifecycle requests only after their own authentication and policy checks. External state labels cannot overwrite the kernel directly.

## Consequences

OriginWeave gains one reviewable state authority that later transport, idempotency, persistence, and recovery slices can compose without duplicating transition semantics. Invalid transitions and unreachable recovery snapshots have deterministic typed failures, while terminal and reconciliation states have explicit closure behavior.

The trade-off is that adapters and durable stores must perform explicit mapping and validation instead of assigning state directly. The current slice also cannot claim commercial crash recovery until durable authenticated evidence and side-effect reconciliation are implemented separately.

## Failure and degraded behavior

- An invalid event returns a typed transition error and leaves state/history unchanged.
- A terminal lifecycle rejects all later events rather than reopening work.
- Sequence exhaustion returns a typed failure rather than wrapping or silently reusing an identifier.
- An unreachable restored state/sequence pair is rejected rather than normalized into a nearby valid state.
- Missing browser, tenant, policy, destination, secret, approval, persistence, or recovery authority is not converted into lifecycle success.
- If a future adapter cannot map external protocol state without ambiguity, it must fail closed or require reconciliation rather than inventing a lifecycle transition.

## Security / privacy / governance impact

This decision narrows authority. It prevents external protocol metadata, stale snapshots, or arbitrary state assignment from becoming execution authority and keeps lifecycle state separate from sensitive-data, secret, browser, network, model, approval, and tenant boundaries. The lifecycle stores no secret values or personal-data payloads by itself. Any future persistent representation must independently satisfy OriginWeave data-governance, retention, tenant-isolation, integrity, and evidence requirements.

## Tests and acceptance evidence

The owning branch must keep executable evidence for:

- the reviewed created/admitted/running/waiting/checkpointed/reconciliation/terminal transition paths;
- fail-closed invalid transitions with no sequence advancement;
- terminal irreversibility;
- cancellation and expiry across allowed pre-dispatch and suspended states;
- explicit reconciliation resolution and governed dead-letter behavior;
- monotonic transition receipts and sequence-exhaustion failure;
- recovery acceptance for reachable snapshots and rejection for unreachable snapshots; and
- deterministic public Rust error contracts.

Repository contracts must also require this ADR so the `originweave-bap` control-plane boundary cannot remain undocumented while the crate is present. Exact protected-main acceptance still depends on current-head CI, exact owned-production coverage, rustdoc, security evidence, review, live governance, and integration state; ADR presence does not substitute for those gates.

## Migration and rollback

No database migration is introduced. Existing callers on this branch construct the typed lifecycle directly. A future durable task repository should persist state and transition evidence in an authenticated form that can be validated by this kernel rather than introducing a second transition authority.

Rollback before acceptance is removal of the active BAP lifecycle branch and its Proposed ADR. After acceptance, rollback or replacement must preserve fail-closed terminal/recovery semantics or explicitly supersede this ADR with a reviewed migration for any persisted lifecycle representation.

## Open follow-ups

- Bind durable idempotency receipts to exact accepted transitions without making retry metadata task authority.
- Define authenticated persistence, atomicity, and concurrency semantics for lifecycle plus command evidence.
- Define crash-recovery classification and reconciliation for ambiguous external side effects.
- Map authenticated BAP/MCP transport messages into typed lifecycle requests without ambient protocol authority.
- Propagate cancellation and expiry into real browser/process supervision only after the corresponding runtime authority exists.

## Supersession / reversal conditions

Supersede this ADR if OriginWeave replaces the BAP lifecycle model, introduces a materially different durable event-sourced task authority, or moves canonical task-state ownership to another reviewed component. A successor must preserve explicit state authority, terminal fail-closure, monotonic recovery evidence, and the rule that lifecycle state cannot mint unrelated browser/network/secret/model/approval/tenant authority.

## References

ContextualWisdomLab. (2026). *OriginWeave architecture* [Repository specification]. *OriginWeave*. [`../../ARCHITECTURE.md`](../../ARCHITECTURE.md)

ContextualWisdomLab. (2026). *OriginWeave architecture decision records* [Repository specification]. *OriginWeave*. [`README.md`](README.md)

ContextualWisdomLab. (2026). *Agent development contract* [Repository specification]. *OriginWeave*. [`../../AGENTS.md`](../../AGENTS.md)
