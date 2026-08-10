# ADR 0011: Manifest V3 compatibility and extension-to-Agent authority

- **Status:** Proposed
- **Date:** 2026-08-10
- **Supersedes:** None
- **Superseded by:** None

## Context

OriginWeave deliberately retains Chromium as its compatibility kernel rather than reimplementing Blink, V8, graphics, or Chrome's extension runtime. That architecture creates two distinct questions that must not be collapsed:

1. whether a declared Manifest V3 extension capability actually works on the pinned Chromium baseline; and
2. whether an extension can influence an OriginWeave Agent Task only through explicit OriginWeave authority.

Issue #27 requires executable compatibility evidence for declared Manifest V3 surfaces and explicit isolation between Chromium extension permissions and OriginWeave Agent capabilities. Protected main already contains partial pinned-Chromium compatibility evidence and extension-to-Agent authority foundations, while the full capability matrix, managed/native-messaging boundaries, release integration, and complete isolation acceptance remain open.

This ADR makes the target decision reviewable without claiming that issue #27 is complete. Until this ADR is Accepted through protected-main governance, it is Proposed design authority only.

## Decision drivers

- Preserve Chromium extension compatibility without creating a second OriginWeave plugin ecosystem.
- Prevent Chrome extension permissions from becoming ambient Agent Task authority.
- Keep Human Mode and delegated Agent Task profile semantics distinct.
- Make compatibility claims falsifiable and bound to an exact Chromium revision and capability matrix.
- Keep extension-produced content, messages, and tool output in the untrusted-observation domain.
- Keep protected secrets and sensitive values behind their own purpose-bound broker authority.
- Support enterprise-managed extensions without granting arbitrary native-process or cross-origin capability.
- Permit safe rollback when a Chromium revision regresses a declared extension surface.

## Assumptions and authority boundaries

- Chromium owns Manifest V3 parsing, service workers, extension APIs, isolated worlds, enterprise extension policy, and the compatibility behavior of `//extensions`.
- OriginWeave owns Agent Task session/profile isolation, extension-to-Agent grants, task/origin/action authority, secret/sensitive-data disclosure, approval, evidence, and release claims.
- A Chromium extension permission authorizes the extension inside Chromium; it does not mint an OriginWeave capability.
- An OriginWeave `extension_grant` authorizes only the explicitly bound OriginWeave interaction. It does not change or emulate Chrome manifest permissions.
- Extension content, page mutations, messages, native-host output, and structured tool output are untrusted observations unless independently authenticated as a separate trusted administrative channel.
- Compatibility evidence and Agent-authority-isolation evidence are independent evidence classes. Neither implies the other.

## Options considered

### Option A — Reimplement Chrome extensions as a Rust plugin system

Rejected. This would create a second extension ecosystem, multiply compatibility work, and move differentiation away from OriginWeave's authority/provenance control plane.

### Option B — Let extensions inherit Agent Task authority from their Chrome permissions

Rejected. Chrome permissions were not designed as OriginWeave task/origin/action/approval grants. Ambient inheritance would create confused-deputy, secret-disclosure, prompt-injection, and cross-origin escalation paths.

### Option C — Disable all extensions in every OriginWeave mode

Rejected as a product-wide rule. Agent Task Mode should default to no extensions or a managed allow-list, but Human Mode must remain compatible with ordinary Chromium extension use and enterprises may require managed extensions.

### Option D — Retain Chromium's extension plane and add explicit OriginWeave grants

Selected. Chromium remains the compatibility implementation while OriginWeave separately controls whether an extension may interact with Agent authority.

## Decision

1. **Retain Chromium Manifest V3 as the compatibility plane.** OriginWeave does not create a competing Rust extension API for product compatibility.
2. **Separate execution modes.** Human Mode may use the person's compatible extension set subject to browser/enterprise policy. Agent Task Mode defaults to no extensions or an explicit managed allow-list. Attached human-tab execution, when later supported, is labelled reduced-assurance because pre-existing extensions can influence page state.
3. **Require explicit OriginWeave extension authority.** Any extension-to-Agent interaction that can affect an Agent Task requires an `extension_grant` or equivalent typed policy decision bound at minimum to extension identity/version policy, browser session, browsing context where applicable, capability, allowed origin/resource scope, expiry, and current task.
4. **Never translate Chrome permission into Agent capability.** `tabs`, `scripting`, `downloads`, `declarativeNetRequest`, native messaging, host permissions, or managed policy do not grant OriginWeave navigation, action, approval, secret, or sensitive-data authority.
5. **Keep extension output untrusted.** Extension messages and content enter the same bounded observation/provenance path as page-controlled data. They cannot alter the trusted task goal, add tools, mint capabilities, approve high-risk actions, or weaken deterministic policy.
6. **Keep protected values brokered.** An extension never obtains raw credential or sensitive values merely because it can observe or modify a page. Any value use must pass the independent secret/sensitive-data authority immediately before trusted browser dispatch.
7. **Bound native messaging separately.** Native messaging is permitted only behind an explicit host-managed allow-list, exact extension/host identity policy, process boundary, bounded I/O, and auditable lifecycle. It is not part of the minimum compatibility claim until that executable boundary exists.
8. **Publish exact capability evidence.** Every public compatibility claim is bound to an exact Chromium revision/build and an explicit Manifest V3 capability matrix. OriginWeave does not claim universal or `100% Chrome extension compatibility`.
9. **Separate Chrome-only service claims.** Web Store distribution, Google-account services, proprietary codecs/DRM, licensing, and other Chrome-only services are not implied by Manifest V3 compatibility.
10. **Make compatibility a release gate only for declared capabilities.** A declared supported capability that regresses on the pinned release baseline blocks that release or must be removed from the published supported matrix before release. Compatibility success never substitutes for Agent-authority-isolation evidence.

