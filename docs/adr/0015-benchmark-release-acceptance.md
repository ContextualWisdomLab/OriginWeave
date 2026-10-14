# ADR 0015: Benchmark release acceptance evidence and safety gates

- Status: Proposed
- Date: 2026-08-30
- Supersedes: none
- Superseded by: none

## Context

OriginWeave needs commercial benchmark evidence that distinguishes a known product failure from missing, non-authoritative, externally disrupted, or statistically insufficient evidence. A binary pass/fail aggregate would create two unacceptable failure modes: an external benchmark outage could be misreported as a product defect, while absent or weak evidence could be promoted into release success.

Active PR #240 introduces a bounded, deterministic release-evidence contract in the dedicated `originweave-release` bounded context. This ADR records the intended authority boundary for that active work. It is **Proposed** and therefore does not make the branch protected-main truth or grant release authority.

Zero observed safety events also do not prove zero underlying risk. The branch therefore retains exact trial counts and declared one-sided confidence, computes the zero-event Clopper-Pearson upper bound, evaluates separately declared fixed-point thresholds, and combines the benchmark and quantitative-safety reports into one fail-closed commercial acceptance evidence decision without turning that evidence into release authority.

## Decision drivers

- Fail closed when mandatory benchmark evidence is missing or inconclusive.
- Preserve first-causal-boundary failure classification instead of collapsing infrastructure, benchmark, capability, and product failures.
- Keep release evaluation deterministic, bounded, credential-free, and independent of benchmark execution I/O.
- Make buyer-visible limitations explicit rather than using an opaque boolean exception to acceptance.
- Represent zero-event safety evidence statistically without claiming a true event rate of zero.
- Require the commercial release policy to declare a threshold for every named zero-event safety metric; omitting a metric is invalid policy, while missing or statistically insufficient observations remain non-passing evidence.
- Bound caller-controlled collections before allocation, cloning, sorting, or map population.
- Keep evidence evaluation separate from the higher-level authority that may eventually approve a release.

## Assumptions and authority boundaries

`originweave-release` owns the benchmark release-acceptance bounded context: pure release-evidence value contracts and deterministic acceptance evaluation. `originweave-core` remains the stable cross-context value-contract kernel and must not depend outward on release-specific policy or evidence types. Neither crate executes benchmarks, authenticates artifacts, queries CI, selects supported browser/profile claims, approves exceptions, publishes releases, or decides whether an operator is authorized to release.

`BenchmarkFailureClass` is evidence causality, not an approval signal. `ZeroEventSafetyEvidence` and `ZeroEventSafetyThreshold` quantify retained observations and policy thresholds, while `decide_commercial_release_with_zero_event_safety` combines benchmark and zero-event reports only as commercial acceptance **evidence**. `DeclaredLimitation` is buyer-visible scope narrowing, not permission to waive a failed mandatory threshold.

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

### Typed deterministic evidence aggregation with mandatory safety gating

Chosen. It preserves causality, makes incompleteness explicit, bounds resource consumption, prevents a quantitative safety-threshold miss from coexisting with an accepted commercial evidence decision, prevents callers from weakening the commercial safety policy by dropping a named metric, and leaves final release authority outside the value-contract layer.

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

`ZeroEventSafetyThreshold` uses fixed-point policy inputs and returns `Satisfied`, `InsufficientConfidence`, or `UpperBoundExceedsThreshold`. `evaluate_zero_event_safety_gate` remains a generic bounded evaluator: it requires at least one declared requirement, bounds both requirement and observation inputs before map population, reports missing/insufficient/excessive evidence explicitly, and returns `Inconclusive` unless every supplied requirement is satisfied. Extra unique observations without a matching requirement remain evidence-only and cannot create vacuous success at that generic boundary.

`decide_commercial_release_with_zero_event_safety` is the stricter combined commercial evidence boundary. Before a commercial acceptance result can be retained, its requirement policy must include all five named safety metrics: unauthorized action, prompt-injection success, stale-authority acceptance, protected-value disclosure, and authority escalation. A non-empty subset is not a narrower successful policy; it fails closed with typed `MissingRequirement` causality. With a complete policy, the function retains the benchmark report and zero-event safety-gate report, preserves a known benchmark `Rejected` decision, preserves benchmark `Inconclusive`, preserves `Accepted` or `AcceptedWithDeclaredLimitations` only when every mandatory zero-event requirement is `Satisfied`, and otherwise returns combined `Inconclusive`. Invalid benchmark or safety-policy inputs preserve their typed causal source errors instead of being converted to success.

The combined commercial evidence report **does not grant release authority**. A higher authoritative boundary must still require authenticated current-head evidence, provenance, repository governance, independent review, release policy, supported-profile claims, operational acceptance, and operator authorization before merge, tag, publication, or release.

## Consequences

Positive consequences:

- buyers and operators can distinguish product failures from evidence insufficiency;
- unsupported capability cannot become silent benchmark success;
- zero-event claims retain sample size and confidence instead of implying zero risk;
- a quantitative safety-threshold miss cannot remain commercially accepted merely because the mandatory benchmark suites passed;
- dropping a named safety metric cannot silently weaken the commercial release policy;
- deterministic canonical ordering supports reproducible evidence reports;
- fixed collection limits prevent attacker-controlled report inputs from creating unbounded allocation or sorting work; and
- future release tooling can consume typed evidence without scraping PR prose.

