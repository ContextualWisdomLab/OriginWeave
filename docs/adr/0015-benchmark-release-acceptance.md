# ADR 0015: Benchmark release acceptance evidence and safety gates

- Status: Proposed
- Date: 2026-08-30
- Supersedes: none
- Superseded by: none

## Context

OriginWeave needs commercial benchmark evidence that distinguishes a known product failure from missing, non-authoritative, externally disrupted, or statistically insufficient evidence. A binary pass/fail aggregate would create two unacceptable failure modes: an external benchmark outage could be misreported as a product defect, while absent or weak evidence could be promoted into release success.

Active PR #240 introduces a bounded, deterministic release-evidence contract in `originweave-core`. This ADR records the intended authority boundary for that active work. It is **Proposed** and therefore does not make the branch protected-main truth or grant release authority.

Zero observed safety events also do not prove zero underlying risk. The branch therefore retains exact trial counts and declared one-sided confidence, computes the zero-event Clopper-Pearson upper bound, and evaluates separately declared fixed-point thresholds without turning the resulting evidence into release authority.

## Decision drivers

- Fail closed when mandatory benchmark evidence is missing or inconclusive.
- Preserve first-causal-boundary failure classification instead of collapsing infrastructure, benchmark, capability, and product failures.
- Keep release evaluation deterministic, bounded, credential-free, and independent of benchmark execution I/O.
- Make buyer-visible limitations explicit rather than using an opaque boolean exception to acceptance.
- Represent zero-event safety evidence statistically without claiming a true event rate of zero.
- Bound caller-controlled collections before allocation, cloning, sorting, or map population.
- Keep evidence evaluation separate from the higher-level authority that may eventually approve a release.

## Assumptions and authority boundaries

`originweave-core` owns pure value contracts and deterministic evaluation only. It does not execute benchmarks, authenticate artifacts, query CI, select supported browser/profile claims, approve exceptions, publish releases, or decide whether an operator is authorized to release.

`BenchmarkFailureClass` is evidence causality, not an approval signal. `ZeroEventSafetyEvidence` and `ZeroEventSafetyThreshold` quantify retained observations and policy thresholds, but the zero-event safety gate does not grant release authority. `DeclaredLimitation` is buyer-visible scope narrowing, not permission to waive a failed mandatory threshold.

Protected-main source, current-head CI/security/coverage evidence, repository governance, independently counted review, authenticated artifact provenance, and the eventual integrated release process remain separate authorities.

## Options considered

### Binary pass/fail benchmark aggregation

Rejected. It conflates product defects with site drift, infrastructure failure, unsupported capability, benchmark defects, and missing evidence.

### Treat unavailable or unsupported execution as success

Rejected. It creates an optimistic-success path and can silently widen commercial claims without evidence.

### Store zero observed events as a zero risk estimate

Rejected. Zero observations only constrain the event rate under a stated statistical model, sample size, and confidence level.

### Keep release evidence in mutable PR prose only

Rejected. Review text is not a typed, bounded product contract and cannot safely serve as downstream release evidence.

### Typed deterministic evidence aggregation with separate safety gating

Chosen. It preserves causality, makes incompleteness explicit, bounds resource consumption, and leaves final release authority outside the value-contract layer.

## Decision

The active release-acceptance boundary uses five mandatory `BenchmarkSuite` identities in one canonical registry. Each suite contributes either passing evidence or a typed failure. Duplicate suite evidence fails closed while the iterator is consumed, and missing suites remain explicitly missing.

`BenchmarkFailureClass` maps deterministic contract failures and stochastic model failures to a failed suite outcome. External site drift, external outage, unsupported capability, infrastructure failure, and benchmark defects map to `Inconclusive`. An `Inconclusive` outcome is never promoted to passing evidence.

`ReleaseDecision` is deterministic:

- any known failed mandatory suite yields `Rejected`;
- otherwise any explicit inconclusive or missing mandatory suite yields `Inconclusive`;
- otherwise complete passing evidence with no declared limitation yields `Accepted`;
- otherwise complete passing evidence with explicit `DeclaredLimitation` values yields `AcceptedWithDeclaredLimitations`.

`DeclaredLimitation` text is bounded, canonical NFC, presentation-safe, non-empty, and duplicate claim identities are rejected. Limitations narrow a buyer-visible claim; they do not erase failed or missing mandatory evidence.

`ZeroEventSafetyEvidence` retains a nonzero trial count and confidence in basis points and exposes the exact one-sided zero-event Clopper-Pearson upper event-rate bound. Named `ZeroEventSafetyObservation` values are bounded to the fixed safety-metric cardinality and duplicate metric identities fail closed.

