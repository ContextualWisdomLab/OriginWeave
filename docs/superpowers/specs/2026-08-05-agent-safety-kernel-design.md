# OriginWeave Agent Safety Kernel Design

- Status: Approved baseline
- Date: 2026-08-05

## Goal

Create the smallest independently reusable Rust foundation that prevents an agent from treating web content as authority, crossing origin or secret boundaries implicitly, consuming browser resources without limits, or returning data without source evidence.

## Scope

The first slice contains four crates: core contracts, deterministic policy, resource directives, and evidence/provenance values. It deliberately excludes Chromium launch, BiDi/CDP transport, MCP, persistence, UI, and model calls.

## Core contracts

An origin accepts HTTPS or loopback HTTP and rejects user information, paths, queries, fragments, malformed authorities, ambiguous IPv6, whitespace/control characters, and invalid ports. Actions have fixed risk and capability mappings. Approval is bound to exact action and target origin.

## Policy order

1. deny human-mode autonomous execution;
2. deny web-content instruction promotion;
3. require exact capability;
4. require readable target origin;
5. for mutations, deny crawler mode, cross-origin change, and unwritable target;
6. for public crawling, require an allowed robots decision;
7. enforce broker-only secret delivery and reject unexpected secret material;
8. deny R5;
9. require exact approval for R3/R4;
10. allow only after every gate passes.

## Resource decisions

Validated budgets contain soft/hard RAM and VRAM limits, fixed CPU thread count, and frame-time budget. The governor protects hard VRAM, hard RAM, frame time, soft RAM, then soft VRAM. Mitigations are reject, pause, CPU offload, cache spill, batch reduction, or continue.

## Evidence

Network evidence contains normalized origin, path, method, and header/query maps after case-insensitive credential redaction. Provenance requires a non-empty source URL, source locator, lowercase SHA-256 identifier, evidence channel, and verification result.

## Error handling

All malformed or incomplete security inputs return explicit values and fail closed. Production libraries do not panic, unwrap, print, or perform I/O at policy boundaries.

## Verification

Realistic integration tests cover accepted and rejected origins, every action mapping, approvals, instruction sources, capabilities, read/write origins, crawler/robots states, secrets, all resource directives, credential fields, paths, hashes, and provenance variants. Production function, line, region, and branch coverage is exactly 100%; all public APIs have useful rustdoc.