Costs and limitations:

- this slice does not execute the commercial benchmark portfolio;
- it does not authenticate benchmark artifacts or bind them to an integrated protected-main release;
- it does not establish the supported browser/profile matrix;
- it does not define who may approve a commercial release; and
- the combined evidence report still requires a separate governance/provenance/operator release-authority boundary.

## Failure and degraded behavior

Missing mandatory suite evidence is `Inconclusive`, never success. Site drift, provider outage, infrastructure failure, unsupported capability, and benchmark defects are retained as typed inconclusive evidence rather than converted to product failure or success. Deterministic or stochastic product threshold failures remain `Rejected`.

Invalid, duplicate, or oversized metadata fails with typed errors before resource-expensive normalization or aggregation. An empty zero-event requirement set fails closed, and a non-empty commercial policy that omits any named mandatory safety metric fails with `MissingRequirement`. With a complete policy, missing observations, insufficient confidence, or an excessive upper bound leave the safety gate `Inconclusive` and therefore prevent an otherwise accepted benchmark report from producing combined commercial acceptance.

No fallback may convert authentication, integrity, provenance, approval, policy, or missing authoritative evidence into success.

## Security / privacy / governance impact

The design reduces optimistic-success and denial-of-service risk at the release-evidence boundary. Inputs are bounded before cloning, sorting, or map population; evidence values contain no secrets; deterministic ordering supports auditability; failure causality prevents an external incident from being mislabeled as a known product defect; and commercial callers cannot downgrade safety coverage by supplying only an easier subset of the named metrics.

Governance remains external to these value contracts. A combined accepted report is evidence, not approval. Repository rules, independent review, exact-head checks, artifact provenance, supported-profile authority, operational acceptance, and release authorization remain mandatory where applicable.

## Tests and acceptance evidence

The active branch contains regression coverage for:

- exact zero-event upper-confidence-bound behavior and fixed-point threshold boundaries;
- missing, duplicate, maximum-cardinality, and maximum-plus-one safety observations;
- maximum-cardinality and oversized safety-gate requirements/observations;
- deterministic failure-class mapping and retained typed failure evidence;
- duplicate-suite short-circuiting without draining arbitrary iterators;
- metadata-validation precedence before evidence consumption;
- declared limitation canonicalization and resource bounds;
- combined commercial evidence that blocks acceptance on a safety-threshold miss while preserving accepted, accepted-with-limitations, rejected, and inconclusive benchmark semantics;
- rejection of a non-empty commercial safety policy that omits any mandatory named metric; and
- stable public error messages and typed source chains across benchmark and safety-policy failures.

Acceptance for this ADR requires current-head Rust contracts, full tests, rustdoc, exact owned-production function/line/region/branch coverage, SAST, Security Scan, applicable compatibility checks, and policy-compliant independent review. Predecessor-head evidence does not transfer.

## Migration and rollback

This branch performs a deliberate **pre-GA breaking migration** of the release-acceptance public path from protected-main `originweave_core::release_acceptance::*` to the dedicated `originweave_release::*` crate. The repository currently has no published GitHub release, so the migration is being made before a GA compatibility promise rather than preserving the obsolete bounded-context ownership by adding a reverse dependency from `originweave-core` to `originweave-release`.

Repository consumers of `originweave_core::release_acceptance::*` must migrate their dependency and imports to `originweave-release` in the same integration lineage. No compatibility shim may make `originweave-core` depend outward on release-specific contracts or duplicate those contracts in both contexts. Existing `decide_release` semantics remain available from the release crate; callers that need commercial acceptance evidence should use `decide_commercial_release_with_zero_event_safety` and provide thresholds for all five named zero-event safety metrics together with their retained observations. Neither entrypoint grants repository or operator release authority.

Rollback before protected-main integration is restoration of the current protected-main `originweave_core::release_acceptance::*` owner and removal of the new release crate. After integration, reversal requires a policy-compliant superseding change and coordinated consumer migration; rollback must not create two authoritative copies or replace typed inconclusive states with optimistic success.

## Open follow-ups

- Execute the complete bounded commercial benchmark portfolio and bind results to authenticated artifacts.
- Define the integrated higher-level release authority that consumes the combined benchmark/safety evidence report alongside supported-profile claims, provenance, independent review, release policy, and operational acceptance.
- Bind benchmark inputs and outputs to immutable protected-main source and reproducible build identity.
- Complete buyer-visible support-profile and limitation governance.

## Supersession / reversal conditions

Supersede this ADR if release authority moves into another bounded context, the mandatory-suite model changes materially, zero-event statistical evidence is replaced by a different accepted safety-evidence model, or authenticated artifact provenance requires a different evidence identity contract. A successor must preserve fail-closed handling of missing/inconclusive evidence and must not treat zero observations as proof of zero risk.

## References

Clopper, C. J., & Pearson, E. S. (1934). The use of confidence or fiducial limits illustrated in the case of the binomial. *Biometrika, 26*(4), 404–413. https://doi.org/10.1093/biomet/26.4.404

See also [`../doctoring.md`](../doctoring.md) for the repository-wide research record and [`README.md`](README.md) for ADR lifecycle authority.
