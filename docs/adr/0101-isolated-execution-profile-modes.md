# ADR 0101: Isolated execution and profile modes

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave serves materially different authority models: a person browsing directly, an assistant helping while a person remains in control, a delegated agent task, and a crawler collecting public material. Treating these modes as cosmetic labels would allow cookies, approvals, secrets, navigation history, or write capabilities to cross trust boundaries. Enterprise buyers also need predictable isolation between users, tasks, tenants, and automated collection jobs.

## Decision drivers

- Human, Assist, Agent Task, and Crawler modes have different mutability and approval semantics.
- Browser profile state must not become an implicit capability grant.
- Delegated tasks require bounded lifetime, cancellation, and provenance.
- Crawler execution must stay read-only and policy/rate/robots aware.
- Enterprise deployments require tenant and user isolation that can be audited.

## Assumptions and authority boundaries

A browser profile stores potentially sensitive state but does not itself authorize an action. Session mode, declared purpose, capabilities, origins, approval evidence, and secret handles are explicit control-plane inputs. Page content cannot switch mode or enlarge authority.

## Options considered

1. One shared profile and runtime with mode flags: rejected because state leakage and confused-deputy risk are too high.
2. Separate applications for every mode: rejected because it duplicates the compatibility/runtime stack and weakens cross-mode consistency.
3. One product with isolated profiles/contexts and explicit mode policy: selected.

## Decision

OriginWeave defines four governed execution modes: Human, Assist, Agent Task, and Crawler. Each session binds an explicit mode, purpose, tenant/user/task identity where applicable, isolated browser profile or context policy, capability set, origin grants, resource budget, and evidence stream. Agent Task and Crawler executions use isolated task contexts by default. Crawler mode is read-only. Assist mode may prepare reversible work but state-changing actions remain governed. Human Mode does not silently transfer direct browser control to an autonomous agent.

## Consequences

Product APIs must carry mode and session identity. Profile reuse requires an explicit policy rather than convenience. More isolation increases browser-process and storage overhead, so the resource governor must account for it. Evidence can reliably attribute actions to the correct mode and task.

## Failure and degraded behavior

If profile isolation cannot be established, automated modes fail closed before loading sensitive state. A task cancellation revokes task-scoped authority and stops further state-changing actions. Recovery may reopen a fresh isolated context from a checkpoint only when the checkpoint does not smuggle stale secret, approval, or document authority.

## Security / privacy / governance impact

Cross-tenant, cross-user, and cross-task cookie/storage leakage is a high-severity boundary violation. Secret handles and approval evidence are scoped independently of browser storage. Retention and export policies apply to profile data and provenance separately. Crawler identity and purpose must remain distinguishable from a human session.

## Tests and acceptance evidence

Require cross-mode isolation tests, cookie/storage partition tests, task cancellation tests, tenant separation tests, profile lifecycle/crash-recovery tests, crawler write-denial tests, and evidence assertions proving mode/purpose/session identity on actions. Hostile page content must be unable to switch execution mode.

## Migration and rollback

Introduce explicit session-mode/profile identifiers before deprecating any shared-state automation path. Rollback can disable an automated mode but must not collapse isolated contexts into one shared profile.

## Open follow-ups

Define enterprise profile-retention defaults, tenant keying, SSO/SCIM bindings, and checkpoint portability across compatible browser versions.

## Supersession / reversal conditions

Supersede only when an alternative isolation model proves equivalent cross-user/task/tenant containment, lower operational cost, and equally auditable authority semantics under hostile-state tests.

## References

See `docs/PRD.md`, `docs/TRD.md`, `docs/THREAT_MODEL.md`, and the conceptual entities in `docs/erd/README.md`.
