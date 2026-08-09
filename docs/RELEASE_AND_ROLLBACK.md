# OriginWeave Release and Rollback Contract

- **Status:** Proposed authoritative release baseline
- **Product status:** Pre-alpha; there is no supported production release yet
- **Quality strategy:** [`TEST_STRATEGY.md`](TEST_STRATEGY.md)
- **Operability:** [`OPERABILITY.md`](OPERABILITY.md)
- **Changelog:** [`../CHANGELOG.md`](../CHANGELOG.md)

## 1. Purpose

A release is a verified immutable product artifact, not a branch, pull request, green model verdict or version string. OriginWeave may publish a supported version only when the exact integrated protected head and the artifacts derived from it satisfy the security, compatibility, quality, provenance, operability and rollback contract defined here.

## 2. Release sources

A production release is cut only from the exact current **protected main** tip or an immutable release commit derived by the repository's documented protected process. Required evidence cannot come solely from:

- a feature branch;
- a predecessor head;
- a synthetic merge commit not used for the artifact;
- a cancelled/skipped/queued/pending/neutral/missing required check;
- an author/self review;
- an automated comment/status presented as independent formal approval.

## 3. Version decision

Use semantic versioning according to the supported public compatibility surface.

Before `1.0.0`, the version still communicates artifact identity and change history but does not imply a frozen public API. Breaking pre-1.0 changes must still be documented clearly.

A version bump requires:

- explicit release scope;
- updated `CHANGELOG.md` moving validated entries from `Unreleased` into the release section;
- exact release commit/artifact identity;
- published supported-browser/protocol/deployment profile;
- migration and **rollback** decision.

## 4. Release gate inventory

The exact artifact source must satisfy all repository-required gates plus the following product gates where applicable:

### Source and review

- exact head/base state refetched;
- no valid unresolved review thread;
- qualifying independent non-author formal approval when required;
- no writer race or unexpected source mutation;
- dependency/lockfile changes intentional and reviewed.

### Correctness and quality

- repository contracts;
- formatting/static checks;
- all required tests;
- strict Clippy/rustdoc and other language quality checks;
- exact owned production 100% coverage under repository policy;
- real protocol/browser integration for shipped claims;
- hostile/security regressions;
- documentation graph consistent with shipped vs Planned/Proposed status.

### Security

- SAST/Semgrep/CodeQL or repository-required equivalent;
- dependency/OSV/vulnerability scans;
- supply-chain/Scorecard checks where configured;
- threat-model changes reviewed;
- no unresolved valid security finding;
- credential/PII leakage scans for applicable test corpus;
- pinned/verified security-sensitive actions/toolchains.

### Compatibility

For relevant releases:

- supported Chromium build profile;
- WebDriver BiDi/CDP/MCP adapter conformance/version tests;
- Manifest V3 compatibility matrix;
- supported OS/hardware profiles;
- schema/API compatibility tests;
- migration/restore compatibility.

### Operations

- health/readiness/task-state acceptance;
- cancellation/recovery/quarantine path;
- observability and safe logging;
- backup/restore where persistence ships;
- release canary/rollback rehearsal;
- incident runbook links.

## 5. Artifact production

Release jobs produce artifacts from a clean immutable source checkout with pinned toolchain/dependencies.

Required outputs, as applicable:

- binaries/packages/container/distribution bundles;
- checksums;
- software bill of materials (**SBOM**) in a standard supported format;
- signed build/source **provenance** or attestations;
- license notices;
- compatibility manifest;
- configuration/schema versions;
- changelog/release notes;
- migration scripts and rollback/restore instructions.

No artifact is accepted until its digest is independently recomputed and matched to the published manifest.

## 6. Reproducibility

The release process records:

```text
release_version
source_commit_sha
toolchain_version
lockfile_digest
build_workflow_identity
build_runner/profile
artifact_digest
sbom_digest
provenance_digest
browser/chromium profile
protocol/schema versions
```

Reproducibility targets are declared per artifact because platform-signed browser bundles may have nondeterministic/signing-specific steps. Any non-reproducible step must be identified and separately attested rather than ignored.

## 7. Signing and trust

Release signing keys/identities are separate from ordinary PR write credentials. Build provenance identifies the source/ref/workflow and artifact digest. Consumer/operator documentation explains how to verify artifacts before deployment.

Key compromise triggers release quarantine, key revocation/rotation, artifact impact analysis and replacement release according to incident policy.

## 8. Release evidence bundle

A release evidence bundle contains or references:

- exact commit and tag;
- required check conclusions and run IDs;
- formal review evidence;
- test/coverage summaries;
- compatibility matrices;
- threat/security scan results;
- SBOM and dependency/license results;
- provenance/attestations;
- artifact digests/signatures;
- migration/rollback evidence;
- protected-main operational acceptance where required;
- known limitations/open risks.

