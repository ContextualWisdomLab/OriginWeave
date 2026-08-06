# ADR 0002: Make the agent safety kernel deterministic and fail closed

- Status: Accepted
- Date: 2026-08-05

## Context

A web agent operates in an adversarial environment. Page content can contain prompt injections, browser profiles contain credentials, and state-changing actions can create financial, privacy, legal, and operational consequences. A language model cannot be the final authority for its own capabilities.

Browser origin parsing is also security-sensitive. Chromium follows the WHATWG host parser, which can interpret shortened, integer, hexadecimal, and legacy octal-looking IPv4 forms differently from a generic DNS-label validator. A policy origin must not name a different destination from the browser origin that will actually be reached.

An action approval is unsafe when it covers only an action class and origin. Two purchases, submissions, uploads, deletions, or permission changes at the same origin can have materially different payloads and consequences.

## Decision

OriginWeave evaluates typed actions with deterministic Rust policy before execution. The decision includes session mode, execution purpose, instruction source, exact capability, normalized source and target origins, robots evidence, secret-delivery mode, fixed risk class, and exact approval scope.

The origin boundary accepts canonical dotted-decimal IPv4, canonical IPv6 literals, validated ASCII DNS names, HTTPS remote origins, and explicitly permitted HTTP loopback origins. Browser-special numeric host spellings are rejected rather than interpreted as DNS text.

Every action request carries an immutable lowercase `sha256:` digest of the complete canonical action intent. R3 and R4 approval evidence is bound to the exact action kind, target origin, and intent digest. Approval for one payload cannot authorize another payload even when both use the same action kind and origin.

Web content is always an untrusted observation. Raw secret values are rejected. Crawler mode is read-only. State-changing actions are same-origin by default. R3 and R4 actions require exact approval. R5 legal consent is denied to autonomous agents.

## Consequences

- Browser and model adapters must translate into stable core types.
- Browser adapters must prove that their URL parser and the policy origin boundary identify the same canonical origin before navigation.
- Action adapters must define a deterministic canonical intent representation before hashing it; ad hoc or partial payload hashes are not acceptable.
- Policy decisions are reproducible and independently testable.
- Some otherwise convenient cross-origin workflows must be decomposed into separately granted steps.
- Model prompting can add defense in depth but cannot expand authority.
- Future policy extensions must preserve fail-closed behavior when evidence is missing or malformed.
