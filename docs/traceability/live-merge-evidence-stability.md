# Live merge evidence stability

## Problem

`scripts/ci/collect_live_merge_evidence.sh` is the canonical reproducible evidence collector for the volatile delivery baseline. Four review findings showed that its prior contract could produce misleading merge-readiness evidence even while preserving exact head/base SHA fields:

1. every pull request was evaluated with the rules for `main`, although stacked pull requests can target feature-parent branches with different rule evaluation;
2. the final stability check re-read only head/base, so a pull request could close, reopen, or change Draft state without invalidating the captured inventory and verdict;
3. an explicitly supplied evidence directory could contain artifacts from an earlier generation, allowing closed pull requests or partial failed runs to survive alongside current output;
4. the collector was documented as directly executable while committed without executable mode.

These are provenance defects, not reasons to close or flatten stacked work. A merge verdict is useful only when the rules, inventory state, PR lifecycle state, and artifacts belong to the same evidence generation.

## Constraints

- Preserve exact head and base SHA binding, current review-decision semantics, unresolved-thread blocking, and fail-closed handling of required workflows.
- Do not weaken protected-branch rules, synthesize approvals, or reinterpret stacked children as `main`-based roots.
- Do not delete historical baseline evidence to hide drift.
- Keep the collector repository-native (`bash`, `gh`, `jq`) and deterministic from one fresh destination directory.
- A moving PR may be retried, but a verdict is not materialized until its head, base SHA, base ref, open state, and Draft flag agree between the inventory, initial detail read, and final detail read.

## Alternatives considered

**Keep using `main` rules for every PR.** Rejected because branch rules are target-specific and this can both invent blockers for stacked children and miss stricter rules on another base.

**Refresh only the queue summary after all per-PR work.** Rejected as insufficient: a per-PR verdict could still have been built from rules/reviews/checks belonging to an earlier lifecycle state.

**Delete or overwrite files in a caller-provided directory.** Rejected because a typo or shared directory could destroy unrelated evidence, and partial cleanup is itself difficult to prove. An explicit destination must instead start empty.

**Document `bash scripts/ci/collect_live_merge_evidence.sh` and leave mode `100644`.** Valid in principle, but rejected because both the baseline and repository maintenance guidance already expose the script itself as the executable interface. The committed mode is therefore made `100755` instead of changing that interface.

## Selected repair

Test-first commit `45aaaedf871e6cd9b192af119fb79db5e34c7464` added fail-closed repository contracts for per-base rules, final state/Draft stability, and empty-generation isolation. Production commit `67151a6ac90e994c360f2500be783fc05fc95354` then:

- captures `.base.ref` for each PR, URL-encodes it, fetches rules for that exact base, and feeds those rules into that PR's verdict;
- binds the verdict to the original open-inventory Draft flag plus initial/final `state`, `draft`, head SHA, base SHA, and base ref;
- writes the verdict only after those values are stable, otherwise discarding the attempt and retrying up to the existing bounded limit;
- rejects a non-directory or non-empty explicit evidence destination while retaining `mktemp` for the default path.

Commit `1cf9f56ad886385b691ef8a17f0dce07015a455b` added the executable-mode contract. Commit `f18e799c2544adeb171d64fb6d7eb91f42b9ba38` changes only the Git mode of `scripts/ci/collect_live_merge_evidence.sh` to `100755`; its blob remains `d204b1ce4ada2dcd3e842182e254bc99ea34e725`.

## Risk and effect

The collector now makes more GitHub API calls because rules are fetched per base. That cost is accepted: correctness of a merge-readiness dossier is more important than minimizing API requests, and the existing bounded retry prevents infinite collection on moving PRs. An unusually active queue can still fail closed after three unstable attempts; that is preferable to publishing a mixed-generation dossier.

The change does not establish that any pull request is merge-ready. It only strengthens the provenance of the evidence used to decide that question.

## Verification and remaining evidence

Source/tree verification on `f18e799c2544adeb171d64fb6d7eb91f42b9ba38` confirms the collector blob is present with mode `100755`. The current review findings are repaired in source semantics. Hosted CI for this exact head is Draft-admission skipped, while Security Scan, Semgrep, and CodeQL are independently generated; therefore skipped repository tests are not claimed as GREEN.

Before promotion, the exact current head still requires executable repository contracts and the repository's normal required checks. Any later queue snapshot must be recollected into a fresh directory rather than inheriting files from this evidence generation.