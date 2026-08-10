# ADR 0012: Architecture decision acceptance governance

- **Status:** Proposed
- **Date:** 2026-08-10
- **Supersedes:** None
- **Superseded by:** None

## Context

OriginWeave separates several authorities that are easy to conflate: protected-main source, executable checks, formal review, documentation, release evidence, and runtime policy. Architecture Decision Records need the same discipline. A Markdown file, issue, chat statement, automation prompt, model verdict, or PR body can propose a decision but cannot independently make it an Accepted governing decision.

The repository's authoritative contributor contract is `AGENTS.md`, together with live GitHub rules and any explicit operationally satisfiable CWL/OriginWeave governance rule. The current contract also documents a solo-maintainer condition: an otherwise impossible independent non-author approval rule is not manufactured when fewer than two eligible independent maintainers exist, while exact-head technical/security/coverage/rustdoc/findings/live-base/branch-protection gates remain mandatory.

A previous documentation-index revision repeated those binding governance details directly in `docs/adr/README.md`. That makes an index file appear to create governance rather than discover it. This ADR proposes a durable architecture decision for ADR acceptance and reversal semantics while keeping live `AGENTS.md` and GitHub policy authoritative until this ADR itself becomes Accepted.

## Decision drivers

- Prevent documentation indexes, chat, model output, or stale PR evidence from silently changing architecture authority.
- Never synthesize, impersonate, self-submit, or otherwise fabricate an approval that current policy actually requires.
- Avoid permanently blocking a solo-maintainer repository on an independent approval route that cannot operationally exist when GitHub does not require it.
- Keep exact-head technical evidence mandatory regardless of review topology.
- Make reviewer-provisioning gaps explicit and reversible when maintainer topology changes.
- Keep ADR status discoverable and machine-checkable without turning README prose into a hidden policy engine.

## Assumptions and authority boundaries

- Current protected-main `AGENTS.md` and live GitHub rules are authoritative for contributor actions.
- This ADR is **Proposed** until protected-main governance accepts it. It therefore documents the proposed durable rule and rationale; it does not override current `AGENTS.md` or live GitHub policy while unaccepted.
- Formal review and technical checks are separate evidence classes. Neither substitutes for the other.
- A review counts only if the governing policy at that exact time recognizes the reviewer identity and review state.
- A status or review attached to a predecessor head does not transfer to a changed exact head unless GitHub policy explicitly defines such behavior.

## Options considered

### Option A — Define ADR acceptance only in the ADR index README

Rejected. An index should discover and summarize decisions, not silently create the binding governance that decides whether its own entries are Accepted.

### Option B — Require a non-author approval unconditionally, even when no eligible reviewer exists

Rejected. This creates a permanent governance deadlock in a genuine solo-maintainer topology and encourages unsafe pressure to invent reviewer identities or weaken the rule.

### Option C — Let the author or automation synthesize the missing approval

Rejected. Self-approval, impersonation, fake identities, model verdicts, reactions, status checks, or synthetic reviews cannot provide independent review evidence.

### Option D — Bind acceptance to live protected-branch governance with an explicit solo-maintainer hold

Selected. Live GitHub rules and current authoritative repository governance determine which review evidence is actually required. A governance rule that requires an independent reviewer must have an operationally valid reviewer path; otherwise the independent-review portion is held rather than fabricated, while all technical and safety gates remain intact.

## Decision

If this ADR is Accepted, OriginWeave uses the following durable architecture-decision governance:

1. **Protected-main transition defines architecture acceptance.** An ADR is not a governing protected-main decision merely because the file exists on a feature branch, appears in an issue, is described as accepted in chat, or receives a model/check verdict. Its status metadata and the protected-main transition must agree.
2. **Live policy determines required review evidence.** When current GitHub branch/ruleset policy requires a counted approval, acceptance requires a formal `APPROVED` review from an eligible identity recognized by that policy on the unchanged head to which the rule applies.
3. **Explicit repository governance may add review requirements only when operationally satisfiable.** A stricter OriginWeave/CWL rule may require an eligible non-author reviewer, but automation must verify that a legitimate reviewer route exists before treating it as executable.
4. **No synthetic approval.** Author approval, COMMENTED reviews, reactions, model verdicts, commit statuses, predecessor-head approvals, impersonated identities, or fabricated reviewer accounts never substitute for a counted independent approval when one is required.
5. **Solo-maintainer hold is narrow.** When fewer than two eligible independent maintainers exist and live GitHub policy does not independently require a counted non-author approval, an otherwise impossible repository-level independent-review requirement is placed on hold. This does not waive exact-head CI, security, SAST, exact owned-code coverage, rustdoc, unresolved finding/thread, live-base, mergeability, branch-protection, release, or operational-evidence gates.
6. **Reviewer provisioning is a first-class governance state.** If live policy requires independent approval but no eligible reviewer route exists, the PR is blocked by a reviewer-provisioning gap. The correct remedy is legitimate reviewer/team/App provisioning or governance change by an authorized human/organization control plane, not self-approval or gate weakening.
7. **The hold reverses automatically.** Independent-review enforcement is re-enabled when two or more eligible independent maintainers exist, when live GitHub policy requires it, or when an accepted superseding governance decision establishes another legitimate counted reviewer route.
8. **Indexes discover; they do not grant status.** `docs/README.md` and `docs/adr/README.md` must reflect each ADR's explicit lifecycle status and protected-main location. Their role is discoverability and consistency checking, not status creation.
9. **Implementation truth remains separate.** Even an Accepted ADR is design authority, not proof that every described capability is implemented or released. Protected-main source, executable tests, artifacts/configuration, and claim-appropriate operational/release evidence establish implementation truth.

