# Product and Technical Gap Baseline

This file is the buyer-facing decision surface for the live OriginWeave delivery gap. Protected-main code, exact GitHub state, executable browser evidence, and immutable owner contracts remain authoritative; this document does not turn an active PR or a protocol command acknowledgement into shipped behavior.

The detailed historical dossier that previously occupied this path is preserved byte-for-byte at [`docs/evidence/product-technical-gap-baseline-through-2026-09-09.md`](evidence/product-technical-gap-baseline-through-2026-09-09.md). Historical measurements are evidence for their recorded generation, not transferable acceptance for a later head.

## Observed delivery cut — 2026-09-21 04:07 UTC

This is a dated observation receipt, not a continuously live counter. At this cut, #309 is merged into this #238 documentation lineage, and #238 is Ready for review after its bounded source currentization. Ready is review/execution admission, not merge acceptance. Live GitHub state supersedes this cut after 2026-09-21 04:07 UTC.

- Protected-main truth remains signed-valid `87c4daa1830bac5a5228b6036752ad5633232085`.
- The observed queue contains **135 open pull requests: 6 Ready/non-draft and 129 Draft; 19 open non-PR issues**. The searches used for total/Ready/non-draft/issue counts returned `incomplete_results=false`; Draft is the exact difference between the complete total and Ready/non-draft counts.
- Open issues: the 19 non-PR items above remain separate from the 135-PR delivery queue and are not counted as shipped product behavior.
- Ruleset `18156473` remains the protected-integration authority recorded by this lineage; exact current rules/reviews/checks must still be re-read immediately before any merge.
- GitHub Release inventory is empty. There is no immutable OriginWeave release to treat as commercial acceptance; tag/release state must still be re-read before any release claim.
- The prior `CHANGELOG.md` delivery-inventory line remains the explicitly dated **2026-09-09 12:57 UTC** receipt. It is historical evidence, not a live queue counter, and is not rewritten merely to mirror this later cut.

Phase 1 is **in progress**, not shipped. A successful source test, an active-PR head, a protocol ACK, or a clean security scan is not protected-main/browser/release acceptance by itself.

## Buyer gap matrix

