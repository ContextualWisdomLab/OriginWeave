# ADR 0016: Release SBOM identity binding

- Status: Proposed
- Date: 2026-08-23

## Context

Issue #201 requires acquisition-grade release evidence, including a software bill of materials that is tied to the same immutable release identity buyers verify. ADR 0015 deliberately stops at a bounded release-manifest identity and does not claim SBOM content conformance, provenance, signing, publication, installation, update, rollback, or release authority.

SPDX 3.0.1 defines a Software `Sbom` as a collection of SPDX elements describing a single package. SPDX also defines JSON-LD serialization and separate structural and semantic validation requirements. A release system therefore needs to distinguish two different claims: that a release manifest names a particular SBOM artifact and exact release artifacts the SBOM is intended to describe, versus the stronger claim that the SBOM bytes themselves conform to SPDX and accurately describe those artifacts.

This slice establishes only the first claim so later SPDX generation and verification can compose without inventing artifact identities outside the release manifest.

## Decision drivers

- SBOM identity must not drift away from the exact release manifest it supplements.
- The SBOM artifact itself must already be an admitted release artifact with a manifest-bound SHA-256 identity.
- Every non-SBOM release artifact in the bound manifest must be represented exactly once in the described-artifact inventory; an admitted release artifact cannot silently fall outside the SBOM identity join.
- Every release artifact declared as described by the SBOM must already be admitted by that same manifest.
- The SBOM artifact must not be one of its own described artifact identities; OriginWeave requires a non-circular artifact join that later content verification can check independently.
- Empty, incomplete, duplicate, foreign, self-described, and case-drifted artifact identities must fail closed.
- The binding must be deterministic and bounded by the existing release-manifest inventory limit.
- Metadata binding must not be confused with SPDX content validation, provenance, signing, publication, installation, update, rollback, or release authority.

## Assumptions and authority boundaries

- ADR 0015 remains the owning decision for release-manifest identity admission.
- `ReleaseSbomFormat::Spdx30JsonLd` identifies SPDX 3.0.1 serialized as JSON-LD only; it is not a parser or conformance certificate.
- `ReleaseSbomBinding` can reference only exact, case-sensitive artifact names already present in one `ReleaseManifest`.
- The SBOM artifact is represented by the existing manifest-backed `ReleaseArtifact`, including its lowercase SHA-256 identity, and cannot also appear in the binding's described-artifact set.
- Every other artifact admitted by that same release manifest must be retained in the described-artifact set exactly once; this is an OriginWeave release-inventory completeness rule, not a general assertion about how SPDX package relationships must be modeled internally.
- The self-description prohibition is an OriginWeave release-integrity rule for a non-circular identity join; it is not presented as a general SPDX conformance requirement.
- The binding contains no package/component graph, license conclusions, vulnerability state, provenance attestations, signing keys, credentials, network authority, installer authority, or update authority.
- Later SPDX generation and verification must independently validate the actual serialized bytes against the reviewed SPDX schema/ontology and product completeness requirements.

## Options considered

### Store an arbitrary SBOM path or URL next to the manifest

Rejected. An ambient path or URL can drift from the bounded release inventory and does not guarantee that the SBOM itself is one of the artifacts buyers can hash against the manifest.

### Accept described artifact names that are not in the manifest

Rejected. That would allow the SBOM layer to invent a second release inventory and weaken ADR 0015's canonical artifact identity boundary.

### Accept only a selected subset of manifest artifacts

Rejected. A non-empty manifest-backed subset can look well-formed while silently omitting another release artifact. That would make the identity binding unsuitable as release-level SBOM inventory evidence even though the omitted artifact remains part of the canonical release manifest. OriginWeave therefore requires every non-SBOM manifest artifact to appear exactly once in the described-artifact inventory.

### Allow the SBOM artifact to describe itself

Rejected for OriginWeave's release identity contract. A self-referential described-artifact set makes the artifact-to-SBOM join circular at the exact-digest boundary that later verification must evaluate. This product boundary therefore requires at least one distinct release artifact and rejects the SBOM artifact itself as a described identity without claiming that SPDX generally forbids such a relationship.

### Treat successful construction as SPDX conformance

Rejected. Binding names and digests cannot establish that the SBOM bytes satisfy SPDX structural/semantic requirements or accurately describe package composition.

### Exact manifest-backed identity binding

Selected. The SBOM artifact and every non-SBOM release artifact resolve exactly within the same release manifest, the SBOM artifact cannot describe itself, incomplete, duplicate, and empty described inventories fail closed, and described identities are stored in deterministic lexical order.

## Decision

OriginWeave adds an inert release-SBOM identity boundary:

