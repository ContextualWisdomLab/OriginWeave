# OriginWeave Test Strategy

- **Status:** Proposed authoritative product test strategy
- **Product status:** Pre-alpha
- **Quality gates:** [`quality-gates.md`](quality-gates.md)
- **Threat model:** [`THREAT_MODEL.md`](THREAT_MODEL.md)
- **Technical requirements:** [`TRD.md`](TRD.md)

## 1. Purpose

OriginWeave tests are evidence for product claims. A test suite is not accepted merely because it has many cases or high line coverage; each material security, compatibility, authority, reliability and buyer-visible requirement must be exercised at the **true production boundary** where that requirement can fail.

This document defines how tests progress from deterministic kernels to real browser/runtime acceptance without converting planned behavior into shipped claims.

## 2. Non-negotiable quality contract

For OriginWeave-owned production code:

- exact production function/line/region/statement/branch coverage is **100%** where the repository's current tooling exposes the metric;
- public Rust APIs have useful rustdoc and public non-Rust APIs have complete docstrings;
- skipped/ignored required tests are not passing evidence;
- a test that rewrites production source or weakens a gate to pass is invalid evidence;
- failures are reproduced and fixed at the narrowest causal layer;
- release claims use exact protected-main/release-artifact evidence, not predecessor heads or synthetic merge results;
- new defects use red-green-refactor TDD when practical: first establish a realistic failing test, then implement the smallest coherent fix.

Coverage is necessary but not sufficient. Generated, unreachable, exclusion-heavy or assertion-light coverage does not replace behavioral proof.

## 3. Test pyramid by authority layer

### 3.1 Pure value and policy contracts

Use deterministic unit/property tests for:

- canonical `Origin` and browser-special-host handling;
- action kinds, risk classes, capabilities and intent digests;
- approval scope and expiry;
- destination classification and resolution snapshots;
- route/proxy authority values;
- resource budgets and cumulative mitigation decisions;
- evidence locator/digest/value-redaction rules.

### 3.2 Stateful boundary tests

Use deterministic state-transition and concurrency tests for:

- document/session/context/node lifetime;
- retry classification and attempt accounting;
- secret-handle issue/use/revoke/expiry lifecycle;
- idempotency/cancellation/checkpoint semantics;
- tenant/task/profile isolation;
- resource admission and mitigation sequencing.

### 3.3 Real protocol integration

Use loopback or hermetic real implementations, not mocks alone, for:

- TCP connection and `peer_addr` evidence;
- rustls/WebPKI certificate identity;
- bounded HTTP framing/content/deadline behavior;
- proxy/PAC execution once implemented;
- browser adapter transport where a safe local target can prove the real path.

### 3.4 Browser vertical slices

Once the browser/session/action adapter exists, run a real supported Chromium build/profile against deterministic hostile and normal local sites. Required flows include:

```text
session creation
-> isolated context
-> navigation
-> semantic observation
-> node selection
-> action policy
-> typed action
-> observed post-condition
-> evidence export
-> task close/recovery
```

Active PR #70 exercises the controlled local Agent Task fixture on the pinned
Chrome for Testing build through real WebDriver input, same-document
post-condition observation and ephemeral-profile cleanup. That lane proves
browser-level fixture execution only; it does not replace the OriginWeave
BiDi/CDP authority adapter, semantic node contract, policy dispatch or
protected-main runtime acceptance required by issue #28.

Active PR #71 additionally verifies browser-computed role/name for the
controlled input and submit target before the real WebDriver action. CSS remains
a fixture-harness locator; this does not establish OriginWeave node authority,
semantic provenance or policy dispatch.

Active PR #72 additionally records bounded browser-process RSS,
semantic-observation bytes, action latency and task duration for the same
controlled fixture. These are test-harness resource evidence from trusted
adapter inputs; they do not establish Chromium process-set attribution,
GPU/VRAM telemetry or a product resource adapter.

### 3.5 Buyer acceptance

Versioned task packs measure repeatable product outcomes rather than one lucky agent run. The benchmark artifact records browser build, OriginWeave version, model/provider/reasoning configuration, seed where supported, policy profile, hardware profile and source fixtures.

## 4. Authority-specific tests

### 4.1 Origin / destination

Include:

- malformed/ambiguous URLs and Unicode host forms;
- IPv4 integer/hex/octal-looking variants;
- IPv4-mapped IPv6;
- private/link-local/metadata/platform/reserved/documentation/benchmark ranges;
- empty/oversized resolver answer;
- hostname/IP/localhost origin constraints;
- DNS contraction vs expansion/rebinding;
- redirect downgrade, cycle, hop limit and new destination authorization.

