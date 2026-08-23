# ADR 0016: Release SBOM identity binding and bounded JSON-LD envelope verification

- Status: Proposed
- Date: 2026-08-23

## Context

Issue #201 requires acquisition-grade release evidence, including a software bill of materials tied to the same immutable release identity buyers verify. ADR 0015 deliberately stops at a bounded release-manifest identity and does not claim SBOM content conformance, provenance, signing, publication, installation, update, rollback, or release authority.

SPDX 3.0.1 defines JSON-LD serialization and separate structural and semantic validation requirements. Every SPDX 3.0.1 JSON-LD document must reference the versioned global context `https://spdx.org/rdf/3.0.1/spdx-context.jsonld` at the top level. A serialization must not define more than one `SpdxDocument`. Full SPDX JSON-LD conformance additionally requires structural validation against the versioned SPDX JSON Schema and semantic validation against the SPDX OWL ontology/SHACL constraints.

OriginWeave therefore separates four claims that must not be collapsed:

1. the declared SPDX serialization identity;
2. the immutable release-manifest/SBOM artifact identity join;
3. a narrow, bounded check of the actual JSON-LD serialization envelope; and
4. full SPDX content conformance and product-specific package/component completeness.

This slice implements claims 1 through 3 only. Claim 4 remains a later independently reviewed release gate.

## Decision drivers

- SBOM identity must not drift away from the exact release manifest it supplements.
- The supported SPDX JSON-LD format must expose its exact specification version and required versioned global context identity rather than rely on an ambient or moving context.
- Actual serialized bytes must not be accepted merely because metadata says they are SPDX 3.0.1 JSON-LD.
- Envelope parsing must be bounded before deeper semantic use and reject malformed UTF-8, malformed JSON, duplicate JSON object members, non-finite JSON constants, unexpected top-level members, non-object graph entries, excessive graph cardinality, and zero or multiple `SpdxDocument` objects.
- Error diagnostics must not reflect attacker- or supplier-controlled document bytes.
- The SBOM artifact itself must already be an admitted release artifact with a manifest-bound SHA-256 identity.
- Every non-SBOM release artifact in the bound manifest must be represented exactly once in the described-artifact inventory.
- Empty, incomplete, duplicate, foreign, self-described, and case-drifted artifact identities must fail closed.
- Partial envelope validation must not be represented as SPDX JSON Schema/OWL/SHACL conformance, package completeness, provenance, signing, publication, installation, update, rollback, or release authority.

## Assumptions and authority boundaries

- ADR 0015 remains the owning decision for release-manifest identity admission.
- `ReleaseSbomFormat::Spdx30JsonLd` identifies SPDX 3.0.1 serialized as JSON-LD and exposes the exact required global context URI.
- `ReleaseSbomBinding` can reference only exact, case-sensitive artifact names already present in one `ReleaseManifest`.
- The SBOM artifact is represented by the existing manifest-backed `ReleaseArtifact`, including its lowercase SHA-256 identity, and cannot also appear in the binding's described-artifact set.
- Every other artifact admitted by that release manifest must be retained in the described-artifact set exactly once. This is an OriginWeave release-inventory completeness rule, not a general SPDX modeling assertion.
- `scripts/release/validate_spdx_jsonld.py` consumes actual bytes only for a narrow envelope check. It accepts at most 16 MiB, requires strict UTF-8 JSON, requires exactly the top-level `@context` and `@graph` members, requires the exact SPDX 3.0.1 global context string, bounds the graph to 65,536 objects, requires each graph entry to be an object with a string `type`, and requires exactly one object whose type is `SpdxDocument`.
- The envelope verifier deliberately does not fetch remote contexts or schemas, resolve external identifiers, authenticate artifacts, validate the SPDX JSON Schema, evaluate the SPDX ontology/SHACL constraints, infer package completeness, or grant any signing/release/update authority.
- Full conformance still requires validation of the same serialized bytes against reviewed SPDX 3.0.1 structural and semantic resources plus OriginWeave product-completeness rules.

## Options considered

### Trust the declared SBOM format without reading bytes

Rejected. A manifest can correctly identify an artifact while the artifact bytes contain the wrong context, malformed JSON, an ambiguous duplicate-key object, or multiple `SpdxDocument` definitions.

### Treat a general-purpose JSON parse as SPDX conformance

Rejected. JSON syntax alone does not establish the required SPDX context, document cardinality, JSON Schema validity, ontology/SHACL validity, or SBOM completeness.

### Fetch the JSON-LD context and schema during the first verifier

Rejected. Ambient network retrieval would introduce moving external authority and availability into a release gate before pinned schema/ontology identities and offline verification are designed. The first verifier uses fixed local constants and no network access.

### Store an arbitrary SBOM path or URL next to the manifest

Rejected. An ambient path or URL can drift from the bounded release inventory and does not guarantee that the SBOM itself is one of the artifacts buyers can hash against the manifest.

### Accept only a selected subset of manifest artifacts

Rejected. A non-empty manifest-backed subset can look well formed while silently omitting another release artifact. OriginWeave therefore requires every non-SBOM manifest artifact to appear exactly once in the described-artifact inventory.

### Allow the SBOM artifact to describe itself

Rejected for OriginWeave's release identity contract. A self-referential described-artifact set makes the artifact-to-SBOM join circular at the exact-digest boundary later verification must evaluate. This is an OriginWeave integrity rule, not a claim that SPDX generally forbids such a relationship.

### Layered manifest binding plus bounded envelope verification

Selected. Metadata establishes exact release/SBOM identity; the bounded verifier checks the first non-ambient properties of the actual serialized bytes; later gates remain responsible for full SPDX and product conformance.

## Decision