| Buyer-visible gap | Evidence at this cut | Canonical owner / next acceptance | State |
| --- | --- | --- | --- |
| Complete governed presentation identity in real Chromium | #299 is Draft/mergeable at exact `4c9add7b8063fecac578fe3d13d7d5f8cb8cb6d3`. Native CI `35119783030` is GREEN, but real Manifest V3 run `35119783036` is RED on pinned Chrome/ChromeDriver `150.0.7871.129`: ordinary MV3 **0/3** and Agent Task **0/3**, with every Agent Task trial failing before navigation as `AgentTaskSessionStartError` caused by `WebDriverSessionNotCreatedError`. | Browser Session/presentation semantics remain in OriginWeave. Canonical `.github#1857` is Draft/mergeable at exact `a6d16c0f36e31da539ee98d550277d2169e33514`; its Node-24-native upload-artifact repair and sandbox contract are not yet protected/consumable. After that owner generation lands normally, pin its immutable protected revision and replay three independent trials through ambient observation → apply → page-observed target → native interaction/outcome → reset → observed baseline → session/profile cleanup. | **OPEN** |
| Trusted docs-only CI classifier before workflow suppression | Ready #287 remains exact `3975daf48e01a5e9d1cf9fb104a3be1aa03b0402` directly on protected main. Native CI `34329801487`, Security Scan `34329801483`, and Semgrep `34329801598` succeed; CodeQL `34329801477` fails at the central verdict path rather than an observed classifier-source finding. | #287 must obtain the required central verdict and normal independent review, reach protected main, then the authorized #279/#212 workflow-owner path may consume the protected classifier generation. | **OPEN** |
| One governed Rust stable baseline across product contracts and workflow materialization | Draft #308 remains exact `0cf4275d364f529eb3c23dbbfec5ce20113db47e`; it preserves the Rust `1.98.1` successor delta without editing `.github/**`. Native CI is skipped by Draft admission; Security Scan and Semgrep succeed; CodeQL publication/wake remains a central-owner defect. | #279 owns OriginWeave workflow materialization and `.github` owns reusable CodeQL publication/wake mechanics. After owner integration, rerun repository contracts, rustfmt, locked workspace tests, strict Clippy, rustdoc, exact production coverage and all central required checks on one unchanged head. | **OPEN** |
| Buyer decision surface remains usable while evidence history stays immutable | #309 has normally merged its compact-surface delta into the #238 documentation lineage. #238 is Ready on its current documentation lineage; Ready does not make pending/predecessor workflows passing evidence. The former baseline remains byte-for-byte preserved as Git blob `c4a87e6c70f2c48b6e4ef820f18e8ddb0dca5696`. | #238 documentation owner: keep dated observation receipts and the buyer matrix near the top, keep detailed runbooks in immutable evidence/traceability files, and require current-head navigation/freshness regressions, review and required workflows before normal integration. | **IN VERIFICATION** |
| Stacked HTTP `Content-Encoding` interoperability without budget bypass | Canonical HTTP parent #37 remains exact `219b43bfa87ab4fd90a77bb962df2797925bf661`. Active successor #327 is exact `0731ccabbc3498d60a43ecd831d0e2aedc5e265b`, directly stacked **26 ahead / 0 behind**. It carries bounded depth-2 `gzip`/`deflate` decoding, original-coded cumulative expansion accounting, exact-layer raw-DEFLATE compatibility, RFC 9530 pre-decode digest handling, ordered per-layer evidence, Proposed ADR 0011 currentization, and an authenticated-TLS buyer-path performance instrument. The instrument still runs 31 measured samples for representative `gzip, deflate` and `deflate, gzip` profiles and fails above p95 **20,000 µs**. Its receipt provenance is now repaired in source: exact source is supplied by `ORIGINWEAVE_PERFORMANCE_SOURCE_REVISION` (with `GITHUB_SHA` only as an identified fallback), execution identity is mandatory through `ORIGINWEAVE_PERFORMANCE_ENVIRONMENT_ID`, and runtime OS/architecture/parallelism are emitted with every profile line. Exact-head CI `35945874758` is still pre-step queued, so this source/environment-bound receipt format is **not an accepted performance receipt** and no ≤20 ms claim is made. | #326/#327 remain the OriginWeave HTTP owner path. Preserve #37 as the canonical parent; require exact-head repository/security/CodeQL/review acceptance on #327 plus an actually executed source/environment-bound receipt whose explicit source revision matches the intended measured head before ordinary parent-first integration. Do not call the current Vec-based bounded implementation streaming and do not move decoding or policy authority into Chromium, MCP, a model, or another CWL owner. | **OPEN — ACTIVE SUCCESSOR** |
| Controlled-benchmark Unicode evidence identity has stable publication provenance | Ready/mergeable #324 is exact `4e70d5ed9ce13f7b59012d39646e94ac41519c89` with profile `unicode-18.0.0-default-ignorable-exclusion`. Unicode 18.0.0 was formally released 2026-09-16. The stable versioned DICP receipt is **1,159,889 bytes**, SHA-256 `09c928886a178fcafd93c29e4bd59073a058e5a100b716d425cb563ab50f68c9`; `Default_Ignorable_Code_Point` is **27 source entries / 4,174 scalars** and the normalized **29,218 bytes** hash to `673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93`, equal to the implementation expansion. Two-sided regression evidence now separates **valid expected / hostile observed** from **hostile expected / valid observed** across all 4,174 scalars. Exact CI `35629802868` and Manifest V3 Compatibility `35629802889` have **terminal runner-backed execution** and both completed **success**; the CI exact head also completed repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, and owned-production exact coverage. Current-head CodeRabbit review reports **no blocking issue** in the requested expected-side isolation/TRACEABILITY scope. | #325 remains the stack/release acceptance gate. This child GREEN does not bypass #237 → #322 → #324 ancestor order and does not transfer across a future restack. Require #237 exact acceptance and protected integration first, then ordinary/non-force #322 adoption/revalidation/integration, then ordinary/non-force #324 adoption and fresh exact-head executable/review/ruleset evidence. A later authoritative Unicode property mismatch requires a fresh RED and a new profile identity; do not silently mutate this profile. | **OPEN — LEAF GREEN, STACK BLOCKED** |
| Protected merge/release evidence | Protected `main` remains signed-valid `87c4daa1830bac5a5228b6036752ad5633232085`; current GitHub Release inventory is empty. Active candidates still require exact-head checks/reviews and current rule satisfaction. | Normal protected-main integration only after exact-head checks/reviews. A release-ready protected head must then re-read tag/package state and produce version/CHANGELOG, signed artifact, SBOM, provenance, reproducibility and rollback evidence before immutable release. | **OPEN** |

