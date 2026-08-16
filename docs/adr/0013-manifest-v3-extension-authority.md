# ADR 0013: Manifest V3 compatibility and extension-to-Agent authority

- **Status:** Proposed
- **Date:** 2026-08-10
- **Supersedes:** None
- **Superseded by:** None

## Context

OriginWeave retains Chromium as its compatibility kernel rather than reimplementing Chrome's extension runtime. That creates two independent product questions: whether a declared Manifest V3 capability works on the pinned Chromium baseline, and whether an extension can influence an OriginWeave Agent Task only through explicit OriginWeave authority.

Issue #27 requires both executable Manifest V3 compatibility evidence and explicit separation between Chromium extension permissions and OriginWeave Agent capabilities. Protected main contains partial pinned-Chromium compatibility evidence and extension-to-Agent authority foundations, but the full capability matrix, managed/native-messaging boundaries, release integration, and complete isolation acceptance remain open.

This ADR makes that target architecture reviewable without claiming issue #27 is complete. Until protected-main governance accepts it, this ADR is Proposed design authority only.

## Decision drivers

- Preserve Chromium extension compatibility without creating a second OriginWeave plugin ecosystem.
- Prevent Chrome extension permissions from becoming ambient Agent Task authority.
- Keep Human Mode and delegated Agent Task profile semantics distinct.
- Bind compatibility claims to an exact Chromium revision and declared capability matrix.
- Keep extension-produced content and messages in the untrusted-observation domain.
- Keep protected secrets and sensitive values behind independent purpose-bound authority.
- Support managed extensions without granting arbitrary native-process or cross-origin capability.
- Allow safe rollback when a Chromium revision regresses a declared extension surface.

## Assumptions and authority boundaries

- Chromium owns Manifest V3 parsing, service workers, extension APIs, isolated worlds, and browser-managed extension policy.
- OriginWeave owns Agent Task isolation, extension-to-Agent grants, task/origin/action authority, secret/sensitive disclosure, approvals, evidence, and release claims.
- A Chromium extension permission authorizes the extension inside Chromium; it does not mint an OriginWeave capability.
- An OriginWeave `extension_grant` authorizes only the explicitly bound OriginWeave interaction; it does not emulate Chrome manifest permissions.
- Extension content, page mutations, messages, native-host output, and structured tool output remain untrusted observations unless independently authenticated through a separate trusted administrative channel.
- Compatibility evidence and Agent-authority-isolation evidence are separate evidence classes. Neither implies the other.

## Options considered

### Reimplement Chrome extensions as a Rust plugin system

Rejected. It would create a second extension ecosystem and duplicate mature Chromium behavior.

### Let extensions inherit Agent Task authority from Chrome permissions

Rejected. Chrome permissions are not OriginWeave task/origin/action/approval grants and ambient inheritance creates confused-deputy, secret-disclosure, prompt-injection, and cross-origin escalation risk.

### Disable extensions in every mode

Rejected as a product-wide rule. Agent Task Mode defaults to no extensions or a managed allow-list, but Human Mode must retain normal compatible extension use and enterprises may require managed extensions.

### Retain Chromium's extension plane and add explicit OriginWeave grants

Selected.

## Decision

1. **Retain Chromium Manifest V3 as the compatibility plane.** OriginWeave does not create a competing Rust extension API for browser compatibility.
2. **Separate execution modes.** Human Mode may use the person's compatible extension set under browser/enterprise policy. Agent Task Mode defaults to no extensions or an explicit managed allow-list. Later attached-human-tab execution is labelled reduced-assurance when pre-existing extensions can influence page state.
3. **Require explicit OriginWeave extension authority.** Any extension-to-Agent interaction that can affect an Agent Task requires an `extension_grant` or equivalent typed decision bound at minimum to extension identity/version policy, session, applicable browsing context, capability, origin/resource scope, expiry, and task.
4. **Never translate Chrome permission into Agent capability.** `tabs`, `scripting`, `downloads`, `declarativeNetRequest`, host permissions, native messaging, or managed policy do not grant OriginWeave navigation, action, approval, secret, or sensitive-data authority.
5. **Keep extension output untrusted.** Extension messages and content enter the bounded observation/provenance path. They cannot alter the trusted goal, add tools, mint capabilities, approve high-risk actions, or weaken deterministic policy.
6. **Keep protected values brokered.** An extension does not receive raw credentials or sensitive values merely because it can inspect or modify a page. Independent secret/sensitive-data authority is rechecked immediately before trusted browser dispatch.
7. **Bound native messaging separately.** Native messaging is supported only behind an explicit host-managed allow-list, exact extension/host identity policy, process boundary, bounded I/O, and auditable lifecycle. It remains unsupported until that executable boundary exists.
8. **Publish exact compatibility evidence.** Public extension claims are bound to an exact Chromium revision/build and explicit Manifest V3 capability matrix. OriginWeave does not claim universal or `100% Chrome extension compatibility`.
9. **Separate Chrome-only services.** Web Store distribution, Google-account services, proprietary codecs/DRM, licensing, and other Chrome-only services are not implied by Manifest V3 compatibility.
10. **Gate releases by declared surfaces.** A declared supported capability that regresses blocks release or must be removed from the published matrix before release. Compatibility success never substitutes for Agent-authority-isolation evidence.

