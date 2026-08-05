# ADR 0002: Make the agent safety kernel deterministic and fail closed

- Status: Accepted
- Date: 2026-08-05

## Context

A web agent operates in an adversarial environment. Page content can contain prompt injections, browser profiles contain credentials, and state-changing actions can create financial, privacy, legal, and operational consequences. A language model cannot be the final authority for its own capabilities.

## Decision

OriginWeave evaluates typed actions with deterministic Rust policy before execution. The decision includes session mode, execution purpose, instruction source, exact capability, normalized source and target origins, robots evidence, secret-delivery mode, fixed risk class, and exact approval scope.

Web content is always an untrusted observation. Raw secret values are rejected. Crawler mode is read-only. State-changing actions are same-origin by default. R3 and R4 actions require exact approval. R5 legal consent is denied to autonomous agents.

## Consequences

- Browser and model adapters must translate into stable core types.
- Policy decisions are reproducible and independently testable.
- Some otherwise convenient cross-origin workflows must be decomposed into separately granted steps.
- Model prompting can add defense in depth but cannot expand authority.
- Future policy extensions must preserve fail-closed behavior when evidence is missing or malformed.
