# Controlled benchmark Unicode identity security

Status: active successor work for #323, stacked on #322 and #237. This record does not describe protected-main shipment. Requirement-to-code/test evidence is indexed in [`../traceability/controlled-benchmark-unicode-identity.md`](../traceability/controlled-benchmark-unicode-identity.md). Unicode 18.0.0 formal-release provenance reconciliation is tracked by #325.

## Problem

`ControlledBenchmarkRunContext` carries OriginWeave-owned reproducibility identities such as source revision, Chromium revision, OS image, hardware profile, protocol-adapter set, model/provider route, reasoning configuration, fixture/corpus version, and seed-set identity. These strings are compared byte-for-byte and are expected to appear in logs, reports, retained benchmark evidence, and later signed evidence summaries.

#322 already rejects C0/C1 controls, `U+2028` LINE SEPARATOR, `U+2029` PARAGRAPH SEPARATOR, and the exact `Bidi_Control` set. That closes reviewed line/paragraph and directional-rendering cases, but not the broader Unicode default-ignorable set. For example, ZERO WIDTH SPACE, ZWNJ, ZWJ, WORD JOINER, ZERO WIDTH NO-BREAK SPACE, and variation selectors can remain byte-distinct while being invisible or presentation-dependent. The exact #322 predecessor therefore admitted values that can be difficult to distinguish in an audit or signed evidence summary.

Unicode Standard Annex #31 Revision 45 defines the Default-Ignorable Exclusion Profile as excluding every code point whose `Default_Ignorable_Code_Point` property is true. Unicode Technical Standard #39 Revision 34 classifies `Default_Ignorable` characters as Restricted in its General Security Profile; ZWJ and ZWNJ are included unless an implementation explicitly adopts and documents a tailored profile. Both publications are stable and approved for Unicode 18.0.0.

The property table implemented by #324 originated from `DerivedCoreProperties-18.0.0.txt` dated 2026-08-07. Before formal publication, the nominal versioned URL still resolved through `/Public/draft/`, so #324 correctly treated that observation as pre-release evidence. Fresh 2026-09-20 KST revalidation establishes a later publication state: the Unicode Consortium release registry lists Unicode 18.0.0 as released on 2026-09-16, the release announcement states that Version 18.0 is now available, the Version 18.0.0 page supplies the formal citation and says the UCD data files are available, and the versioned `DerivedCoreProperties.txt` path is directly available. The Version 18.0.0 summary page still contains a contradictory preliminary-draft banner; #325 records that as a publication-site inconsistency rather than allowing it to override the Consortium's release registry, announcement, formal version citation, and versioned UCD artifact.

This rule is intentionally narrower than a product-wide Unicode policy. OriginWeave must preserve browser-issued protocol identity such as WebDriver BiDi `browser.UserContext` losslessly because Browser Session does not own that external identifier grammar. The controlled benchmark does own its reproducibility-label grammar, so a fail-closed identifier profile is appropriate only at this boundary.

## Decision

Adopt an explicit benchmark evidence-identity profile named `unicode-18.0.0-default-ignorable-exclusion`. The current implementation rejects all 4,174 DICP scalars in the versioned Unicode 18.0.0 `DerivedCoreProperties.txt` property set with no tailored exceptions. It continues to reject the existing C0/C1 and `U+2028`/`U+2029` rendering controls. Ordinary visible Korean, Japanese, Chinese, Vietnamese, Spanish, German, French, Arabic, and Hebrew remain admissible when they do not contain an excluded scalar.

The implementation pins that property table locally rather than relying on a moving Unicode dependency. Fresh direct comparison of the formally released versioned UCD artifact finds 27 DICP source entries / 4,174 scalars. Those entries compress exactly to the implementation's 17 ranges/singletons: `U+00AD`, `U+034F`, `U+061C`, `U+115F..1160`, `U+17B4..17B5`, `U+180B..180F`, `U+200B..200F`, `U+202A..202E`, `U+2060..206F`, `U+3164`, `U+FE00..FE0F`, `U+FEFF`, `U+FFA0`, `U+FFF0..FFF8`, `U+1BCA0..1BCA3`, `U+1D173..1D17A`, and `U+E0000..E0FFF`. The compressed `180B..180F`, `2060..206F`, and `E0000..E0FFF` ranges are exact unions of adjacent source entries with no gaps and therefore do not widen the published property set.

No normalization, case folding, confusable skeletonization, mixed-script restriction, or ASCII-only admission is introduced. Byte identity remains authoritative after admission. The existing public `ControlCharacterRunContext` variant is retained for compatibility even though its diagnostic now describes the broader rendering/default-ignorable boundary; changing the public variant name would create unrelated API churn without changing the fail-closed result.

Validation still runs independently on expected and observed context values before equality can influence benchmark evidence. No benchmark threshold, registry membership, browser authority, model/provider routing, evidence signing/persistence, workflow, or release authority changes in this slice.

## Test-first evidence

Parent #322 exact: `a4c8ceaf67a075ef483334802aacfc54cf502068`.

Test-first exact: `592a1a3bc49a922df86285eab332308770b77474`. It adds hostile cases for `U+200B`, `U+200C`, `U+200D`, `U+2060`, `U+FEFF`, and `U+FE0F` and requires the existing typed fail-closed error. The predecessor validator accepted each of those values because they are outside its C0/C1, line-separator, and `Bidi_Control` predicates. Hosted Rust execution was not available when the RED was authored, so this is a source-reproduced semantic RED, not a claim that a GitHub runner executed the failure.

