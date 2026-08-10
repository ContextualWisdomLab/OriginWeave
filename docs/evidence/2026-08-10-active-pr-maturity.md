# Active pull-request maturity evidence — 2026-08-10

This dated appendix records volatile implementation evidence that must not be embedded as timeless architecture truth. Protected `main` remains the only shipped-code authority. Active pull requests are implementation evidence only until they integrate and protected-main acceptance is re-established.

## Protected-main anchor

- Protected `main`: `67af7c87589edc2039545af335c95064d9b8391c`
- Product status: pre-alpha
- Documentation verdict: **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## Active implementation evidence

| PR | Scope | Maturity on 2026-08-10 | Dependency / evidence boundary |
|---|---|---|---|
| #37 | Bounded HTTP/1.1 over authenticated governed transport | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `a38980683f073c8df8bebb8d674efaca4cf3e66d` is gate-clean and mergeable; protected main still reports HTTP as Planned. Historical #11 remains predecessor lineage until protected integration. |
| #40 | Browser protocol identifier → OriginWeave authority registry | **IMPLEMENTED_ON_ACTIVE_PR** | Current exact head `c30c76ecc217497fb23b188e095ad486fd612498` is gate-clean across CI, Security Scan, SAST, Manifest V3 Compatibility and CodeRabbit; the real browser adapter remains Planned under #28. |
| #43 | Real pinned-Chromium Manifest V3 downloads compatibility | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `27ce89066ed1473dcd66eb26a2f91becf9df5424` is gate-clean; this proves one declared compatibility surface, not full extension compatibility or Agent authority. |
| #44 | Canonical documentation reconciliation | **IMPLEMENTED_ON_ACTIVE_PR** | This branch owns the documentation repair itself; its content does not become protected-main truth until integration. |
| #45 | Credential-free sensitive-handle lifecycle evidence | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `0f07fea031090c72a448fd9501b49d4dd7568419` is gate-clean; trusted broker/storage/value resolution remain Planned under #10. |
| #46 | In-process authoritative sensitive-handle use reservation | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `5f212cdfbf3c453472069973138fd9563cf7bff8` is gate-clean; no cross-process/database transactionality or protected-value resolution is claimed. |
| #47 | Bounded resolution freshness authority | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` is gate-clean; first-party socket planning must still consume this authority immediately before I/O. |
| #48 | TLS revocation-material freshness primitive | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `9bbe12860436027a3b7cd5786775f1dacfbc835d` is gate-clean; no OCSP/CRL acquisition, signature validation, cache, or unrevoked claim is implemented. |
| #49 | Ephemeral Agent Task profile-isolation regression | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #43 at exact head `96a4e949d96b5794ef473ccf813987b8e69ea566`; CI is green but dependency-gated and not independently integrable before #43. |
| #50 | First-party network consumption of resolution freshness | **PARTIAL** | Draft stacked on #47. The prior exact head `18d3b19523de61f59dd47a11c2c82d6451512272` established a real compile-boundary RED because tests required fresh authority while the exported network API still exposed the untimed planner. Production/export/test repair is active; no protected-main claim is permitted. |
| #51 | Browser-task runtime telemetry value object | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `1c85b966087191f52b4a709a2822b2a53fb0e2fa` is CI/Security/SAST/CodeRabbit clean and Ready for review; it carries validated measurements but performs no OS/Chromium sampling itself. |

## Historical lineage

PR #11 is a historical HTTP predecessor, not current implementation authority. It may close as superseded only after #37 reaches protected main and unique-work preservation plus protected-main acceptance are revalidated.

## Interpretation rules

1. `IMPLEMENTED_ON_ACTIVE_PR` never means shipped.
2. A green active PR does not authorize release or change an ADR lifecycle state.
3. A Draft or stacked PR remains dependency-gated even if its own checks pass.
4. Exact heads and workflow run identifiers are volatile evidence and belong in dated appendices such as this one, not in timeless Architecture/PRD/TRD claims.
5. After an active PR integrates, canonical PRD/TRD/Architecture/UML/ERD/traceability must be re-evaluated from the new protected-main head before reclassifying the capability.
