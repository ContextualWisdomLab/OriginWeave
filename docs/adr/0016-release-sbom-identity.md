# ADR 0016: Release SBOM identity binding and bounded JSON-LD envelope verification

- Status: Proposed
- Date: 2026-08-23

## Context

Issue #201 requires acquisition-grade release evidence, including a software bill of materials tied to the same immutable release identity buyers verify. ADR 0015 deliberately stops at a bounded release-manifest identity and does not claim SBOM content conformance, provenance, signing, publication, installation, update, rollback, or release authority.

SPDX 3.0.1 defines JSON-LD serialization and separate structural and semantic validation requirements. The serialization prose requires the versioned global context `https://spdx.org/rdf/3.0.1/spdx-context.jsonld`, defines `spdxId` as an alias for `@id` and `type` as an alias for `@type`, and says additional namespace mappings may be defined in a separate context object. The same SPDX 3.0.1 specification requires structural validation against its official JSON Schema. That schema constrains top-level `@context` with a `const` equal to the exact versioned context string.

Those two official 3.0.1 resources therefore expose an interoperability tension: the prose describes extension mappings that JSON-LD can represent with context composition, while the published structural schema accepts only the exact context string. More importantly for this preliminary verifier, an arbitrary inline JSON-LD context object is not merely a namespace map: JSON-LD control terms such as `@import`, `@base`, and `@vocab`, or redefinitions of SPDX aliases such as `type` and `spdxId`, can change downstream interpretation. Treating every inline object as a benign namespace map would therefore create semantic and remote-authority ambiguity before schema/ontology validation.

OriginWeave separates four claims that must not be collapsed:

1. the declared SPDX serialization identity;
2. the immutable release-manifest/SBOM artifact identity join, including the exact manifest-backed SBOM SHA-256 digest;
3. a narrow, bounded check of the actual JSON-LD serialization envelope over those exact candidate bytes; and
4. full SPDX content conformance and product-specific package/component completeness.

This slice implements claims 1 through 3 only. Claim 4 remains a later independently reviewed release gate.

## Decision drivers

- SBOM identity must not drift away from the exact release manifest it supplements.
- The supported SPDX JSON-LD format must expose its exact specification version and required versioned global context identity rather than rely on an ambient or moving context.
- The preliminary envelope gate must not interpret arbitrary JSON-LD context controls or term redefinitions as harmless namespace declarations.
- The gate should remain structurally compatible with the official SPDX 3.0.1 JSON Schema until a reviewed schema-aware extension policy exists.
- Actual serialized bytes must not be accepted merely because metadata says they are SPDX 3.0.1 JSON-LD.
- A syntactically valid but substituted SPDX document must not satisfy release evidence when its bytes do not match the exact SBOM artifact digest already admitted by the release manifest.
- Envelope parsing must be bounded before deeper semantic use and reject malformed UTF-8, malformed JSON, duplicate JSON object members, non-finite JSON constants, unexpected top-level members, non-object graph entries, excessive graph cardinality, and zero or multiple `SpdxDocument` objects.
- Error diagnostics must not reflect attacker- or supplier-controlled document bytes or release digest values.
- The SBOM artifact itself must already be an admitted release artifact with a manifest-bound SHA-256 identity.
- Every non-SBOM release artifact in the bound manifest must be represented exactly once in the described-artifact inventory.
- Empty, incomplete, duplicate, foreign, self-described, and case-drifted artifact identities must fail closed.
- Partial digest/envelope validation must not be represented as SPDX JSON Schema/OWL/SHACL conformance, cryptographic provenance, signing, publication, installation, update, rollback, or release authority.

## Assumptions and authority boundaries

