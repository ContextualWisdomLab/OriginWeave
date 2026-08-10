# Active pull-request maturity evidence series — opened 2026-08-10

- **Evidence series opened:** 2026-08-10
- **Last refreshed:** 2026-08-11
- **Filename semantics:** the date in this filename is the date this evidence series was opened; refresh provenance is recorded separately and is never backdated to match the filename.

This dated appendix records volatile implementation evidence that must not be embedded as timeless architecture truth. Protected `main` remains the only shipped-code authority. Active pull requests are implementation evidence only until they integrate and protected-main acceptance is re-established.

## Protected-main anchor

- Protected `main`: `67af7c87589edc2039545af335c95064d9b8391c`
- Product status: pre-alpha
- Documentation verdict: **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**

## Active implementation evidence

| PR | Scope | Maturity | Dependency / evidence boundary |
|---|---|---|---|
| #37 | Bounded HTTP/1.1 over authenticated governed transport | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `a38980683f073c8df8bebb8d674efaca4cf3e66d` is gate-clean and mergeable; protected main still reports HTTP as Planned. Historical #11 remains predecessor lineage until protected integration. |
| #40 | Browser protocol identifier → OriginWeave authority registry | **IMPLEMENTED_ON_ACTIVE_PR** | Current exact head `9e635e80e9813a1d2a9c408155d52221b76eeed3` is gate-clean across CI, Security Scan, SAST, Manifest V3 Compatibility and CodeRabbit; the real browser adapter remains Planned under #28. |
| #43 | Real pinned-Chromium Manifest V3 downloads compatibility | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `27ce89066ed1473dcd66eb26a2f91becf9df5424` is gate-clean; this proves one declared compatibility surface, not full extension compatibility or Agent authority. |
| #44 | Canonical documentation reconciliation | **IMPLEMENTED_ON_ACTIVE_PR** | This branch owns the documentation repair itself; its content does not become protected-main truth until integration. |
| #45 | Credential-free sensitive-handle lifecycle evidence | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `0f07fea031090c72a448fd9501b49d4dd7568419` is gate-clean; trusted broker/storage/value resolution remain Planned under #10. |
| #46 | In-process authoritative sensitive-handle use reservation | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `5f212cdfbf3c453472069973138fd9563cf7bff8` is gate-clean; no cross-process/database transactionality or protected-value resolution is claimed. |
| #47 | Bounded resolution freshness authority | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `6b5ed4dcea281b505f67db6180bb14c3bc95b392` is gate-clean. Its first-party consumer is now implemented on stacked #50, but neither capability is protected-main truth until dependency-ordered integration. |
| #48 | TLS revocation-material freshness primitive | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `9bbe12860436027a3b7cd5786775f1dacfbc835d` is gate-clean; no OCSP/CRL acquisition, signature validation, cache, or unrevoked claim is implemented. |
| #49 | Ephemeral Agent Task profile-isolation regression | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #43 at exact head `96a4e949d96b5794ef473ccf813987b8e69ea566`; CI is green but dependency-gated and not independently integrable before #43. |
| #50 | First-party network consumption of resolution freshness | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on exact #47 head `6b5ed4dcea281b505f67db6180bb14c3bc95b392`. Exact head `f8b43bc94444986ab23aa4ef3086e446a0b39295` structurally hides the untimed public network planner, migrates first-party TLS integration helpers through `FreshConnectionPlan`, and passes CI run `31408474576` including exact owned function/line/region/branch coverage; CodeRabbit exact-head status is success. Dependency order, not implementation incompleteness, keeps the PR Draft. |
| #51 | Browser-task runtime telemetry value object | **IMPLEMENTED_ON_ACTIVE_PR** | Exact head `1c85b966087191f52b4a709a2822b2a53fb0e2fa` is CI/Security/SAST/CodeRabbit clean and Ready for review; it carries validated measurements but performs no OS/Chromium sampling itself. |
| #52 | Bounded semantic-node observation and relationship value contract | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #40. Test-only exact head `b1bd4f8bd3b5597dac8ad3c40530beba7288e8ca` intentionally proved the missing bounded parent/child relationship boundary by failing compilation and also exposed one canonical rustfmt delta. Current exact head `dbe75ca557fc6f501b0e54846c81dffa58812ced` adds at most 128 ordered child relationships, optional parent linkage, exact session/context/origin/document authority matching, self/duplicate rejection and stable credential-free errors. Current-head CI is still running, so predecessor-head success is not promoted to exact-head gate evidence. The value contract still performs no browser I/O or action dispatch. |
| #53 | Authoritative in-process sensitive-handle revocation state | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #46 at exact head `86ce4bc1c11c270dc532593d673c42bd6f623d74`; CI and CodeRabbit are green. It adds typed first-revocation-wins state but no durable broker, cross-process transactionality, protected-value resolution, KMS, or persistence. |
| #54 | Recheck resolution freshness at socket use | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #50 at exact head `ec81031c537f2b662910c1ce78c7ae0e0bfc9c1e`; CI and CodeRabbit are green. `connect_at` revalidates freshness immediately before socket I/O and the compatibility path derives elapsed monotonic time; no resolver, DNS lookup, proxy/PAC or wall-clock authority is added. |
| #55 | Bind opaque sensitive-value handle use to a non-transferable audience | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on exact #53 head `86ce4bc1c11c270dc532593d673c42bd6f623d74`. Test-only head `95f0f1e418024f5dbe7aa613e5fd1e9d88a9417a` and CI run `31419991170` proved a real regression: audience binding had caused a revoked handle with later mismatched policy state to return `ScopeMismatch` instead of authoritative `Revoked`. Current exact head `8d3ccf0a3b99fd9789210dd9798b422431fab7d8` restores revocation precedence, retains audience binding, and adds a synchronized one-use concurrency regression. CI run `31421061134` passes repository contracts, rustfmt, locked workspace check, all workspace tests, strict Clippy, rustdoc and exact owned production function/line/region/branch coverage; CodeRabbit exact-head status is success. A future trusted broker must still derive the audience from authenticated workload/service identity. |
| #56 | Real pinned-Chromium bookmark mutation compatibility | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on #43. Exact predecessor head `50111a845927bd6e657063b85ce76da45c13436e` already passed the real Manifest V3 browser workflow but CI exposed one stale repository contract that still required read-only `chrome.bookmarks.getTree`. Current exact head `e1099e35ac000c7bf87ea75666cfdd928a386370` aligns that contract with the bounded create → get → remove lifecycle; CI run `31427219564`, Manifest V3 Compatibility run `31427220684`, and CodeRabbit exact-head status all succeed. This is compatibility evidence only: it grants no OriginWeave Agent capability and does not complete issue #27's full extension matrix. |
| #57 | Typed semantic-node query over bounded observation evidence | **IMPLEMENTED_ON_ACTIVE_PR** | Draft stacked on exact #52 head `94fd284fe41746eeba9edc05d9753903b1c41ebf`. Test-only head `d0cd133f5be62fff99612d5b08aa4cf08ce2f29f` and CI run `31429065905` intentionally proved the missing public query boundary by failing compilation on absent `SemanticNodeQuery`/`SemanticNodeQueryError`. Current exact head `b4fa49953cbbb21c879a3340e264a6e132e41634` implements bounded exact role, accessible-name and typed-action selection against already validated `SemanticNodeObservation` values, with no CSS/XPath/raw DOM selector language, arbitrary JavaScript, browser I/O or action authority. Manifest V3 Compatibility is exact-head success; current CI is still running after a strict-Clippy remediation, so no predecessor-head gate result is transferred. |

## Historical lineage

PR #11 is a historical HTTP predecessor, not current implementation authority. It may close as superseded only after #37 reaches protected main and unique-work preservation plus protected-main acceptance are revalidated.

## Interpretation rules

1. `IMPLEMENTED_ON_ACTIVE_PR` never means shipped.
2. A green active PR does not authorize release or change an ADR lifecycle state.
3. A Draft or stacked PR remains dependency-gated even if its own checks pass.
4. Exact heads and workflow run identifiers are volatile evidence and belong in dated appendices such as this one, not in timeless Architecture/PRD/TRD claims.
5. After an active PR integrates, canonical PRD/TRD/Architecture/UML/ERD/traceability must be re-evaluated from the new protected-main head before reclassifying the capability.
6. A formatting-only or metadata-only correction invalidates predecessor-head exactness: current-head checks must be rerun before a lane is called gate-clean.
