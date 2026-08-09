# ADR 0108: Policy-bound crawler mode

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

Crawler mode lets OriginWeave collect public web content at scale, but scale changes the product's risk profile. A crawler can overload sites, ignore declared exclusion policy, retain excessive personal data, accidentally submit forms, or be pressured to evade anti-automation controls. A provenance-native enterprise runtime must make crawl purpose, rate, robots handling, origin grants, retention, and failure semantics explicit rather than treating crawling as fast navigation.

## Decision drivers

- Crawler mode is read-only and purpose-bound.
- robots policy and rate limits must be explicit inputs to execution.
- Network and resource budgets must prevent accidental denial of service.
- Evidence must record crawl purpose and policy decisions.
- CAPTCHA or blocking controls must not be evaded automatically.

## Assumptions and authority boundaries

Crawler output and webpage content are untrusted data. Crawl configuration is trusted only when supplied through the control plane by an authorized user or enterprise policy. A site's robots response, rate feedback, HTTP status, or access-control challenge affects permission/degradation but cannot grant capabilities beyond configured policy. RFC 9309 defines a robots-exclusion interoperability protocol; it is not authentication, authorization, or a legal-right oracle.

## Options considered

1. Best-effort crawler that retries until content is obtained: rejected because it encourages overload and block evasion.
2. Rely solely on browser defaults and site responses: rejected because purpose, robots, rate, and retention are not explicit enough.
3. Dedicated read-only Crawler mode with policy, robots, rate, resource, and provenance controls: selected.

## Decision

Crawler mode is a separate execution mode paired with a public-crawl purpose. It receives explicit origin scope, concurrency and request budgets, per-origin rate limits, robots decision, retention policy, user-agent/product identity policy, and evidence configuration. State-changing typed actions are denied. Redirects and newly resolved destinations are reauthorized through the same network authority model as other navigation. robots disallow or unknown states fail according to configured fail-closed policy rather than being silently ignored. CAPTCHA, challenge, or blocking pages are recorded as blocked/degraded outcomes; OriginWeave does not provide CAPTCHA solving, fingerprint spoofing, residential-proxy rotation, or other block-evasion behavior.

HTTP retry/backoff behavior remains bounded and typed. A status such as `429 Too Many Requests` can trigger an allowed delay only within the caller's rate/time budget; it cannot authorize indefinite retry, scope expansion, alternate identity, or route evasion. Redirects never inherit crawl or network authority merely because they originated from an allowed page.

## Consequences

Some public data will remain unavailable without human or contractual access. Throughput is intentionally bounded. Operators gain predictable controls and provenance for why a URL was or was not collected. Scheduler/resource policy becomes part of crawler correctness.

## Failure and degraded behavior

Rate-limit responses back off within bounded policy or stop the affected origin. Unknown robots state, repeated transport failure, authentication challenge, CAPTCHA, or block page does not trigger evasion. Partial crawl output is labeled incomplete and retains reason/evidence. A failure on one origin need not cancel unrelated origins when their budgets and policies remain safe.

## Security / privacy / governance impact

Crawler tenancy, purpose, retention, and export are auditable. Collection minimizes unnecessary PII and credentials; authentication is not borrowed from unrelated Human/Assist profiles. Hostile pages cannot turn crawl observations into new targets outside authorized scope. Resource limits reduce abuse and noisy-neighbor risk. Terms, contractual rights, copyright, and privacy obligations remain separate governance inputs; a robots allow rule is not evidence that every collection/use purpose is legally or contractually allowed.

## Tests and acceptance evidence

Require RFC 9309-compatible robots parsing/matching tests, allow/disallow/unknown/unavailable-state tests, per-origin rate and bounded backoff tests, redirect reauthorization, cross-origin scope denial, form/write denial, challenge/CAPTCHA no-evasion tests, hostile link expansion tests, tenant/resource isolation, retention/export checks, and provenance assertions for policy decisions. Network tests also cover RFC 9110 HTTP semantics relevant to redirect and rate/degraded behavior.

## Migration and rollback

Migrate existing collection flows into explicit Crawler sessions with policy defaults before enabling scale. Rollback may reduce concurrency or disable an origin; it must not fall back to an unconstrained retry/evasion crawler.

## Open follow-ups

Define organization-configurable robots policy for ambiguous/unavailable cases, default rate envelopes, crawl-budget APIs, retention classes, and signed crawler identity/contract options for partner sites.

## Supersession / reversal conditions

Supersede only if another collection model provides equivalent read-only guarantees, robots/rate/resource controls, no-evasion posture, privacy governance, and provenance under representative hostile-site tests.

## References

Fielding, R., Nottingham, M., & Reschke, J. (Eds.). (2022). *HTTP semantics* (RFC 9110). RFC Editor. https://doi.org/10.17487/RFC9110

Koster, M., Illyes, G., Zeller, H., & Sassman, L. (2022). *Robots Exclusion Protocol* (RFC 9309). RFC Editor. https://doi.org/10.17487/RFC9309

## Related documents

See ADR 0002, `docs/PRD.md`, `docs/THREAT_MODEL.md`, `docs/OPERABILITY.md`, and `docs/DATA_GOVERNANCE.md`.