`ZeroEventSafetyThreshold` uses fixed-point policy inputs and returns `Satisfied`, `InsufficientConfidence`, or `UpperBoundExceedsThreshold`. `evaluate_zero_event_safety_gate` requires at least one declared requirement, bounds both requirement and observation inputs before map population, reports missing/insufficient/excessive evidence explicitly, and returns `Inconclusive` unless every declared requirement is satisfied. Extra unique observations without a matching requirement remain evidence-only and cannot create vacuous success.

The zero-event safety gate **does not grant release authority**. Callers must combine its report with authenticated current-head benchmark evidence, repository governance, review, release policy, and other mandatory gates at a higher authoritative boundary.

## Consequences

Positive consequences:

- buyers and operators can distinguish product failures from evidence insufficiency;
- unsupported capability cannot become silent benchmark success;
- zero-event claims retain sample size and confidence instead of implying zero risk;
- deterministic canonical ordering supports reproducible evidence reports;
- fixed collection limits prevent attacker-controlled report inputs from creating unbounded allocation or sorting work; and
- future release tooling can consume typed evidence without scraping PR prose.

Costs and limitations:

- this slice does not execute the commercial benchmark portfolio;
- it does not authenticate benchmark artifacts or bind them to an integrated protected-main release;
- it does not establish the supported browser/profile matrix;
- it does not define who may approve a commercial release; and
- a higher-level release authority must explicitly combine the separate safety-gate result with the release-decision report.

## Failure and degraded behavior

Missing mandatory suite evidence is `Inconclusive`, never success. Site drift, provider outage, infrastructure failure, unsupported capability, and benchmark defects are retained as typed inconclusive evidence rather than converted to product failure or success. Deterministic or stochastic product threshold failures remain `Rejected`.

Invalid, duplicate, or oversized metadata fails with typed errors before resource-expensive normalization or aggregation. A missing zero-event requirement set fails closed. Missing observations, insufficient confidence, or an excessive upper bound leave the safety gate `Inconclusive`.

No fallback may convert authentication, integrity, provenance, approval, policy, or missing authoritative evidence into success.

## Security / privacy / governance impact

The design reduces optimistic-success and denial-of-service risk at the release-evidence boundary. Inputs are bounded before cloning, sorting, or map population; evidence values contain no secrets; deterministic ordering supports auditability; and failure causality prevents an external incident from being mislabeled as a known product defect.

Governance remains external to these value contracts. A passing report or satisfied safety gate is evidence, not approval. Repository rules, independent review, exact-head checks, artifact provenance, and release authorization remain mandatory where applicable.

## Tests and acceptance evidence

The active branch contains regression coverage for:

- exact zero-event upper-confidence-bound behavior and fixed-point threshold boundaries;
- missing, duplicate, maximum-cardinality, and maximum-plus-one safety observations;
- maximum-cardinality and oversized safety-gate requirements/observations;
- deterministic failure-class mapping and retained typed failure evidence;
- duplicate-suite short-circuiting without draining arbitrary iterators;
- metadata-validation precedence before evidence consumption;
- declared limitation canonicalization and resource bounds; and
- stable public error messages and source chains.

Acceptance for this ADR requires current-head Rust contracts, full tests, rustdoc, exact owned-production function/line/region/branch coverage, SAST, Security Scan, applicable compatibility checks, and policy-compliant independent review. Predecessor-head evidence does not transfer.

## Migration and rollback

The change is additive inside the active `originweave-core` release-evidence surface. Existing `decide_release` callers retain the compatibility entrypoint with no zero-event observations. Callers adopting typed failure evidence or safety observations should migrate explicitly and must not infer release authority from the new reports.

Rollback is removal of the active-PR API and its callers before protected-main integration, or a later policy-compliant superseding change after integration. Rollback must not replace typed inconclusive states with optimistic success.

## Open follow-ups

- Execute the complete bounded commercial benchmark portfolio and bind results to authenticated artifacts.
- Define the integrated higher-level release authority that combines suite evidence, zero-event safety gates, supported-profile claims, provenance, review, and operational acceptance.
- Bind benchmark inputs and outputs to immutable protected-main source and reproducible build identity.
- Complete buyer-visible support-profile and limitation governance.

## Supersession / reversal conditions

Supersede this ADR if release authority moves into another bounded context, the mandatory-suite model changes materially, zero-event statistical evidence is replaced by a different accepted safety-evidence model, or authenticated artifact provenance requires a different evidence identity contract. A successor must preserve fail-closed handling of missing/inconclusive evidence and must not treat zero observations as proof of zero risk.

## References

Clopper, C. J., & Pearson, E. S. (1934). The use of confidence or fiducial limits illustrated in the case of the binomial. *Biometrika, 26*(4), 404–413. https://doi.org/10.1093/biomet/26.4.404

See also [`../doctoring.md`](../doctoring.md) for the repository-wide research record and [`README.md`](README.md) for ADR lifecycle authority.
