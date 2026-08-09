# ADR 0104: Prompt-injection and secret authority separation

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave intentionally processes hostile web content while also supporting authenticated enterprise tasks. A page can contain text, WebMCP output, accessibility labels, structured data, downloads, or model-visible artifacts designed to impersonate instructions and solicit credentials. If the same channel can both describe page state and grant access to secrets or actions, prompt injection becomes an authority escalation rather than merely bad content.

## Decision drivers

- Raw webpage content is untrusted data, never instruction authority.
- Models must not receive durable raw secrets merely because a task needs authentication.
- Secret disclosure must be purpose-bound, origin-bound, action-bound, and auditable.
- High-risk side effects require policy and approval independent of model persuasion.
- Renderer compromise must not become secret-broker compromise.

## Assumptions and authority boundaries

Trusted instruction sources are explicit user or managed enterprise policy inputs accepted by the control plane. Page content, WebMCP/tool output, DOM, accessibility, network bodies, downloads, and model-generated interpretations are untrusted observations. The secret broker is a separate trusted component. The model receives an opaque handle or capability reference, never the underlying secret unless a separately reviewed product contract explicitly requires disclosure.

## Options considered

1. Put credentials in model context and rely on prompt hygiene: rejected because prompt injection can exfiltrate them.
2. Allow page scripts to request a password-manager API directly: rejected because the page would become the requester and policy authority.
3. Opaque secret handles resolved by a trusted broker only after independent policy authorization: selected.

## Decision

OriginWeave separates instruction, observation, policy, approval, and secret authority. A secret-fill typed action carries an opaque handle plus canonical target/action intent. Policy verifies mode, purpose, capability, origin, risk, approval, and secret-delivery mode before the trusted broker resolves anything. The broker binds resolution to the authorized session/task, destination/origin, purpose, and action contract and sends the minimum required value directly to the browser integration path. Model-visible logs, evidence, errors, and provenance store handles or redacted fingerprints, not raw secret material.

## Consequences

Authenticated automation requires explicit broker integration and cannot be implemented as arbitrary text substitution. Troubleshooting needs credential-safe evidence. Enterprise deployments can rotate or revoke secrets independently of task prompts. Some websites with unconventional credential flows may require new typed broker operations rather than model access to raw values.

## Failure and degraded behavior

If the broker is unavailable, authorization is stale, destination identity changes, or the handle cannot be resolved within scope, secret use fails closed. The runtime does not ask the model to reconstruct the credential, log the raw value, or widen the destination. A task may continue only on paths that no longer require that secret and remain independently authorized.

## Security / privacy / governance impact

This is a primary prompt-injection containment boundary. Secret access is least-privilege, purpose-bound, selectively disclosed, and auditable. Tenant and user ownership are checked outside browser content. Evidence must prove the authorization decision and delivery event without containing the secret. Revocation and retention policies apply to handles and audit records separately from the secret store.

## Tests and acceptance evidence

Require hostile prompt-injection tests across DOM, WebMCP, accessibility, structured data, downloads, and network content; broker-origin mismatch tests; stale/revoked handle tests; renderer-compromise tests; log/evidence secret-leak scans; exact-action and task/session scope tests; and end-to-end proof that denied secret resolution causes no browser disclosure.

## Migration and rollback

Move any direct credential passing behind opaque broker handles before enabling model-backed authenticated tasks. Rollback may disable broker-backed automation and return control to a person; it must not restore model-visible raw-secret delivery.

## Open follow-ups

Finalize task/session-bound handle expiry, one-time or bounded-use semantics, enterprise vault adapters, tenant keying, and recovery behavior for interrupted secret fills.

## Supersession / reversal conditions

Supersede only if an alternative proves equivalent isolation from untrusted content and model context, purpose/origin/action binding, revocation, least disclosure, and credential-free evidence under adversarial testing.

## References

See ADR 0002, `SECURITY.md`, `docs/THREAT_MODEL.md`, `docs/API_CONTRACT.md`, and the sensitive-data work tracked in OriginWeave.
