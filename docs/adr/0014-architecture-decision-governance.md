# ADR 0014: Architecture decision acceptance governance

- **Status:** Proposed
- **Date:** 2026-08-10
- **Supersedes:** None
- **Superseded by:** None

## Context

OriginWeave separates protected-main source, executable checks, formal review, documentation, release evidence, and runtime policy as distinct authorities. Architecture Decision Records need the same discipline: a Markdown file, issue, chat statement, automation prompt, model verdict, or PR body can propose a decision but cannot independently make it an Accepted governing decision.

Current contributor authority comes from protected-main `AGENTS.md`, live GitHub policy, and any explicit operationally satisfiable CWL/OriginWeave governance rule. The current contract also describes a solo-maintainer condition: an otherwise impossible independent non-author approval rule is not manufactured when fewer than two eligible independent maintainers exist, while technical/security/coverage/rustdoc/findings/live-base/branch-protection gates remain mandatory.

The ADR index previously repeated these binding details directly. An index should discover governance rather than create it. This ADR therefore records the proposed durable acceptance model and its reversal conditions. While Proposed, it does not override `AGENTS.md` or live GitHub policy.

## Decision drivers

- Prevent indexes, chat, model output, or stale PR evidence from silently changing architecture authority.
- Never synthesize, impersonate, self-submit, or fabricate approval that current policy requires.
- Avoid permanent solo-maintainer deadlock when an independent reviewer route does not operationally exist and GitHub does not require one.
- Keep exact-head technical evidence mandatory regardless of review topology.
- Make reviewer-provisioning gaps explicit and reversible.
- Keep ADR status machine-checkable without turning README prose into a hidden policy engine.

## Assumptions and authority boundaries

- Protected-main `AGENTS.md` and live GitHub rules are authoritative for contributor actions.
- This ADR is Proposed until protected-main governance accepts it.
- Formal review and technical checks are separate evidence classes.
- A review counts only if the governing policy recognizes that reviewer identity and review state for the relevant exact head.
- Predecessor-head approval does not transfer across a changed head unless live policy explicitly defines that behavior.

## Options considered

### Define ADR acceptance only in the index README

Rejected. The index should summarize and discover decisions, not define the binding algorithm that grants its own statuses.

### Require non-author approval unconditionally

Rejected. In a genuine solo-maintainer topology this creates an unsatisfiable governance deadlock and pressure to invent reviewer identities or weaken the rule.

### Let the author or automation synthesize approval

Rejected. Self-approval, impersonation, model verdicts, reactions, status checks, or fabricated identities cannot provide independent review evidence.

### Bind acceptance to live protected-branch governance with a narrow solo-maintainer hold

Selected.

## Decision

If Accepted, OriginWeave applies these durable ADR-governance rules:

1. **Protected-main transition defines architecture acceptance.** A branch file, issue, chat statement, prompt, PR body, check, or model verdict does not independently create a governing Accepted ADR.
2. **Live policy defines mandatory review evidence.** When current GitHub rules require counted approval, acceptance requires a formal `APPROVED` review from an eligible identity recognized by that policy on the applicable unchanged head.
3. **Repository-specific review requirements must be operationally satisfiable.** A stricter CWL/OriginWeave rule may require an eligible non-author reviewer only when a legitimate reviewer route exists.
4. **No synthetic approval.** Author approval, COMMENTED reviews, reactions, model verdicts, statuses, predecessor-head approvals, impersonated identities, and fabricated accounts never substitute for required counted approval.
5. **The solo-maintainer hold is narrow.** When fewer than two eligible independent maintainers exist and live GitHub policy does not independently require counted non-author approval, an otherwise impossible repository-level independent-review requirement is held. CI, security, SAST, exact owned-code coverage, rustdoc, unresolved findings/threads, live-base, mergeability, branch protection, release, and operational evidence remain mandatory.
6. **Reviewer provisioning is a first-class state.** If live policy requires independent approval but no eligible reviewer route exists, the PR is reviewer-provisioning-blocked. The remedy is legitimate reviewer/team/App provisioning or an authorized governance change, never self-approval or gate weakening.
7. **The hold reverses automatically.** Independent-review enforcement returns when two or more eligible independent maintainers exist, live GitHub policy requires it, or an Accepted successor defines another legitimate counted-review route.
8. **Indexes discover; they do not grant status.** `docs/README.md` and `docs/adr/README.md` must mirror each ADR's explicit lifecycle metadata and protected-main location.
9. **Design authority is not implementation evidence.** Even an Accepted ADR does not prove described behavior is implemented or released; protected-main code/tests/artifacts/configuration and claim-appropriate operational evidence establish that truth.

## Consequences

The repository can remain review-realistic without weakening technical gates, and maintainer-topology changes have explicit re-enablement semantics. The trade-off is that reviewer eligibility and live policy must be re-evaluated when governance changes; some otherwise-green work may legitimately remain blocked on reviewer provisioning.

## Failure and degraded behavior

- If live review requirements cannot be determined, do not infer permission to accept or merge; treat review authority as unresolved and continue non-conflicting work.
- If a required reviewer cannot be provisioned under current authority, classify the exact PR/head as reviewer-provisioning-blocked.
- If an ADR index and file disagree, the documentation contract fails until repaired.
- If an Accepted ADR describes behavior absent from protected-main implementation evidence, product docs must label that capability partial/planned rather than shipped.

## Security / privacy / governance impact

This is governance hardening. It prevents automation from manufacturing social proof, preserves branch/ruleset authority, and keeps model/check output non-authoritative for approval. It introduces no new secret or personal-data path.

## Tests and acceptance evidence

The documentation contract must prove that every ADR file is indexed exactly once in both canonical indexes, index status matches file metadata, Accepted and Proposed entries are not silently interchanged, superseded decisions retain discoverable successors where applicable, and active-PR ADRs are not presented as protected-main implementation evidence. README prose should point to `AGENTS.md`, live GitHub policy, and this ADR instead of independently redefining the acceptance algorithm.

Operational acceptance for an actual merge additionally requires a current-authority probe of GitHub rules and reviewer eligibility whenever counted review matters; a documentation test cannot prove runtime reviewer eligibility.

## Migration and rollback

No database or runtime migration is introduced. On acceptance, duplicate binding review logic should be removed from ADR-index prose and replaced by concise references to `AGENTS.md`, live GitHub policy, and this ADR. A superseding governance change must update both indexes and contributor-governance documentation coherently.

## Open follow-ups

- Keep the machine-checkable ADR-index/status contract aligned with lifecycle and supersession states.
- Re-evaluate reviewer topology whenever maintainers, teams, Apps, or branch rules change.
- Keep scheduler prompts subordinate to protected-main `AGENTS.md` and live GitHub policy.
- Record any future organization-wide reviewer authority and its eligibility boundary in an Accepted successor before relying on it as repository-specific governance.

## Supersession / reversal conditions

Supersede this ADR if GitHub governance changes to a materially different review model, the organization adopts a managed independent-review service/team with explicit eligibility semantics, or OriginWeave changes its ADR lifecycle. A successor must retain the prohibitions on synthetic approval and on treating technical/model evidence as formal review authority.

## References

Current contributor authority is defined by [`../../AGENTS.md`](../../AGENTS.md), live GitHub repository policy, and the ADR lifecycle index [`README.md`](README.md). This ADR deliberately does not freeze mutable GitHub product semantics into timeless prose.