- ADR 0015 remains the owning decision for release-manifest identity admission.
- `ReleaseSbomFormat::Spdx30JsonLd` identifies SPDX 3.0.1 serialized as JSON-LD and exposes the exact required global context URI.
- `ReleaseSbomBinding` can reference only exact, case-sensitive artifact names already present in one `ReleaseManifest`.
- The SBOM artifact is represented by the existing manifest-backed `ReleaseArtifact`, including its lowercase `sha256:` identity, and cannot also appear in the binding's described-artifact set.
- Every other artifact admitted by that release manifest must be retained in the described-artifact set exactly once. This is an OriginWeave release-inventory completeness rule, not a general SPDX modeling assertion.
- `scripts/release/validate_spdx_jsonld.py` consumes actual bytes for a narrow envelope check. It accepts at most 16 MiB, requires strict UTF-8 JSON, requires exactly the top-level `@context` and `@graph` members, and requires `@context` to equal the exact SPDX 3.0.1 global-context string. Context arrays and inline/remote context extensions fail closed at this stage.
- Release admission that composes those bytes with `ReleaseSbomBinding` uses `validate_release_spdx_3_0_1_jsonld_bytes` and supplies the exact canonical lowercase `sha256:` identity from `ReleaseSbomBinding::sbom_artifact()`. The helper recomputes SHA-256 from the bounded candidate bytes and rejects malformed expected identities or mismatches before promoting envelope evidence.
- Digest equality proves correspondence to the declared manifest artifact identity only. It does not authenticate who produced the manifest or bytes and does not replace provenance, signature, transparency, reproducibility, or release-approval checks.
- The exact-string context choice is deliberately narrower than the SPDX serialization prose because the official 3.0.1 JSON Schema itself uses that exact-string constraint and because this preliminary verifier does not implement complete JSON-LD context semantics or collision detection against the pinned SPDX context.
- The verifier does not fetch remote contexts or schemas, resolve external identifiers, validate the full SPDX JSON Schema, evaluate the SPDX ontology/SHACL constraints, infer package completeness, or grant any signing/release/update authority.
- Full conformance still requires validation of the same digest-bound serialized bytes against reviewed SPDX 3.0.1 structural and semantic resources plus OriginWeave product-completeness rules.

## Options considered

### Trust the declared SBOM format without reading bytes

Rejected. A manifest can correctly identify an artifact while a different candidate file is supplied, or while the candidate bytes contain the wrong context, malformed JSON, an ambiguous duplicate-key object, or multiple `SpdxDocument` definitions.

### Validate an SPDX envelope without binding the bytes to the declared artifact digest

Rejected for release admission. A completely valid SPDX document for another release could pass the envelope gate and be associated with the wrong `ReleaseSbomBinding`. The release-facing helper therefore hashes the exact bounded candidate bytes and compares the canonical `sha256:` identity to the manifest-backed SBOM artifact before returning release evidence.

### Admit the exact SPDX context plus arbitrary inline context objects

Rejected. An inline JSON-LD object can contain `@import`, `@base`, `@vocab`, or term/alias redefinitions rather than only a new namespace prefix. Without a complete pinned-context collision model, accepting those objects would allow the preliminary gate to count raw `type` values under semantics that later JSON-LD processing may reinterpret.

### Require `@context` to be only the SPDX global-context string

Selected for this preliminary gate. The official SPDX 3.0.1 JSON Schema constrains `@context` to that exact string, this representation is used by the specification's normative example, and the rule prevents context-array semantic or remote-authority ambiguity. SPDX prose support for additional namespace mappings is retained as an explicit interoperability follow-up rather than silently approximated with an unsafe object-type check.

### Fetch the JSON-LD context or schema during the first verifier

Rejected. Ambient network retrieval would introduce moving external authority and availability into a release gate before pinned schema/ontology identities and offline verification are designed. The first verifier uses fixed local constants and no network access.

### Treat a general-purpose JSON parse as SPDX conformance

Rejected. JSON syntax alone does not establish the required SPDX context, document cardinality, JSON Schema validity, ontology/SHACL validity, or SBOM completeness.

### Store an arbitrary SBOM path or URL next to the manifest

Rejected. An ambient path or URL can drift from the bounded release inventory and does not guarantee that the SBOM itself is one of the artifacts buyers can hash against the manifest.

### Accept only a selected subset of manifest artifacts

Rejected. A non-empty manifest-backed subset can look well formed while silently omitting another release artifact. OriginWeave therefore requires every non-SBOM manifest artifact to appear exactly once in the described-artifact inventory.

### Allow the SBOM artifact to describe itself

Rejected for OriginWeave's release identity contract. A self-referential described-artifact set makes the artifact-to-SBOM join circular at the exact-digest boundary verification must evaluate. This is an OriginWeave integrity rule, not a claim that SPDX generally forbids such a relationship.

