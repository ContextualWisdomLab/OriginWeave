# Workflow Registry Audit

## Status and scope

`scripts/ci/audit_workflow_registry.py` is an operator-facing, credential-free, read-only audit utility. It classifies operator-collected GitHub Actions workflow-registry evidence; it does **not** call GitHub, disable workflows, change repository settings, or grant mutation authority. Any later workflow disablement remains a separate authorized control-plane action that must independently refetch the exact workflow identity immediately before mutation.

## Invocation

```bash
python3 scripts/ci/audit_workflow_registry.py INPUT.json
python3 scripts/ci/audit_workflow_registry.py INPUT.json --output evidence.json
```

Without `--output`, canonical JSON evidence is written to stdout. With `--output`, the target is create-once: an existing leaf is never overwritten.

## Collection contract

The input is bounded to four MiB and must be a directly named regular UTF-8 JSON file reached without symbolic-link path components. FIFO/device input, ambiguous parent components, file-identity movement, duplicate JSON members, floating-point schema values, non-standard JSON constants, oversized integers, and malformed input fail closed.

Schema version 1 binds the collection to equal expected and observed protected-default-branch commit SHAs and one valid second-precision UTC observation timestamp. Registry pages must be contiguous from page 1, every accepted page must have HTTP 200, `has_next` must agree with the supplied page set, and the sum of all page records must equal GitHub's unfiltered `reported_total_count` for that collection. A non-200 page remains failed evidence. The typed retry metadata only states whether bounded recollection is appropriate for reviewed transient statuses (`408`, `429`, `500`, `502`, `503`, `504`, plus `403` when a validated bounded `Retry-After` was retained); it never converts failure into success.

Protected-main workflow paths and active-PR workflow paths are exact, duplicate-free canonical `.github/workflows/*.yml` or `.yaml` identities. Every active-PR exemption is bound to one positive pull-request number and two independently supplied lowercase 40-character contributor-head observations that must match. Protected-main and active-PR ownership may not overlap. Registry records reject duplicate IDs, duplicate paths, ambiguous path case/encoding/traversal, unsupported state values, and malformed workflow identities.

## Output and durability contract

The evidence records the exact protected-head SHA, observation time, pagination receipts, immutable workflow IDs, exact paths/states, classifications, and active-PR ownership where applicable. `active_orphan_repository_workflow` may be emitted as a `disable_candidate`; that field is evidence for review, not permission to mutate GitHub.

A file output is staged as a mode-0600 regular inode in the already-authorized parent directory, flushed and file-`fsync`ed, identity/link-count checked, linked to the absent canonical leaf, and reduced to one canonical link. The final parent directory is then `fsync`ed before success is reported. If publication, staging cleanup, or parent-directory durability fails, the utility preserves the primary typed error, performs only identity-checked bounded cleanup/rollback, and does not report successful evidence publication. Unknown identity changes fail closed rather than deleting an unproven path.

## Operator acceptance

Treat the generated JSON as point-in-time evidence only. Before any authorized workflow lifecycle mutation, independently refetch protected `main`, the relevant active-PR heads, the workflow registry, ruleset/branch-protection state, and the exact immutable workflow ID. If any authority, owner, head, path, or registry state changed, discard the earlier mutation decision and recollect. Scheduled OriginWeave writers remain subject to `AGENTS.md`; this utility does not expand their authority.
