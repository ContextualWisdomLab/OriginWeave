# ADR 0109: Hourly automation secret ordering and operational closure

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave's hourly product-development workflow can perform deterministic maintenance and, only when appropriate, enter a model-backed development path. A prior incident showed that checking conditions inside a shell branch is insufficient if `NVIDIA_NIM_API_KEY` is already materialized in the step environment. It also showed that nominal fallback sequences are meaningless when their declared runtime exceeds the job budget, retries inherit dirty workspaces, failure classes are collapsed, raw secrets are rematerialized for validation, or a source repair is called complete without protected-main execution evidence.

## Decision drivers

- Deterministic gates should require no model credential.
- Secrets are materialized only on the exact path that consumes them.
- Model attempts must start from pristine exact HEAD and fit within the physical job budget.
- Failures need actionable classifications and fail-closed publication behavior.
- Source merge is not sufficient operational-closure evidence.
- Hourly execution must be work-conserving rather than report-as-completion.

## Assumptions and authority boundaries

The workflow runner and pinned actions/scripts are trusted only within their declared permissions and egress. Repository content and model output are untrusted change proposals until independently validated. `NVIDIA_NIM_API_KEY` is model-development authority, not a general repository credential. Publication, review, and merge identities remain separate. Protected branch policy is an independent authority.

## Options considered

1. Materialize the model secret at job start and conditionally use it later: rejected because unused deterministic paths still receive the credential.
2. Run all maintenance through the model path: rejected because deterministic repository state does not need model authority and availability.
3. Deterministic gate first, conditional secret broker/model path second, independent validation/publication last: selected.

## Decision

The hourly workflow executes credential-free deterministic gates first, including open PR, release blocker, dry-run, writer-lease, and feasibility state. An open PR outcome is represented explicitly as `open_pull_request` and stops before model-secret materialization. Only a zero-PR or explicitly eligible development state may enter the credential step that exposes `NVIDIA_NIM_API_KEY` to the loopback broker/model path. Model attempts use pristine exact HEAD, bounded per-attempt and total budgets, and classified failures such as timeout, model/tool failure, broker failure, validation failure, or publication-authority failure. Post-model validation uses credential-free fingerprints/handles rather than rematerializing the raw secret. Missing publication authority for a verified non-empty change fails closed. The single-flight hourly loop returns to other safe work after each blocked item.

## Consequences

Model availability no longer blocks deterministic maintenance. Secret exposure is narrower and easier to audit. Fallbacks consume more explicit setup/reset time but are independently attributable. Operational acceptance requires real workflow executions from the integrated protected branch rather than only source-level tests.

## Failure and degraded behavior

A deterministic open-PR/release-blocker state exits before credential access and leaves the model path untouched. Broker failure stops model fallback immediately. A model timeout or tool failure may proceed to another pristine attempt only if the broker remains healthy and the remaining job budget is sufficient. Validation or publication failure cannot be converted to success by discarding a non-empty patch. Unknown failure classes fail closed with evidence.

## Security / privacy / governance impact

Harden Runner egress stays fail closed with evidence-backed endpoint sets. The model secret is never replaced with `COPILOT_GITHUB_TOKEN`, guessed PATs, or blanket secret inheritance. Raw secret values are absent from model-visible evidence and post-model scanning. Writer, publication, review, and merge authorities remain separated so automation cannot manufacture independent approval.

## Tests and acceptance evidence

Repository contracts must verify gate-before-secret ordering, absence of the raw key from deterministic/post-model steps, exact egress, physically schedulable time budgets, pristine fallback reset, bounded model-controlled file reads, classified failures, and fail-closed publication. Incident closure additionally requires protected-main scheduled or manual evidence: with an open PR the run reaches `open_pull_request` without materializing `NVIDIA_NIM_API_KEY`; after a later zero-PR state, a run reaches the conditional model path or an explicit deterministic product/release gate; controlled model failure demonstrates classification and pristine retry where feasible.

## Migration and rollback

Integrate scheduler changes through normal protected review. Rollback may return to the previous protected workflow only if it preserves deterministic-before-secret ordering, fail-closed egress, authority separation, and protected-main evidence. Never roll back to raw-secret rematerialization or an unschedulable fallback sequence.

## Open follow-ups

Collect protected-main acceptance runs after the incident repair merges; maintain runbooks for broker/provider outages; keep reviewer-provisioning governance separate from the workflow's technical correctness.

## Supersession / reversal conditions

Supersede only if a replacement scheduler demonstrates equal or stronger secret minimization, exact-head reset, physical budget feasibility, failure classification, authority separation, work-conserving behavior, and protected-main operational proof.

## References

See `docs/OPERABILITY.md`, `docs/TEST_STRATEGY.md`, `docs/RELEASE_AND_ROLLBACK.md`, `AGENTS.md`, and the hourly product-development workflow and incident repair history.
