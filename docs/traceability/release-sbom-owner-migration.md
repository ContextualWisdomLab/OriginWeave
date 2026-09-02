# Release SBOM canonical-owner migration

- Status: Proposed
- Owning gap: #201
- Repair PR: #221
- Release-context owner candidate: #240

This record is migration evidence for active pre-GA work. Active PR metadata is **not shipped truth**; protected `main` remains implementation truth until the dependency-ordered reconstruction, exact-head verification, and normal governance path complete.

## Problem

PR #221 currently carries release-specific SBOM identity and completeness semantics in `originweave-core` and implements release-facing digest, hostile JSON-LD envelope, and local file admission canonically in `scripts/release/validate_spdx_jsonld.py`. Three independently valid constraints now make that placement provisional:

1. OriginWeave's first-party security/runtime boundary is Rust-first. Digest correspondence, bounded hostile JSON parsing, and descriptor-relative/no-follow file admission have practical Rust implementations, so Python cannot remain the canonical security boundary without an explicit no-practical-Rust rationale and removal condition.
2. Active PR #240 establishes `originweave-release` as the dedicated Release bounded context and moves release policy/evidence ownership out of `originweave-core`. Shipping #221 unchanged would therefore create competing canonical ownership.
3. The current Python secure-open path deliberately fails closed when POSIX-style `O_NOFOLLOW` plus descriptor-relative `os.open(..., dir_fd=...)` are unavailable. That is safe for the provisional verifier, but #201 is a cross-platform distribution/release track. The final Rust owner must preserve the same anti-symlink/reparse and identity-stability guarantees on each supported release platform instead of making a Unix-only primitive the commercial release contract.

## Target ownership

`ReleaseManifest` remains a stable cross-context identity contract in `originweave-core`. The Release context depends inward on that contract; core must not depend outward on Release.

After the #240 owner lineage is available for integration, `originweave-release` is the proposed canonical owner for:

- SPDX serialization identity and SBOM binding;
- manifest-to-SBOM described-artifact completeness policy;
- release-facing SHA-256 correspondence;
- bounded hostile JSON-LD envelope admission;
- identity-stable local SBOM file admission; and
- later full SPDX structural/semantic conformance and its release evidence.

There is **no compatibility shim** that makes core depend on Release, and there is **no duplicated release API** split across `originweave-core` and `originweave-release`. The active #240 branch is evidence of the intended owner boundary, not shipped truth and not authority to copy its tree wholesale.

`scripts/release/validate_spdx_jsonld.py` may remain only as a **thin conformance/fixture harness** after the Rust-owned admission boundary exists. Python must not remain the canonical source of release/security authority.

## Invariants that reconstruction must preserve

The Rust-owned replacement must preserve or strengthen the currently reviewed fail-closed behavior:

- maximum 16 MiB candidate input before unbounded parsing/allocation;
- strict UTF-8 and JSON admission;
- duplicate object-key and non-finite numeric rejection;
- exact SPDX 3.0.1 global context identity for this preliminary gate;
- top-level `@graph` object shape and bounded graph cardinality;
- exactly one top-level `SpdxDocument` and rejection of any additional nested `SpdxDocument`;
- canonical lowercase `sha256:` manifest/SBOM digest correspondence;
- value-redacted diagnostics that do not echo hostile document bytes, expected digest values, or candidate paths;
- regular-file admission through an already-opened, identity-stable handle rather than path-only trust;
- on POSIX-capable targets, descriptor-relative `openat`-class traversal with no-follow semantics and identity revalidation before/during/after the bounded read;
- on Windows targets, reviewed handle-based no-reparse traversal/admission semantics that prevent ordinary reparse-point following and revalidate file/directory identity around the bounded read; and
- fail-closed behavior on a platform where equivalent anti-redirection and identity-stability guarantees cannot be established. Portability must not silently degrade to lexical/path-only checks.

Digest/envelope success remains correspondence evidence only. It is not full SPDX JSON Schema/OWL/SHACL conformance, SLSA provenance, reproducibility, signature verification, or release approval.

## Dependency-ordered repair

1. Preserve #221's validated manifest-join, digest, envelope, hostile-input, file-admission tests and exact historical evidence as reconstruction input. Do not close the PR or discard its unique delta.
2. Resolve the dedicated Release owner lineage through #240 without treating its active branch as protected-main truth. Read intervening changes and reconstruct non-destructively rather than force-rebasing or copying the whole branch.
3. On the resulting `originweave-release` lineage, add the smallest realistic failing Rust test for each migrated canonical invariant and observe the failure before production implementation. Include Linux/macOS symlink-swap and parent-replacement cases plus Windows reparse-point/junction and handle-identity cases for every platform claimed by #201.
4. Implement the smallest Rust-owned admission boundary that depends inward on the core `ReleaseManifest` identity contract. Keep platform-specific secure-open mechanics behind Release-owned adapters; do not introduce a core-to-release dependency, source copy, cross-context SQL, generic JSON authority, path-only portability fallback, or permissive fallback.
5. Reduce `scripts/release/validate_spdx_jsonld.py` to a thin conformance/fixture harness only after Rust owns the release decision boundary and parity regressions prove the behavior.
6. Update ADR 0018, `ARCHITECTURE.md`, `CHANGELOG.md`, doctoring/TRACEABILITY, and the product/technical gap baseline to name the final owner and distinguish protected-main truth from active-PR evidence.
7. Require unchanged exact-head Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, 100% owned-production function/line/region/branch coverage, applicable security/review gates, current live-base verification, and supported-platform parity before any readiness transition.

## Current exact evidence

The migration contract was introduced test-first on #221 after reviews `5095297316` (Rust-first boundary) and `5095598850` (DDD owner conflict). The predecessor #221 head was `f341ab946beb509f53a9cc8cc52fc20d650faa87`; the test-only owner-migration contract is commit `f3a519e986dc6bda46fb03ac3768ea2bef7a131d`. Review `5096271206` adds the cross-platform secure-open finding after confirming that the provisional Python implementation explicitly rejects platforms without POSIX-style no-follow and descriptor-relative open support. Active #240 was last independently read at `47ab81b721aca246948607754be1807b9d4c8dda` and was not exact-head GREEN.

This document is the causal documentation fix for the missing durable owner-migration and portability contract. It does not claim that the Rust migration itself has occurred, that Windows/macOS parity has been implemented, that #240 has shipped, or that #221 is exact-head GREEN.

## Standards traceability

SPDX Workgroup. (2026). *SPDX specification 3.0.1*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/

SPDX Workgroup. (2026). *SPDX specification 3.0.1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/serializations/

SPDX Workgroup. (2026). *SpdxDocument*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/model/Core/Classes/SpdxDocument/

The Open Group. (2024). *open, openat — open file* (POSIX.1-2024). https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html

Microsoft. (n.d.). *CreateFileW function (fileapi.h)*. Microsoft Learn. Retrieved September 3, 2026, from https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
