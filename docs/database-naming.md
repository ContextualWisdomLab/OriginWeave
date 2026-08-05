# Database Naming Contract

OriginWeave does not yet ship a database schema. Every future persistent object must follow this contract from its first migration.

## Rules

- Use lower-case `snake_case` by default.
- Every table, view, materialized view, sequence, enum, domain, trigger, policy, index, constraint, function, procedure, and column name contains at least two semantic words.
- Avoid generic one-word names such as `user`, `event`, `data`, `log`, `type`, `name`, or `status`.
- Foreign keys use the referenced semantic object plus `_id`.
- Timestamps state their semantics, such as `created_at`, `observed_at`, `available_at`, or `expires_at`.
- Boolean names state a proposition, such as `is_ephemeral` or `has_user_approval`.
- Constraints and indexes include the object and purpose.
- Renames require compatibility migrations and an ADR when part of a public contract.

## Canonical examples

```text
agent_session
browser_profile
page_snapshot
semantic_node
network_exchange
content_record
extraction_schema
extracted_value
action_event
policy_decision
provenance_record
download_artifact
task_checkpoint
resource_budget
extension_grant
```

```text
agent_session_id
source_origin_text
target_origin_text
risk_class_code
approval_scope_hash
observed_at
available_at
is_ephemeral
```

## Rejected examples

```text
user
event
log
data
status
session
profile
snapshot
```

Automated migration tests must inspect catalog objects and fail on violations before the first database-backed release.