## Decision

OriginWeave adopts the following release-SBOM boundary:

1. The initial format is `ReleaseSbomFormat::Spdx30JsonLd`, reporting SPDX specification version `3.0.1` and global context URI `https://spdx.org/rdf/3.0.1/spdx-context.jsonld`.
2. The named SBOM artifact must exactly match an artifact admitted by the same `ReleaseManifest`.
3. At least one distinct described release artifact is required.
4. Every non-SBOM artifact admitted by the same release manifest must appear exactly once in the described-artifact inventory.
5. The SBOM artifact itself cannot be one of its described artifact identities.
6. Foreign, duplicate, incomplete, and case-drifted described artifact identities fail closed.
7. Described artifact identities are stored deterministically and the manifest's existing bound transitively limits the inventory.
8. Release-facing validation of actual candidate bytes uses `validate_release_spdx_3_0_1_jsonld_bytes` with the exact `sha256:` identity from the manifest-backed SBOM artifact. The expected digest must be `sha256:` followed by 64 lowercase hexadecimal digits; the verifier recomputes SHA-256 over the same bounded bytes and a mismatch fails closed with a value-redacted typed error.
9. Only after exact digest correspondence is established do those candidate bytes pass through the narrow `validate_spdx_3_0_1_jsonld_bytes` envelope contract. The lower-level function remains available for composition and testing but is not by itself release-artifact correspondence evidence.
10. The envelope verifier admits only non-empty strict-UTF-8 payloads no larger than 16 MiB, rejects duplicate JSON keys and non-finite constants, requires exactly the top-level `@context` and `@graph` members, and requires `@context` to equal the exact versioned SPDX 3.0.1 context string. Context arrays and all additional context entries fail closed. It allows at most 65,536 graph objects, requires object entries with string `type`, and requires exactly one raw `SpdxDocument` under the pinned alias semantics.
11. Validation errors expose stable error codes and generic diagnostics without reflecting document bytes or expected digest values.
12. Digest/envelope success is not a certificate of complete SPDX JSON Schema validity, semantic ontology/SHACL validity, package/component completeness, producer authenticity, provenance, signatures, publication, installation, update, rollback, or release authority.
13. No verifier fallback may substitute another SPDX version, discard the required pinned context or expected artifact digest, reinterpret an unvalidated context extension, discard duplicate keys, silently truncate a graph, fetch an ambient remote resource, or convert malformed or mismatched content into success.

## Consequences

Downstream release tooling receives an exact versioned serialization identity, a non-circular and release-inventory-complete identity join, and a bounded first check over the exact bytes named by that join. A syntactically valid SPDX document for a different artifact cannot satisfy the release-facing helper merely because its envelope is valid. A candidate also cannot pass this layer with the wrong SPDX context, a context array that can alter JSON-LD interpretation, an extra top-level field, malformed JSON, duplicate members, a non-object graph entry, an unbounded graph, or zero/multiple `SpdxDocument` objects.

The exact-context rule can reject a serialization that the SPDX 3.0.1 prose intends to allow through additional namespace mappings. That false negative is accepted at this preliminary gate because the same version's official structural schema currently requires the exact string and because OriginWeave does not yet possess a reviewed, schema-aware rule capable of proving that an extension object is namespace-only and non-colliding. Support for such mappings must be added only together with pinned full structural/semantic validation or another equally strong proof of interpretation.

The boundary remains intentionally narrower than a commercial SBOM generator or complete verifier. The official SPDX 3.0.1 specification requires structural validation against the JSON Schema and semantic validation against the OWL ontology/SHACL constraints; those checks, package/component completeness, root-element rules, producer/provenance authentication, signing, and integrated release acceptance remain separate work. Digest/envelope success must never be presented as full SPDX conformance or signed supply-chain provenance.

## Failure and degraded behavior

Manifest-binding failures occur before a binding is produced. Release-facing byte verification rejects a malformed expected digest or candidate-byte digest mismatch with deterministic value-redacted error codes. Envelope failures likewise return deterministic redacted error codes and no document-controlled value is echoed into the diagnostic. Oversized input is rejected before hashing or JSON parsing. Invalid UTF-8, invalid JSON, duplicate keys, missing or incorrect required context, context arrays/extensions, invalid top-level shape, non-object graph entries, excessive graph cardinality, and invalid `SpdxDocument` cardinality all fail closed.