## Consequences

### Positive

- Buyers can distinguish a real Chromium compatibility claim from an OriginWeave security claim.
- OriginWeave can preserve mature Chromium extension behavior while keeping its differentiating authority logic in reusable Rust control-plane modules.
- Enterprise extension policy can be integrated without granting extensions ambient task authority.
- Capability regressions can be isolated to exact Chromium revisions and exact declared surfaces.

### Costs and trade-offs

- Release acceptance needs both browser compatibility fixtures and OriginWeave authority-isolation tests.
- Managed extension identity, update, migration, and native-host lifecycle require explicit adapters and evidence.
- Attached human-profile automation cannot offer the same assurance as an isolated Agent Task profile when arbitrary user extensions are active.

## Failure and degraded behavior

- If a declared MV3 fixture fails on the pinned Chromium revision, the affected capability is unsupported for that release until fixed or explicitly removed from the supported matrix.
- If extension identity, grant scope, session/context binding, origin, expiry, or task cannot be verified, the OriginWeave interaction fails closed.
- If an extension attempts to widen task origin/action authority, provide a trusted instruction, resolve a secret, or synthesize approval, the request is denied and recorded as bounded credential-free evidence.
- If native-host policy or process isolation is unavailable, native messaging remains unsupported rather than falling back to ambient process execution.
- If an Agent Task must attach to a human tab with unknown extensions, the session is marked reduced-assurance and must not silently inherit isolated-task release claims.

## Security / privacy / governance impact

The decision reduces confused-deputy and prompt-injection risk by preventing Chromium extension permissions from being interpreted as OriginWeave policy. Secret and sensitive-data disclosure remain purpose-bound and separate. Extension observations and compatibility diagnostics must not expose raw credentials, arbitrary local filesystem paths, unrestricted native-process output, or protected values in logs/evidence. Enterprise-managed extension policy is an input to OriginWeave policy, not a replacement for task-specific authorization.

## Tests and acceptance evidence

Acceptance of the complete issue #27 boundary requires realistic pinned-Chromium evidence for the declared supported matrix and separate production authority tests. At minimum the evolving suite must cover:

- install/enable/update/restart and extension service-worker lifecycle;
- content scripts and isolated-world behavior;
- declared APIs such as storage, scripting, downloads, bookmarks, history, commands, side panel, DNR, tabs/windows, and managed policy where supported;
- restart/update persistence;
- Agent Task isolation when an extension is not granted;
- explicit managed grant success;
- denial of extension attempts to widen task origin/action authority;
- extension-produced prompt-injection/untrusted-message treatment;
- secret/sensitive-data non-disclosure;
- exact Chromium/OriginWeave build binding and repeated-run evidence;
- native messaging denial until its explicit host boundary is implemented; and
- release failure when a publicly declared capability regresses.

Current active compatibility PRs are implementation evidence only for their unchanged exact heads; they do not make this Proposed ADR Accepted or close issue #27.

## Migration and rollback

No persistent database migration is introduced by this ADR. A release can roll back an affected Chromium baseline, disable a managed extension, revoke an `extension_grant`, or remove an unproven capability from the published compatibility matrix without widening authority. Rollback must preserve evidence of which exact Chromium/build/capability set was tested.

## Open follow-ups

- Complete issue #27's declared compatibility matrix and production extension-isolation acceptance.
- Define and test managed-extension identity/update semantics.
- Implement the native-messaging host allow-list/process boundary before claiming support.
- Integrate the first complete Agent Task browser vertical slice under issue #28.
- Reconcile PRD/TRD/traceability from protected-main evidence as each active compatibility slice integrates.
- Promote this ADR from Proposed only through explicit protected-main governance; file presence or green compatibility checks are insufficient.

## Supersession / reversal conditions

Supersede this ADR if Chromium replaces Manifest V3 with a materially different extension authority model, if OriginWeave intentionally drops Chromium extension compatibility, or if an accepted architecture proves a safer compatibility mechanism that preserves equivalent buyer-visible extension behavior without ambient Agent authority. Any replacement must retain explicit compatibility evidence and task-authority separation.

## References

Primary browser/extension/protocol evidence and APA 7 references are maintained in [`../doctoring/browser-agent-protocols.md`](../doctoring/browser-agent-protocols.md) and [`../doctoring.md`](../doctoring.md). Related governing decisions include ADR 0001 (Chromium compatibility kernel), ADR 0002 (Agent safety kernel), ADR 0007 (purpose-bound sensitive-data authority), ADR 0010 (session/context-bound node authority), ADR 0101 (isolated execution/profile modes), ADR 0104 (prompt-injection and secret authority separation), and ADR 0107 (browser protocol adapter strategy).