### 4.2 Route / TCP

Include direct-only default, unauthorized proxy/PAC origin, PAC-selected DIRECT vs proxy authority, exact address set membership, port/timeout/attempt bounds, permission/input/address errors, transient retry allow-list, exact peer mismatch and single-use plan replay.

### 4.3 TLS

Use real loopback certificates/roots for:

- correct/wrong DNS SAN;
- Common Name fallback rejection;
- correct/wrong IPv4/IPv6 SAN;
- trusted/untrusted roots;
- not-yet-valid/expired/fixed trusted time;
- TLS 1.2/TLS 1.3;
- allowed/required/absent ALPN;
- transport-origin mismatch;
- peer mutation/inspection failure;
- certificate/trust/ALPN/deadline bounds;
- task-horizon safety policy if shipped.

### 4.4 HTTP

When protected-main HTTP capability exists, tests include:

- valid Content-Length, chunked and close-delimited responses;
- HEAD and no-body status semantics;
- conflicting/malformed Content-Length;
- Transfer-Encoding + Content-Length ambiguity;
- invalid status/header syntax/obs-fold/whitespace;
- incomplete/premature EOF;
- chunk/trailer limits;
- gzip/deflate expansion ratio;
- total exchange deadline;
- digest field valid/absent/malformed/unsupported/mismatch;
- supplied/observed MIME and nosniff behavior;
- safe/unsafe Content-Disposition names;
- redirect returned as evidence without ambient follow.

### 4.5 Browser session, observation and action

When shipped, tests include:

- nonzero opaque session/context IDs;
- same local node ID in two sessions/contexts;
- same origin+epoch in two contexts;
- navigation/replace rotates document epoch;
- stale handle rejected before input dispatch;
- iframe/shadow DOM/virtualized interface behavior where supported;
- accessible vs hidden text and source-channel disagreement;
- typed click/input/select/scroll/download/upload actions;
- post-condition pass/fail/timeout;
- cross-origin action decomposition;
- browser crash and context recovery.

## 5. Hostile-input strategy

Every parser and boundary that accepts attacker-controlled bytes receives table-driven negatives, property tests and fuzzing where practical.

Hostile classes include:

- control/NUL/invalid UTF-8/hostile Unicode and bidi controls;
- enormous count/length/nesting values;
- percent-encoding ambiguity and encoded separators;
- duplicate singleton fields;
- path traversal/device names;
- protocol smuggling/framing ambiguity;
- malformed certificates/HTTP fields/JSON/protocol messages;
- prompt injection in visible, hidden, metadata, structured, tool and visual channels;
- poisoned provenance locators/digests;
- replay/stale/race sequences;
- hostile extension messages;
- cross-tenant identifier collision.

A hostile test must assert the safe failure/result, not merely that the process does not crash.

## 6. Prompt-injection and LLM tests

Model behavior is nondeterministic evidence and must be surrounded by deterministic assertions.

### Offline contracts

Test that:

- untrusted page text cannot directly become a policy instruction;
- model output schema rejects untyped/arbitrary actions;
- secret values are absent from model-bound payloads;
- capability/origin/risk/approval gates are evaluated after model proposal;
- model result cannot mark an unobserved post-condition successful.

### Live smoke

Live LLM tests use `NVIDIA_NIM_API_KEY` through the reviewed credential boundary, preferably `contextual-orchestrator`. `COPILOT_GITHUB_TOKEN` is not used. Provider/model/reasoning/prompt hashes are recorded. Provider outages/rate limits are distinguished from product correctness failures.

### Evaluation

Use multiple prompt-injection variants, repeat runs and outcome distributions rather than a single success. Measure unauthorized-action rate, injection success rate, abstention/fail-closed rate and unsupported-claim rate.

## 7. Secret / PII tests

Use synthetic values unique enough for exact byte-occurrence scanning.

A representative sensitive flow must prove the approved value is usable where the business task requires it while absent from:

- model input/output unless selectively authorized;
- application logs;
- exceptions/errors;
- traces and metric labels;
- screenshots/AX labels unless explicitly necessary and policy-approved;
- generic WARC/PROV evidence;
- support/crash bundles;
- URLs/query strings;
- clipboard/history where not explicitly required.

Test wrong tenant/task/field/purpose/origin/audience, expired/revoked handle, max-use/replay, concurrent-use race, break-glass lifecycle and provider-region mismatch.

## 8. Resource and performance tests