OriginWeave adopts the following release-SBOM boundary:

1. The initial format is `ReleaseSbomFormat::Spdx30JsonLd`, reporting SPDX specification version `3.0.1` and global context URI `https://spdx.org/rdf/3.0.1/spdx-context.jsonld`.
2. The named SBOM artifact must exactly match an artifact admitted by the same `ReleaseManifest`.
3. At least one distinct described release artifact is required.
4. Every non-SBOM artifact admitted by the same release manifest must appear exactly once in the described-artifact inventory.
5. The SBOM artifact itself cannot be one of its described artifact identities.
6. Foreign, duplicate, incomplete, and case-drifted described artifact identities fail closed.
7. Described artifact identities are stored deterministically and the manifest's existing bound transitively limits the inventory.
8. Before deeper SPDX processing, actual candidate JSON-LD bytes may pass through `validate_spdx_3_0_1_jsonld_bytes`.
9. That verifier admits only non-empty strict-UTF-8 payloads no larger than 16 MiB, rejects duplicate JSON keys and non-finite constants, requires exactly the top-level `@context` and `@graph` members, requires the exact versioned context, allows at most 65,536 graph objects, requires object entries with string `type`, and requires exactly one `SpdxDocument`.
10. Validation errors expose stable error codes and generic diagnostics without reflecting document bytes.
11. Envelope success is not a certificate of SPDX JSON Schema validity, semantic ontology/SHACL validity, package/component completeness, artifact authenticity, provenance, signatures, publication, installation, update, rollback, or release authority.
12. No verifier fallback may substitute another SPDX version, relax the context, discard duplicate keys, silently truncate a graph, fetch an ambient remote resource, or convert malformed content into success.

## Consequences

Downstream release tooling receives an exact versioned serialization identity, a non-circular and release-inventory-complete identity join, and a bounded first check over actual bytes. A candidate cannot pass this layer with the wrong SPDX context, an extra top-level field, malformed JSON, duplicate members, a non-object graph entry, an unbounded graph, or zero/multiple `SpdxDocument` objects.

The boundary remains intentionally narrower than a commercial SBOM generator or complete verifier. The official SPDX 3.0.1 specification requires structural validation against the JSON Schema and semantic validation against the OWL ontology/SHACL constraints; those checks, package/component completeness, root-element rules, artifact digest verification, provenance, signing, and integrated release acceptance remain separate work. Envelope success must never be presented as full SPDX conformance.

## Failure and degraded behavior

Manifest-binding failures occur before a binding is produced. Envelope failures return deterministic redacted error codes and no document-controlled value is echoed into the diagnostic. Oversized input is rejected before JSON parsing. Invalid UTF-8, invalid JSON, duplicate keys, incorrect context, invalid top-level shape, non-object graph entries, excessive graph cardinality, and invalid `SpdxDocument` cardinality all fail closed.

Failure at either layer does not authorize release through another path. There is no alternate version fallback, network context fallback, permissive parse mode, or silent default success.

## Security / privacy / governance impact

The boundary reduces supply-chain ambiguity without introducing ambient network authority. Bounded parsing constrains memory exposure before deeper validation, duplicate-key rejection prevents parser interpretation drift, exact context matching prevents version drift, and redacted diagnostics prevent supplier-controlled SBOM bytes from being reflected into CI/release logs. The verifier introduces no secrets, personal data, signing material, privileged network access, reviewer authority, or release authority.

## Tests and acceptance evidence

The Rust identity tests must continue to prove exact SPDX specification/context identity, manifest-backed SBOM admission, complete coverage of every non-SBOM manifest artifact, deterministic ordering, and typed fail-closed binding behavior.

The Python release-verifier contract tests must prove:

- exact SPDX 3.0.1 context and one `SpdxDocument` are admitted;
- wrong context and unexpected top-level members fail closed;
- duplicate JSON keys, malformed UTF-8, non-finite constants, invalid graph entries, excessive graph size, and zero/multiple documents fail closed; and
- hostile external bytes never appear in diagnostics.

Repository Python contracts, Rust 1.97.1 formatting/check/tests/Clippy/rustdoc, exact Rust production function/line/region/branch coverage, and any applicable browser/security gates must pass on the unchanged exact head. Predecessor evidence does not transfer.

## Migration and rollback

This remains a pre-release branch with no protected-main persisted SBOM schema migration. The verifier has no network or durable state. Before acceptance it can be revised or withdrawn with its owning branch. If a durable external release/SBOM format depends on this boundary, incompatible context, size, graph, or error-contract changes require explicit versioning and migration rather than silent parser broadening.

## Open follow-ups

- Generate deterministic SPDX 3.0.1 SBOM content for supported release packages.
- Validate the same candidate bytes against the reviewed SPDX 3.0.1 JSON Schema and OWL/SHACL resources using immutable, offline-verifiable identities.
- Define package/component completeness and root-element rules for OriginWeave distribution artifacts.
- Verify the bound SBOM artifact digest before content admission and compose the result with SLSA provenance, signing identity, timestamps, platform packages, reproducibility evidence, and updater trust.
- Complete issue #201's integrated release acceptance and rollback/freeze-protection architecture.

## Supersession / reversal conditions

Supersede this ADR when a versioned external release/SBOM specification replaces the in-process binding and envelope contract, when OriginWeave adopts a different SBOM standard under a reviewed migration, or when a stronger integrated provenance model subsumes these boundaries. Reversal must preserve fail-closed manifest-backed identity and must not make metadata or partial parsing equivalent to release authority.

## References

SPDX Workgroup. (2026). *SPDX specification 3.0.1*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/

SPDX Workgroup. (2026). *SPDX specification 3.0.1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/serializations/