Production repair exact: `19a37a667baeffe824c4fb025942eecb40bc67c8`. It introduces the versioned profile constant and the pinned DICP predicate, then applies it before expected/observed byte equality.

Coverage successor exact: `02edb5b2cb5ff9e1237afbc83d9a0f3e7aa48d6a`. It covers the profile identifier and endpoints or representatives of every compressed property range while retaining visible multilingual and ordinary RTL acceptance cases.

These commits are active branch evidence only. Exact-head repository contracts, rustfmt, locked tests, strict Clippy, rustdoc/API docs, production function/line/region/branch coverage, applicable browser/security workflows, and #325 immutable-receipt closure still need to complete before promotion.

## Publication provenance gate

Formal-release finality and property-set equivalence are now satisfied at the standards/data level. On 2026-09-20 KST, the official Unicode release registry identifies Unicode 18.0.0 as the 2026-09-16 release; the official release announcement says Version 18.0 is available; the formal Version 18.0.0 page states that UCD data files are available; and the direct versioned `DerivedCoreProperties.txt` artifact exposes the same 27-entry / 4,174-scalar DICP set already pinned by #324.

The property comparison has its own reproducible receipt. Expanding the 27 source entries to the ascending scalar set and encoding each scalar as six uppercase hexadecimal digits followed by LF (`%06X\n`) produces a 29,218-byte normalized stream with SHA-256 `673264e62183e35f6055a2ad4940403e706669e0750fcc5d56a99f158fb3bb93`. Expanding #324's 17 compressed implementation ranges produces the same stream and digest. This digest proves property-set equivalence; it is deliberately separate from the raw `DerivedCoreProperties.txt` file hash.

#325 therefore no longer waits for the stale preliminary banner to disappear and no longer lacks a normalized property digest. Its remaining provenance work is the official versioned artifact's raw byte length and full-file SHA-256. If that raw-file receipt confirms the same artifact, the dossier may close provenance without changing admission semantics. If the artifact unexpectedly resolves to different content or a later normalized property comparison differs, OriginWeave must not silently rewrite this profile: a fresh compatibility/security RED and a new profile identity are required.

## Alternatives considered

Keeping #322's finite reviewed list was rejected because it leaves other byte-distinct invisible or presentation-dependent DICP values admissible and would make the policy depend on recurring manual discovery. A moving Unicode library predicate was rejected because DICP has no unconditional stability guarantee; a dependency or toolchain update could silently change the accepted evidence grammar. Tailoring ZWJ/ZWNJ or emoji variation selectors was rejected for this benchmark-owned machine/audit identity because these fields are not natural-language prose or emoji labels and there is no buyer requirement that justifies the extra ambiguity.

Rejecting all non-ASCII text was rejected because it would conflate script diversity with evidence-rendering risk. NFC/NFKC, case folding, and confusable detection were rejected from this slice because they change comparison semantics rather than merely excluding the explicitly adopted property and require their own migration analysis.

Treating the 2026-08-07 file as final before formal publication was rejected and remains correct for that historical checkpoint. Treating the lingering preliminary banner as stronger than the later official release registry, release announcement, formal version citation, and directly published versioned UCD artifact is now also rejected; doing so would preserve a false release gate after the Consortium has formally published 18.0.0.

## Migration, reversal, and risk

Every retained benchmark result is already bound to an OriginWeave source revision. The new profile adds an explicit profile identifier so future evidence schemas can record the grammar directly rather than infer it from source history. Until that binding is wired into durable evidence by its canonical owner, source revision remains the compatibility anchor.

A future Unicode-version upgrade or tailored exception must not silently edit the current profile. It requires a new profile identifier, a fresh property/data comparison, hostile and compatibility tests, doctoring and changelog changes, and an explicit decision about whether old evidence remains acceptable. Reversal to a less restrictive profile has the same migration requirement because previously rejected identities could become admissible.

This exclusion reduces audit/display ambiguity but does not establish complete Unicode spoofing resistance. Confusables, mixed-script policy, normalization, and natural-language join-control usability remain separate threat and compatibility decisions. Durable evidence signing and provenance remain owned by their existing canonical boundaries.

## References

Unicode Consortium. (2026, September 16). *Unicode recent releases*. https://www.unicode.org/releases/

Unicode Consortium. (2026, September 16). *Announcing the Unicode Standard, Version 18.0*. https://blog.unicode.org/2026/09/announcing-unicode-standard-version-180.html

Unicode Consortium. (2026). *The Unicode Standard, Version 18.0.0*. https://www.unicode.org/versions/Unicode18.0.0/

Unicode Consortium. (2026, September 1). *Unicode identifiers and syntax* (Unicode Standard Annex #31, Version 18.0.0, Revision 45). https://www.unicode.org/reports/tr31/tr31-45.html

Unicode Consortium. (2026, August 27). *Unicode security mechanisms* (Unicode Technical Standard #39, Version 18.0.0, Revision 34). https://www.unicode.org/reports/tr39/tr39-34.html

Unicode Consortium. (2026, August 7). *DerivedCoreProperties-18.0.0.txt* [Unicode 18.0.0 Unicode Character Database artifact]. https://www.unicode.org/Public/18.0.0/ucd/DerivedCoreProperties.txt