## Consequences

OriginWeave can preserve mature Chromium extension behavior while keeping its differentiating authority logic in reusable Rust modules. Buyers receive exact, falsifiable compatibility claims and separately reviewable security evidence. The cost is maintaining both a real-browser compatibility suite and independent authority-isolation tests, plus explicit managed-extension/native-host lifecycle work.

## Failure and degraded behavior

- A failed declared MV3 fixture makes that capability unsupported for the affected pinned release until fixed or removed from the published matrix.
- Invalid extension identity, grant scope, session/context binding, origin, expiry, or task fails closed.
- Attempts to widen task authority, inject a trusted instruction, resolve a secret, or synthesize approval are denied and recorded as bounded credential-free evidence.
- Missing native-host policy/process isolation keeps native messaging unsupported rather than falling back to ambient process execution.
- Attached human-tab sessions with unknown extensions are reduced-assurance and cannot inherit isolated-task release claims.

## Security / privacy / governance impact

The decision reduces confused-deputy and prompt-injection risk by keeping Chrome extension permissions outside OriginWeave policy. Secret and sensitive-data disclosure remain independently purpose-bound. Extension observations and compatibility diagnostics must not expose raw credentials, arbitrary local filesystem paths, unrestricted native-process output, or protected values in logs/evidence. Enterprise-managed extension policy is policy input, not a replacement for task authorization.

## Tests and acceptance evidence

Issue #27 acceptance requires pinned-Chromium evidence for the declared matrix and separate production authority tests, including service-worker/content-script lifecycle, declared APIs, restart/update persistence, Agent Task isolation without a grant, managed-grant success, denial of origin/action widening, untrusted-message handling, secret non-disclosure, exact build binding, repeated-run evidence, native-messaging denial until implemented, and release failure when a public capability regresses.

Current active compatibility PRs are evidence only for their unchanged exact heads. They do not make this Proposed ADR Accepted or close issue #27.

## Migration and rollback

No persistent database migration is introduced. A release can roll back the Chromium baseline, disable a managed extension, revoke an `extension_grant`, or remove an unproven capability from the published matrix without widening authority. Rollback evidence must retain the exact Chromium/build/capability set that was tested.

## Open follow-ups

- Complete issue #27's compatibility matrix and production isolation acceptance.
- Finish managed-extension update/version semantics after the host-managed Agent Task allow/block admission kernel. Chrome `ExtensionSettings` / `force_installed` remain Chromium install policy, not Agent grants.
- Implement the native-messaging allow-list/process boundary before claiming support.
- Integrate the complete Agent Task browser vertical slice under issue #28.
- Reconcile PRD/TRD/traceability from protected-main evidence as compatibility slices integrate.
- Promote this ADR only through explicit protected-main governance.

## Supersession / reversal conditions

Supersede this ADR if Chromium adopts a materially different extension authority model, OriginWeave intentionally drops Chromium extension compatibility, or an accepted architecture provides safer equivalent compatibility without ambient Agent authority. A successor must retain explicit compatibility evidence and task-authority separation.

## References

Primary browser/extension/protocol evidence and APA 7 references are maintained in [`../doctoring/browser-agent-protocols.md`](../doctoring/browser-agent-protocols.md) and [`../doctoring.md`](../doctoring.md). Related decisions include ADR 0001, ADR 0002, ADR 0007, ADR 0010, ADR 0101, ADR 0104, and ADR 0107.