### 8.1 Deterministic governor

Property tests cover exact soft/hard boundaries and cumulative simultaneous pressures.

### 8.2 Platform acceptance

On declared hardware profiles measure:

- foreground input latency;
- frame/compositor time and dropped frames;
- tab/process/task peak RSS;
- semantic snapshot/diff bytes;
- CPU-worker use/context switches where observable;
- peak VRAM/model residency;
- model batch/offload/CPU fallback;
- network/TLS/HTTP timing;
- task throughput and queue depth.

A resource test must prove the active offending consumer is reduced/paused and new admission is rejected at hard limits; merely preventing future work is insufficient.

## 9. Extension compatibility tests

For each supported Chromium release profile, verify representative Manifest V3 behavior:

- install/update;
- extension service-worker restart;
- content scripts and scripting;
- storage;
- declarativeNetRequest;
- downloads;
- native messaging where supported;
- side panel/commands;
- browser restart persistence;
- task-mode isolation and separate OriginWeave agent grants.

Compatibility is a versioned matrix, not a claim that every Chrome extension works.

## 10. Protocol interoperability tests

WebDriver BiDi, CDP, WebMCP and MCP adapters require:

- supported-version declaration;
- schema/message validation at the process boundary;
- unknown/extra/oversized input tests;
- identifier translation/lifetime tests;
- cancellation/timeouts;
- adapter crash/restart behavior;
- conformance tests from the primary standard/project where usable;
- proof that protocol permissions do not bypass OriginWeave authority.

## 11. Persistence, provenance and data tests

When persistence adapters ship:

- write/read round-trip every versioned record;
- enforce two-word `snake_case` database naming;
- tenant/task authorization before access;
- object/WARC/PROV digest consistency;
- retention/deletion/legal-hold transitions;
- backup/restore and migration compatibility;
- provenance completeness from result to source/action/policy/post-condition;
- tamper/corruption detection;
- no raw secret in general evidence tables/artifacts.

## 12. Concurrency and race tests

Required race classes include:

- navigation between observation and action;
- two actions using the same one-use handle;
- concurrent secret revocation/use;
- browser context close during action;
- resource pressure while task admission changes;
- cancellation during network/model/action execution;
- retry after ambiguous external side effect;
- multiple tenant sessions sharing process infrastructure;
- repository writer changing exact PR head during review/check/merge decisions.

Use deterministic barriers/latches/fakes for race orchestration, then real integration stress where practical.

## 13. Release and protected-main acceptance

A feature PR being green is not a release proof. Before a release:

1. verify exact protected-main SHA and generated artifact identity;
2. run required CI/SAST/security/coverage/packaging gates;
3. run supported browser/protocol/extension compatibility profiles;
4. run required product vertical-slice and hostile suites;
5. run protected-main operational acceptance for scheduler/runtime fixes;
6. verify SBOM/provenance/reproducibility/rollback artifacts;
7. verify no required check is queued, pending, skipped, cancelled, neutral, absent, failed or predecessor-head only;
8. verify qualifying independent review where policy requires it.

The phrase **protected-main** appears in evidence reports so release claims can be distinguished from branch-local evidence.

## 14. Failure triage and RCA

For every unexpected failure:

```text
exact evidence
-> first failing boundary
-> minimal reproduction
-> falsifiable root-cause hypothesis
-> materially distinct remedy candidates
-> feasibility check
-> smallest safe fix
-> rerun exact failure
-> affected full gates
```

Do not retry deterministic failures blindly. If multiple distinct fixes fail, reassess the architecture/contract rather than stacking speculative patches.

## 15. Test-data governance

- Prefer generated/synthetic data for secrets and PII.
- Real third-party sites are not mutated without authorization.
- Browser fixtures are versioned and reproducible.
- Public benchmark licenses/terms are recorded.
- Production user data is not copied into repository fixtures.
- Test artifacts follow retention and evidence rules.

## 16. Documentation evidence

Documentation contracts intentionally validate only durable properties such as required files, links, status vocabularies and authority assertions. Do not create brittle tests that freeze wording without preventing a real documentation defect.

## 17. Exit criteria for a production capability

A capability may be documented as Implemented only when:

- the production code is on protected `main`;
- realistic positive/negative/security tests pass;
- owned production coverage is 100% under repository policy;
- public documentation is complete;
- governing ADR/PRD/TRD/architecture/traceability are consistent;
- degraded and error behavior is tested;
- the real integration path, not a mock-only path, has acceptance evidence;
- no known valid security/review finding remains unresolved.
