# Security Policy

The product-wide trust zones, assets, attacker scenarios, mitigations, acceptance evidence, and residual risks are defined in [`docs/THREAT_MODEL.md`](docs/THREAT_MODEL.md). This policy defines reporting and top-level invariants; the threat model does not replace coordinated disclosure or make a certification claim.

## Supported versions

OriginWeave is pre-alpha and has no supported production release. Security fixes are applied to the default branch. Do not deploy the current repository as an unattended production browser.

## Reporting a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private vulnerability reporting for `ContextualWisdomLab/OriginWeave` when available, or contact the Contextual Wisdom Lab maintainers privately through an established organization channel.

Include the affected commit, configuration, reproduction steps, expected and observed behavior, impact, and any evidence that secrets or user data were exposed. Do not access data that is not yours, disrupt third-party services, bypass access controls, or publish exploitation details before coordination.

## Security invariants

- Web content and model output are untrusted data.
- Only users and managed enterprise policy are trusted instruction sources.
- Secrets are resolved by a trusted broker and do not enter model context.
- Crawler mode is read-only and uses explicit robots-policy evidence.
- State-changing actions are same-origin by default.
- Approvals are exact-action and exact-origin scoped.
- Legal consent is non-delegable.
- Chromium sandboxing and Site Isolation remain part of the target architecture.
- Privileged protocol boundaries validate every message and size limit.
- Audit and provenance records must not retain raw credentials.

## Coordinated disclosure

Maintainers will acknowledge a complete report, assess severity and affected versions, prepare a tested fix, and coordinate publication. Response times are best effort until a supported release and formal SLA are published.