Failure at either layer does not authorize release through another path. There is no alternate digest, alternate version fallback, network context fallback, permissive parse mode, or silent default success.

## Security / privacy / governance impact

The boundary reduces supply-chain ambiguity without introducing ambient network authority. Exact SHA-256 correspondence prevents a different but otherwise valid SPDX document from being promoted under the manifest-backed SBOM identity. Bounded parsing constrains memory exposure before deeper validation, duplicate-key rejection prevents parser interpretation drift, and the pinned exact SPDX context prevents version drift, remote `@import`, `@base`/`@vocab` mutation, and term/alias rebinding from being treated as already-admitted envelope semantics. Redacted diagnostics prevent supplier-controlled SBOM bytes or expected digest values from being reflected into CI/release logs. The verifier introduces no secrets, personal data, signing material, privileged network access, reviewer authority, or release authority.

## Tests and acceptance evidence

The Rust identity tests must continue to prove exact SPDX specification/context identity, manifest-backed SBOM admission, complete coverage of every non-SBOM manifest artifact, deterministic ordering, and typed fail-closed binding behavior.

The Python release-verifier contract tests must prove:

- exact candidate bytes match the canonical lowercase manifest-backed `sha256:` identity;
- a valid but substituted SPDX document fails closed on digest mismatch without reflecting document-controlled bytes;
- malformed, uppercase, truncated, or prefixless expected digests fail closed;
- exact SPDX 3.0.1 context and one `SpdxDocument` are admitted;
- context arrays, including apparently simple namespace mappings, fail closed until schema-aware extension validation exists;
- `@import`, `@base`, `@vocab`, `type`, and `spdxId` context overrides fail closed;
- wrong context, additional remote context entries, and unexpected top-level members fail closed;
- duplicate JSON keys, malformed UTF-8, non-finite constants, invalid graph entries, excessive graph size, and zero/multiple documents fail closed; and
- hostile external bytes never appear in diagnostics.

Repository Python contracts, Rust 1.97.1 formatting/check/tests/Clippy/rustdoc, exact Rust production function/line/region/branch coverage, and any applicable browser/security gates must pass on the unchanged exact head. Predecessor evidence does not transfer.

## Migration and rollback

This remains a pre-release branch with no protected-main persisted SBOM schema migration. The verifier has no network or durable state. Before acceptance it can be revised or withdrawn with its owning branch. If a durable external release/SBOM format depends on this boundary, incompatible digest, context, size, graph, or error-contract changes require explicit versioning and migration rather than silent parser broadening.

## Open follow-ups

- Generate deterministic SPDX 3.0.1 SBOM content for supported release packages.
- Validate the same digest-bound candidate bytes against reviewed, immutable SPDX 3.0.1 JSON Schema and OWL/SHACL resources using offline-verifiable identities.
- Reconcile the SPDX 3.0.1 prose allowance for additional namespace mappings with the official 3.0.1 JSON Schema's exact-string `@context` constraint before admitting context composition.
- Define package/component completeness and root-element rules for OriginWeave distribution artifacts.
- Compose the digest-bound SBOM result with SLSA provenance, signing identity, timestamps, platform packages, reproducibility evidence, and updater trust.
- Complete issue #201's integrated release acceptance and rollback/freeze-protection architecture.

## Supersession / reversal conditions

Supersede this ADR when a versioned external release/SBOM specification replaces the in-process binding and envelope contract, when OriginWeave adopts a different SBOM standard under a reviewed migration, or when a stronger integrated provenance model subsumes these boundaries. Reversal must preserve fail-closed manifest-backed identity and must not make metadata or partial parsing equivalent to release authority.

## References

SPDX Workgroup. (2026). *SPDX specification 3.0.1*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/

SPDX Workgroup. (2026). *SPDX specification 3.0.1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/serializations/

SPDX Workgroup. (2026). *SPDX 3.0.1 JSON Schema*. The Linux Foundation. https://spdx.org/schema/3.0.1/spdx-json-schema.json

World Wide Web Consortium. (2020, July 16). *JSON-LD 1.1: A JSON-based serialization for linked data*. https://www.w3.org/TR/json-ld11/
