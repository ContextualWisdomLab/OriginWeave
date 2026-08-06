# ADR 0001: Retain Chromium as the compatibility kernel

- Status: Accepted
- Date: 2026-08-05

## Context

OriginWeave needs modern web compatibility, JavaScript execution, graphics acceleration, process isolation, and Manifest V3 extension compatibility. Reimplementing Blink, V8, the compositor, and extension runtime would consume the project while producing worse compatibility and slower security updates.

## Decision

Chromium remains the compatibility kernel. New OriginWeave product behavior is implemented in Rust control-plane modules connected through narrow, validated adapters. Chromium patches are limited to integration points that cannot be supplied through WebDriver BiDi, CDP, Mojo, or an external process boundary.

## Consequences

- OriginWeave inherits Chromium's large upstream surface and must track security releases.
- The project can validate product demand with a stock Chromium sidecar before maintaining a distribution.
- Chrome Manifest V3 compatibility remains achievable.
- Rust ownership and memory safety apply to new OriginWeave modules, not to every line of Chromium.
- Any proposal to replace Blink or V8 requires a superseding ADR with compatibility, security-update, staffing, and benchmark evidence.
