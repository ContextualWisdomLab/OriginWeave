# Live merge evidence stability

## Problem

`scripts/ci/collect_live_merge_evidence.sh` is the canonical reproducible evidence collector for the volatile delivery baseline. Four review findings showed that its prior contract could produce misleading merge-readiness evidence even while preserving exact head/base SHA fields:

1. every pull request was evaluated with the rules for `main`, although stacked pull requests can target feature-parent branches with different rule evaluation;
2. the final stability check re-read only head/base, so a pull request could close, reopen, or change Draft state without invalidating the captured inventory and verdict;
3. an explicitly supplied evidence directory could contain artifacts from an earlier generation, allowing closed pull requests or partial failed runs to survive alongside current output;
4. the collector was documented as directly executable while committed without executable mode.

After those four defects were repaired, a second provenance gap remained: the collector checked each PR immediately after collecting that PR, but did not close the whole generation against a fresh inventory after the long all-PR traversal. A PR processed early could therefore close, open, move head/base, or change Draft state while later PRs were still being collected; non-PR issue membership could change for the same reason. Every per-PR verdict might have been valid at its own observation time while the directory as a whole no longer represented one current inventory.

These are provenance defects, not reasons to close or flatten stacked work. A merge verdict is useful only when the rules, inventory state, PR lifecycle state, and artifacts belong to the same bounded evidence generation.

## Constraints

- Preserve exact head and base SHA binding, current review-decision semantics, unresolved-thread blocking, and fail-closed handling of required workflows.
- Do not weaken protected-branch rules, synthesize approvals, or reinterpret stacked children as `main`-based roots.
- Do not delete historical baseline evidence to hide drift.
- Keep the collector repository-native (`bash`, `gh`, `jq`) and deterministic from one fresh destination directory.
- A moving PR may be retried, but a verdict is not materialized until its head, base SHA, base ref, open state, and Draft flag agree between the inventory, initial detail read, and final detail read.
- A collection is not complete merely because every per-PR loop returned. Final open-PR and non-PR-issue membership must still match the generation's initial inventory before a completion receipt exists.

## Alternatives considered

**Keep using `main` rules for every PR.** Rejected because branch rules are target-specific and this can both invent blockers for stacked children and miss stricter rules on another base.

**Refresh only the queue summary after all per-PR work.** Rejected as insufficient: a per-PR verdict could still have been built from rules/reviews/checks belonging to an earlier lifecycle state.

**Treat per-PR final checks as a coherent whole-generation snapshot.** Rejected because those checks occur at different times. With a large queue, an early PR may change after its own recheck but before the collector finishes later PRs.

**Delete or overwrite files in a caller-provided directory.** Rejected because a typo or shared directory could destroy unrelated evidence, and partial cleanup is itself difficult to prove. An explicit destination must instead start empty.

**Document `bash scripts/ci/collect_live_merge_evidence.sh` and leave mode `100644`.** Valid in principle, but rejected because both the baseline and repository maintenance guidance already expose the script itself as the executable interface. The committed mode is therefore made `100755` instead of changing that interface.

## Selected repair

Test-first commit `45aaaedf871e6cd9b192af119fb79db5e34c7464` added fail-closed repository contracts for per-base rules, final state/Draft stability, and empty-generation isolation. Production commit `67151a6ac90e994c360f2500be783fc05fc95354` then:

- captures `.base.ref` for each PR, URL-encodes it, fetches rules for that exact base, and feeds those rules into that PR's verdict;
- binds the verdict to the original open-inventory Draft flag plus initial/final `state`, `draft`, head SHA, base SHA, and base ref;
- writes the verdict only after those values are stable, otherwise discarding the attempt and retrying up to the existing bounded limit;
- rejects a non-directory or non-empty explicit evidence destination while retaining `mktemp` for the default path.

Commit `1cf9f56ad886385b691ef8a17f0dce07015a455b` added the executable-mode contract. Commit `f18e799c2544adeb171d64fb6d7eb91f42b9ba38` changes only the Git mode of `scripts/ci/collect_live_merge_evidence.sh` to `100755`; the then-current script blob remained unchanged.

Fresh review then identified the whole-generation closure gap. Test-first `ceecca9d89fa1fcdbe0febf627b258cdb3c1a331` requires a fresh PR and non-PR-issue inventory at the end of collection, explicit initial/final projections, fail-closed invalidation of merge verdicts when membership or PR head/base/Draft identity drifts, and an `evidence-generation.json` completion receipt. Production `b98c25882a7c4e5464b989b885a0596e2ce539e1` implements that contract:

- the initial non-PR issue set is materialized as `open-issues.json` rather than only printing its count;
- after all per-PR evidence is collected, open PRs and non-PR issues are fetched again into separate `*-rechecked.json` files;
- PR generation identity compares `{number, draft, head SHA, base ref, base SHA}` sorted by PR number; issue generation identity compares the sorted set of open non-PR issue numbers;
- any mismatch deletes all `pr-*-merge-verdict.json` files, emits a fixed failure diagnostic, and exits non-zero;
- only a stable generation receives `evidence-generation.json` with `complete: true`, completion time, and the final PR/Ready/Draft/non-PR-issue counts.

The contents-API source update preserved executable mode `100755`; exact tree `3ecd93163027765da1be955cbc03671b1be9e09b` records `scripts/ci/collect_live_merge_evidence.sh` as mode `100755`, blob `f530df1aaf2eb3265037c4a416ddb20d28cb3a46`.

## Risk and effect

The collector now makes more GitHub API calls because rules are fetched per base and both PR and issue inventories are fetched again at generation close. That cost is accepted: correctness of a merge-readiness dossier is more important than minimizing API requests, and the existing bounded retry prevents infinite per-PR collection on moving heads.

An active queue can now cause the whole run to fail after substantial work if membership or PR identity changes before closure. That is deliberate. The raw evidence remains available for RCA, but merge-verdict files are removed and no completion receipt is published, so a partial run cannot be mistaken for a complete generation merely because its directory exists.

This does not make GitHub observation transactional: review decisions, branch rules, collaborator authority, and workflow state can still change after their individual reads. The selected boundary makes the inventory/lifecycle claim explicit and fail-closed without pretending GitHub supplies a repository-wide snapshot transaction. Future evidence contracts should extend the completion receipt rather than weakening this distinction.

The change does not establish that any pull request is merge-ready. It only strengthens the provenance of the evidence used to decide that question.

## Verification and remaining evidence

Source/tree verification after `b98c25882a7c4e5464b989b885a0596e2ce539e1` confirms the current collector blob is present with mode `100755`. The original four P2 review findings were resolved after exact-source verification. The whole-generation closure was added as a new test-first successor rather than retroactively claiming the earlier review covered it.

Hosted repository CI on these Draft heads is admission-skipped, so the new Python contract and Bash syntax check are not claimed as hosted GREEN. Security, Semgrep, and CodeQL run independently and must be evaluated on the final exact head. Before promotion, the exact current head still requires executable repository contracts and the repository's normal required checks. Any later queue snapshot must be recollected into a fresh directory and accepted only when its `evidence-generation.json` receipt exists.