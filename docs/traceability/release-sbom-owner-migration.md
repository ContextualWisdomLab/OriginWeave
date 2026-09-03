# Release SBOM canonical-owner migration

- Status: Proposed
- Owning gap: #201
- Repair PR: #221
- Release-context owner candidate: #240

This record is migration evidence for active pre-GA work. Active PR metadata is **not shipped truth**; protected `main` remains implementation truth until the dependency-ordered reconstruction, exact-head verification, and normal governance path complete.

## Problem

PR #221 currently carries release-specific SBOM identity and completeness semantics in `originweave-core` and implements release-facing digest, hostile JSON-LD envelope, and local file admission canonically in `scripts/release/validate_spdx_jsonld.py`. Four independently valid constraints now make that placement provisional:

1. OriginWeave's first-party security/runtime boundary is Rust-first. Digest correspondence, bounded hostile JSON parsing, and descriptor-relative/no-follow file admission have practical Rust implementations, so Python cannot remain the canonical security boundary without an explicit no-practical-Rust rationale and removal condition.
2. Active PR #240 establishes `originweave-release` as the dedicated Release bounded context and moves release policy/evidence ownership out of `originweave-core`. Shipping #221 unchanged would therefore create competing canonical ownership.
3. The current Python secure-open path deliberately fails closed when POSIX-style `O_NOFOLLOW` plus descriptor-relative `os.open(..., dir_fd=...)` are unavailable. That is safe for the provisional verifier, but #201 is a cross-platform distribution/release track. The final Rust owner must preserve the same anti-symlink/reparse and identity-stability guarantees on each supported release platform instead of making a Unix-only primitive the commercial release contract.
4. A 16 MiB input-byte ceiling and 65,536 top-level `@graph` objects did not bound parser heap amplification. The first reproduction used a compact nested-container payload; a second reproduction showed that a roughly 2 MiB document with more than 1,048,576 scalar array members also passed the container-only preflight and was materialized by `json.loads`. The provisional boundary therefore now applies both a 524,288 opening-container ceiling and a 1,048,576 JSON structure-token ceiling before generic-tree materialization. The Rust migration must preserve an equivalent or stricter pre-materialization resource invariant rather than deserialize hostile input into an unbounded generic tree first.

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
- a pre-materialization opening-container budget equivalent to or stricter than the provisional 524,288-container ceiling;
- a broader pre-materialization JSON structure-token budget equivalent to or stricter than the provisional 1,048,576-token ceiling so scalar/member fan-out is bounded as well as container count;
- bracket/brace/comma/colon text inside JSON strings must not consume structural budgets, while malformed JSON still fails in the grammar parser;
- strict UTF-8 and JSON admission;
- duplicate object-key and non-finite numeric rejection;
- exact SPDX 3.0.1 global context identity for this preliminary gate;
- top-level `@graph` object shape and bounded graph cardinality;
- exactly one top-level `SpdxDocument` and rejection of any additional nested `SpdxDocument`;
- canonical lowercase `sha256:` manifest/SBOM digest correspondence;
- value-redacted diagnostics that do not echo hostile document bytes, expected digest values, or candidate paths;
- regular-file admission through an already-opened, identity-stable handle rather than path-only trust;
- on POSIX-capable targets, descriptor-relative `openat`-class traversal with no-follow semantics and identity revalidation before/during/after the bounded read;
- on Windows targets, use reviewed handle-based traversal/admission that prevents ordinary reparse-point following. The concrete leaf/reparse admission primitive must include `CreateFileW` with `FILE_FLAG_OPEN_REPARSE_POINT`; identity checks must use `GetFileInformationByHandleEx(FileIdInfo)` / `FILE_ID_INFO`, comparing the `VolumeSerialNumber` plus `FileId` for the opened handle before and after the bounded read. `CreateFileW` alone is not treated as proof that parent-directory swaps are impossible; the final implementation must use an equivalent handle-relative parent traversal or another reviewed mechanism and prove it with junction/reparse and parent-swap regressions; and
- fail-closed behavior on a platform where equivalent anti-redirection and identity-stability guarantees cannot be established. Portability must not silently degrade to lexical/path-only checks.

Digest/envelope success remains correspondence evidence only. It is not full SPDX JSON Schema/OWL/SHACL conformance, SLSA provenance, reproducibility, signature verification, or release approval.

## Dependency-ordered repair