The bundle must be sufficient for an auditor/buyer to distinguish implementation evidence from roadmap prose.

## 9. Canary strategy

Production-capable releases use a bounded canary where deployment allows it.

Observe at least:

- process/session creation health;
- crash rate;
- task success and quarantine rate;
- stale-node/post-condition errors;
- destination/TLS/HTTP rejection drift;
- resource pressure/frame health;
- secret/PII leakage indicators;
- evidence/persistence integrity;
- extension/protocol compatibility regressions.

Canary promotion is reversible; a canary problem does not lead to weakening a safety gate to make metrics green.

## 10. Rollback triggers

Examples:

- security regression or provenance verification failure;
- cross-tenant/secret/PII exposure;
- unexpected privileged action;
- crash or data-integrity regression above the release profile threshold;
- browser/protocol incompatibility breaking supported critical workflows;
- evidence/provenance corruption;
- migration cannot satisfy integrity checks;
- resource governor regression threatening interactive safety.

## 11. Rollback procedure

General sequence:

```text
stop promotion / unsafe new admission
-> preserve evidence and affected release identity
-> quarantine ambiguous tasks
-> select last verified compatible artifact
-> verify artifact digest/signature/provenance
-> evaluate data/schema backward compatibility
-> execute application rollback or restore/forward-fix plan
-> revalidate health + critical task suite
-> confirm tenant/session/evidence integrity
-> communicate incident and next action
```

Rollback never means restoring an artifact with a known critical vulnerability merely because it is operationally familiar; choose a safe forward fix or compensated migration when necessary.

## 12. Database/schema rollback

When durable storage exists, every migration is classified:

- backward compatible / expand-contract;
- reversible transformation;
- irreversible destructive transformation;
- large/online migration requiring checkpointing.

Before applying:

- snapshot/backup/restore path tested;
- migration checksum/version recorded;
- tenant-impact and downtime estimated;
- rollback or forward-recovery steps defined;
- evidence/provenance data preserved.

Irreversible destructive migrations require explicit release governance and cannot rely on “downgrade binary” as a rollback plan.

## 13. Protocol/schema rollback

OriginWeave Protocol and evidence schemas use version negotiation. A rollback must not cause a newer persisted/event schema to be silently interpreted under older semantics.

Use:

- explicit schema version;
- compatibility reader/writer where supported;
- dual-read/dual-write or translation window when required;
- rejection/quarantine when semantics are not safely compatible.

## 14. Chromium rollback

Browser rollback has security consequences. Before downgrading Chromium:

- assess whether the prior build reintroduces a known security issue;
- verify OriginWeave adapter compatibility;
- verify profile/data compatibility;
- run critical MV3/session/action/observation tests;
- prefer forward fix when rollback would expose a patched high-severity vulnerability.

## 15. Model/provider rollback

Model routing/configuration is independently versioned from deterministic runtime authority. Rolling a model back requires its evaluation artifact and tenant/provider policy to remain valid. It does not require rolling back deterministic policy/network code unless the interface itself changed.

## 16. Secret/config rollback

Never restore revoked/expired secrets merely to recreate an old release. Configuration rollback validates current:

- tenant policy;
- keys/certificates;
- origins/routes/trust roots;
- provider regions;
- retention rules;
- feature flags.

Security configuration is current authority, not necessarily historical application state.

## 17. Emergency release

A security emergency may shorten normal cadence but may not fabricate checks/approval. The emergency path documents:

- incident/vulnerability;
- exact smallest fix;
- required tests/security scans actually run;
- independent review/authority used;
- residual unrun evidence and why;
- rollback/forward-fix plan;
- follow-up full validation deadline.

Repository branch/ruleset protections are not bypassed unless an established organizational emergency governance process explicitly authorizes and records it; OriginWeave automation itself does not create that authority.

## 18. Release notes truthfulness

Release notes distinguish:

- shipped Implemented capability;
- security/correctness fix;
- compatibility change;
- experimental/preview feature;
- deprecated behavior;
- Planned/Proposed work not included.

Do not claim safe Chromium navigation, SOC 2/CSAP certification, complete extension compatibility or enterprise privacy controls before the corresponding exact release evidence exists.

## 19. Post-release verification

After publication/deployment:

1. download/pull the public artifact through the consumer path;
2. verify digest/signature/provenance;
3. run smoke/critical task acceptance outside the source tree;
4. verify version/build metadata;
5. verify SBOM/provenance links;
6. confirm release notes/changelog match artifact;
7. monitor canary/production SLIs.

A successful publish API response alone is not proof that the usable released artifact is correct.

## 20. Release closure

A release is closed only when:

- artifact is publicly/privately available at the intended channel;
- artifact identity/provenance verifies;
- post-release smoke passes;
- rollback target/procedure remains available and verified;
- critical monitoring has no release-blocking regression;
- release evidence bundle is archived under policy;
- `CHANGELOG.md` and repository release metadata are correct.