1. The supported initial format is `ReleaseSbomFormat::Spdx30JsonLd`, which reports SPDX specification version `3.0.1`.
2. The named SBOM artifact must exactly match an artifact admitted by the same `ReleaseManifest`.
3. At least one distinct described release artifact is required.
4. Every non-SBOM artifact admitted by the same release manifest must appear exactly once in the described-artifact inventory; omission fails closed with a typed validation error.
5. The SBOM artifact itself cannot be one of its described artifact identities; self-description fails closed with a typed validation error.
6. Every described artifact name must exactly match an artifact admitted by the same manifest.
7. Duplicate described artifact names fail closed.
8. Case-drifted names are not normalized and fail as unknown unless they exactly match the preserved manifest spelling.
9. Described artifact names are stored in deterministic lexical order.
10. The manifest's existing bounded artifact inventory transitively bounds the binding.
11. Validation failures remain typed, deterministic standard Rust errors without hidden fallback.
12. Constructing the binding does not claim SPDX content conformance, package/component completeness inside the serialized SBOM, artifact authentication, provenance, signatures, publication, installation, update, rollback, or release authority.

## Consequences

Later SPDX tooling receives one explicit non-circular and release-inventory-complete join point between SBOM identity and the exact release inventory. It cannot silently omit a release artifact admitted by the manifest, cannot describe an artifact that the release manifest did not admit, cannot treat the SBOM artifact as its own exact described release artifact, and buyers can retain the manifest's existing digest identity for the SBOM artifact itself.

This boundary is intentionally narrower than a commercial SBOM generator or verifier. Those later components must consume the actual SBOM bytes, establish SPDX 3.0.1 structural and semantic conformance, verify required software/package content, and preserve exact artifact/provenance linkage without broadening authority. Release-level artifact-inventory completeness here does not prove package/component completeness inside the SPDX document.

## Failure and degraded behavior

A missing SBOM artifact, empty or incomplete described-artifact set, self-described SBOM artifact, foreign described artifact, duplicate described identity, or case-drifted identity fails closed before a binding is produced. No fallback rewrites a name, drops an artifact, substitutes another manifest entry, accepts a partial or circular identity inventory, or converts an invalid binding into success.

Failure to construct this metadata binding does not authorize release through another path. Conversely, successful construction cannot be used as evidence that SPDX content is structurally valid, semantically valid, or package/component-complete.

## Security / privacy / governance impact

The boundary reduces supply-chain identity ambiguity by preventing the SBOM layer from creating a second ambient artifact namespace, silently omitting a release artifact, or creating a circular exact-digest join. It introduces no secrets, personal data, signing material, privileged network access, reviewer authority, or release authority. Scheduled development agents remain unable to merge, sign, tag, publish, install, or change repository governance.

## Tests and acceptance evidence

The owning `originweave-core` tests must prove exact manifest-backed SBOM admission, complete coverage of every non-SBOM manifest artifact, deterministic described-artifact ordering, and typed fail-closed behavior for missing SBOM identity, empty or incomplete described sets, self-description, foreign artifacts, duplicates, and case drift. Public error text and `std::error::Error` behavior are part of the deterministic contract.

Owned production function, line, region, and branch coverage remains exactly 100% on the unchanged reviewed head. Rust formatting, workspace/all-target checks, full tests, strict Clippy, and rustdoc must pass on that same head. Predecessor checks or model-only evidence do not transfer.

## Migration and rollback

This is a new in-process metadata primitive with no protected-main persisted SBOM schema migration. Before acceptance it can be revised or withdrawn with its owning feature branch. If a durable external release-manifest/SBOM format later depends on this contract, incompatible identity changes require explicit versioning and migration rather than parser broadening.

## Open follow-ups

- Generate SPDX 3.0.1 SBOM content for supported release packages through a separately reviewed deterministic pipeline.
- Validate JSON-LD structure and SPDX ontology semantics against the reviewed 3.0.1 resources.
- Define package/component completeness and root-element rules inside the SPDX document for OriginWeave distribution artifacts.
- Bind SBOM content, SLSA provenance, signing identity, timestamps, platform packages, reproducibility evidence, and updater trust without transferring authority from this metadata type.
- Complete issue #201's integrated release acceptance and rollback/freeze-protection architecture.

## Supersession / reversal conditions

Supersede this ADR when a versioned external release/SBOM specification replaces the in-process binding, when OriginWeave adopts a different SBOM standard under a reviewed migration, or when a stronger integrated provenance model subsumes this identity join. Reversal must preserve fail-closed manifest-backed artifact identity and cannot make metadata possession equivalent to release authority.

## References

SPDX Workgroup. (2026). *SPDX specification 3.0.1*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/