1. Preserve #221's validated manifest-join, digest, envelope, hostile-input, file-admission tests and exact historical evidence as reconstruction input. Do not close the PR or discard its unique delta.
2. Resolve the dedicated Release owner lineage through #240 without treating its active branch as protected-main truth. Read intervening changes and reconstruct non-destructively rather than force-rebasing or copying the whole branch.
3. On the resulting `originweave-release` lineage, add the smallest realistic failing Rust test for each migrated canonical invariant and observe the failure before production implementation. Include both compact nested-container and scalar/member fan-out cases that remain well below the byte ceiling, Linux/macOS symlink-swap and parent-replacement cases, plus Windows reparse-point/junction and handle-identity cases for every platform claimed by #201.
4. Implement the smallest Rust-owned admission boundary that depends inward on the core `ReleaseManifest` identity contract. Parse hostile JSON through a bounded or streaming path that enforces byte, container, structure-token/member, graph, numeric, and diagnostic limits before an unbounded generic tree can be materialized. Keep platform-specific secure-open mechanics behind Release-owned adapters; do not introduce a core-to-release dependency, source copy, cross-context SQL, generic JSON authority, path-only portability fallback, or permissive fallback.
5. Reduce `scripts/release/validate_spdx_jsonld.py` to a thin conformance/fixture harness only after Rust owns the release decision boundary and parity regressions prove the behavior.
6. Update ADR 0018, `ARCHITECTURE.md`, `CHANGELOG.md`, doctoring/TRACEABILITY, and the product/technical gap baseline to name the final owner and distinguish protected-main truth from active-PR evidence.
7. Require unchanged exact-head Rust 1.97.1 formatting/check/tests/strict Clippy/rustdoc, 100% owned-production function/line/region/branch coverage, applicable security/review gates, current live-base verification, and supported-platform parity before any readiness transition.

## Current exact evidence

The migration contract was introduced test-first on #221 after reviews `5095297316` (Rust-first boundary) and `5095598850` (DDD owner conflict). The predecessor #221 head was `f341ab946beb509f53a9cc8cc52fc20d650faa87`; the test-only owner-migration contract is commit `f3a519e986dc6bda46fb03ac3768ea2bef7a131d`. Review `5096271206` adds the cross-platform secure-open finding after confirming that the provisional Python implementation explicitly rejects platforms without POSIX-style no-follow and descriptor-relative open support. Active #240 was last independently read at `47ab81b721aca246948607754be1807b9d4c8dda` and was not exact-head GREEN.

The parser-amplification repair was performed test-first in two passes. Test-only `e9c43d9940c03df4ead17f1e2b0b80efa5fcfd0f` exposed nested-container amplification below the byte ceiling; `3e2b4a39b0bb22309f70cf88aaa88a536ec25d1d` added the first string-aware opening-container preflight and `0317263f6e412c2040b4bdfe578781c6fa24a11c` covered escaped-string handling. Follow-up review found that a flat nested scalar array could still create more than one million parsed values with only a handful of containers. Test-only `cbd0d9629d4d5ce42d217c47c54e5f6d33e83255` exposes that scalar fan-out; production `d301f8f79e34f3f09744b0c38c71a54e58ad6283` adds the broader JSON structure-token budget; and `96e8dbd2ad0414481b0ff6ff0e841751147fb780` separates the container-specific and scalar-specific edge cases using the production-exported limits. These are provisional Python hardening commits while the Rust owner migration remains dependency-ordered; they are not evidence that the canonical Rust release boundary exists.

This document is the causal documentation fix for the missing durable owner-migration, parser-resource, and portability contracts. It does not claim that the Rust migration itself has occurred, that Windows/macOS parity has been implemented, that #240 has shipped, or that #221 is exact-head GREEN.

## Standards traceability

Bray, T. (2017). *The JavaScript Object Notation (JSON) Data Interchange Format* (RFC 8259; STD 90). Internet Engineering Task Force. https://www.rfc-editor.org/rfc/rfc8259

SPDX Workgroup. (2026). *SPDX specification 3.0.1*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/

SPDX Workgroup. (2026). *SPDX specification 3.0.1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/serializations/

SPDX Workgroup. (2026). *SpdxDocument*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/model/Core/Classes/SpdxDocument/

The Open Group. (2024). *open, openat — open file* (POSIX.1-2024). https://pubs.opengroup.org/onlinepubs/9799919799/functions/open.html

Microsoft. (n.d.). *CreateFileW function (fileapi.h)*. Microsoft Learn. Retrieved September 3, 2026, from https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew

Microsoft. (2024). *GetFileInformationByHandleEx function (winbase.h)*. Microsoft Learn. https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex

Microsoft. (2024). *FILE_ID_INFO structure (winbase.h)*. Microsoft Learn. https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info