## Consequences

### Positive

- Review governance remains realistic without weakening technical gates.
- The repository has a durable explanation for why impossible independent approval is held rather than faked in a solo-maintainer condition.
- ADR indexes can be machine-checked as inventories instead of becoming hidden policy documents.
- Maintainer-topology changes have a clear re-enablement condition.

### Costs and trade-offs

- The repository must periodically evaluate reviewer eligibility and live GitHub rules instead of relying on a timeless prose assumption.
- A future maintainer-topology change can legitimately make previously non-required independent review mandatory.
- Some changes may remain blocked on reviewer provisioning even when every technical check is green.

## Failure and degraded behavior

- If live review requirements cannot be determined, do not infer permission to accept or merge; treat review authority as unresolved and continue non-conflicting safe work.
- If a required eligible reviewer cannot be provisioned under current authority, classify the exact PR/head as reviewer-provisioning-blocked rather than weakening the rule.
- If an ADR index and ADR file disagree, the documentation contract fails and the mismatch must be repaired before using the index as architecture discovery.
- If an Accepted ADR describes behavior absent from protected-main implementation evidence, product documentation must label the capability partial/planned rather than upgrading it to shipped truth.

## Security / privacy / governance impact

This decision is governance-hardening. It prevents automation from manufacturing social proof, preserves branch/ruleset authority, and keeps model/check output non-authoritative for approval. It introduces no new secret or personal-data flow. Reviewer identity/eligibility evidence should be limited to repository/organization metadata required to establish governance and should not be copied into long-lived product telemetry.

## Tests and acceptance evidence

The documentation contract should prove at minimum that:

- every ADR file is indexed exactly once in both canonical documentation indexes;
- each indexed lifecycle status agrees with the ADR metadata;
- Accepted and Proposed entries are not silently interchanged;
- a Superseded ADR identifies a discoverable successor where applicable;
- active-PR ADRs are not presented as protected-main implementation evidence; and
- README prose points to the authoritative governance sources and this ADR rather than independently redefining the acceptance algorithm.

Operational acceptance additionally requires a current-authority probe of GitHub rules/reviewer eligibility when an actual change depends on counted approval. A documentation test cannot prove that a reviewer is eligible at runtime.

## Migration and rollback

This decision introduces no database or runtime migration. On acceptance, remove duplicate binding acceptance logic from ADR index prose and retain a concise reference to `AGENTS.md`, live GitHub policy, and this ADR. If the decision is later superseded, update both indexes and `AGENTS.md`/governance documentation coherently so no stale approval algorithm remains discoverable as current authority.

## Open follow-ups

- Keep the machine-checkable ADR-index/status contract aligned with new lifecycle states and supersession links.
- Re-evaluate reviewer topology whenever maintainers/teams/Apps or branch rules change.
- Keep scheduler prompts subordinate to protected-main `AGENTS.md` and live GitHub policy.
- If an organization-wide reviewer authority is introduced, record its eligibility and trust boundary in a superseding or amended Accepted ADR before relying on it as an OriginWeave-specific governance rule.

## Supersession / reversal conditions

Supersede this ADR if GitHub repository governance changes to a different mandatory review model, the organization adopts a formally managed independent-review service/team with explicit eligibility semantics, or OriginWeave changes its ADR lifecycle model. Any successor must retain the prohibitions on synthetic approval and on treating technical/model evidence as formal review authority.

## References

Current contributor and maintenance authority is defined by [`../../AGENTS.md`](../../AGENTS.md), live GitHub repository policy, and the ADR lifecycle index [`README.md`](README.md). This ADR deliberately does not freeze mutable GitHub product semantics into timeless prose; runtime enforcement must always be checked against the live repository policy.