No row above authorizes a local copy of Wardnet, EgressWeave, Keyverse, contextual-orchestrator, Context Fabric, `.github` workflow source, or another repository's mutable PR head. Browser-domain truth stays in OriginWeave; external capabilities enter through released/versioned contracts and ACLs.

## Acceptance order

1. Repair canonical prerequisites in their owner repositories or bounded contexts; do not patch around them in an OriginWeave leaf.
2. Adopt owner deltas by ordinary merge/restack without force-push or destructive rebase and re-read the exact resulting head/base.
3. Run repository contracts and full Rust quality gates on that exact head. Draft/skipped/queued checks are not GREEN.
4. For browser claims, execute realistic pinned Chromium. Success requires browser-observed post-conditions, navigation/session provenance, crash/cleanup handling and all required independent trials; command ACK is insufficient.
5. Satisfy counted review when eligible reviewers exist, required central workflows and thread resolution before protected-main integration; an impossible solo-maintainer approval requirement is held rather than bypassed.
6. Only a protected, release-ready exact head may proceed to signed artifact, SBOM/provenance, reproducibility, rollback, tag/package and immutable release verification.

## Evidence and history index

- [Historical product/technical gap dossier through the 2026-09-09 `16098a0...` generation](evidence/product-technical-gap-baseline-through-2026-09-09.md) — byte-for-byte preservation of the former baseline, including its previous verified cuts, historical PR heads, checks, measurements and runbooks.
- [Historical dossier navigation receipt](evidence/product-technical-gap-baseline-through-2026-09-09-navigation.md) — preserves the archive blob while mapping its original `docs/`-relative link semantics after relocation under `docs/evidence/`.
- [Current decision-surface navigation traceability](traceability/product-gap-baseline-navigation.md) — problem, constraints, alternatives, decision, risks and acceptance contract for separating buyer navigation from evidence history.
- [Product roadmap](product-roadmap.md) — planned delivery sequence; plans do not establish shipped state.
- [PRD](PRD.md) and [TRD](TRD.md) — product intent and technical design with maturity labels.
- [Test strategy](TEST_STRATEGY.md), [operability](OPERABILITY.md), [threat model](THREAT_MODEL.md), and [release/rollback contract](RELEASE_AND_ROLLBACK.md) — acceptance and operating controls.

### Historical contract anchors

The archived dossier retains the full **Observed snapshot: 2026-08-29** and all later dated implementation receipts. The short anchors below exist only so current readers can understand long-lived product boundaries without traversing that dossier. They are not a claim that those historical heads are current.

Protected-main truth is distinct from active implementation. The reviewer-provisioning gap remains an acceptance concern whenever no eligible non-author counted approval is present on a candidate; bot or advisory review text does not silently satisfy the ruleset.

### Open pull requests

Active PRs are implementation/review evidence; none of them is protected-main behavior until merged through the current rules. The historical #195/#198 and #149 anchors below remain useful domain examples, while their exact old heads and measurements live only in the evidence dossier.

#### #195/#198 WebDriver BiDi opening path status

The WebDriver BiDi foundation remains a dependency boundary rather than a shipped-product shortcut. Phase 1 is **in progress**, not shipped. Browser acceptance still requires a real browser session, observed page effects, causal provenance and cleanup evidence on the exact candidate generation.

#### #149 VPN/profile intent status

The profile/VPN contract remains an intent boundary: it does not create a tunnel, route, DNS state, authenticated gateway, or connectivity proof. Network authority belongs to its canonical owner and must be consumed through an explicit released contract rather than inferred from browser configuration.

### Review and merge authority

Ready means review/execution admission, not merge readiness. Re-fetch exact head/base, current reviews, unresolved threads and required checks before any protected merge. Administrative bypass, self-approval, synthesized status, gate weakening, force-push and destructive rebase are not acceptance mechanisms.

## Maintenance rule

Keep this file decision-sized. A new dated observation may replace the bounded observation receipt and matrix evidence, but detailed per-run prose, repeated historical queue dumps, screenshots, long command transcripts and superseded exact-head narratives go to a linked immutable file under `docs/evidence/` or `docs/traceability/`. The regression in `tests/test_product_gap_baseline_navigation_contract.py` fails if the buyer matrix is buried again, a volatile queue snapshot is presented as continuously live, stable Unicode publication is regressed to the superseded pre-release gate, or historical daily cuts are reintroduced here.
