# Product Documentation Baseline — Research and Standards Doctoring

- **Reviewed:** 2026-08-09
- **Purpose:** Fresh primary-source evidence for the PRD/TRD/UML/ERD documentation baseline
- **Relationship:** Supplements [`../doctoring.md`](../doctoring.md); it does not supersede the deeper network/TLS/research bibliography there.

This addendum exists because product-level documentation introduced explicit interoperability and accessibility status claims that need a current evidence snapshot. The production design still requires feature-specific evidence in the governing ADR and main doctoring document.

## WebDriver BiDi

The W3C latest published WebDriver BiDi document reviewed for this baseline is a **Working Draft dated 29 June 2026**. It defines a bidirectional browser automation protocol and links to an implementation report and Web Platform Tests. Because it is still a Working Draft, OriginWeave keeps it behind a versioned adapter and conformance tests instead of making its object identifiers or protocol semantics the core authority model.

**Product implication:** `OriginWeave Protocol` remains the internal stable boundary; WebDriver BiDi is a replaceable/versioned adapter.

## Chrome DevTools Protocol and WebMCP

The Chrome DevTools Protocol tip-of-tree documentation states that the latest protocol changes frequently, may break at any time, and does not guarantee backwards compatibility for new capabilities. The reviewed tip-of-tree snapshot was updated on 8 August 2026.

The same tip-of-tree documentation exposes a **WebMCP** domain and marks it **Experimental**. The domain supports tool registration/invocation lifecycle events and explicitly warns that tool output is untrusted and may contain prompt-injection content; its annotations include `readOnly` and `untrustedContent` hints.

**Product implications:**

- CDP-specific capability remains behind a versioned Chromium adapter.
- WebMCP is a preferred typed observation/tool channel when available, not the sole observation path.
- WebMCP tool descriptions, inputs and outputs are page-originated untrusted data; annotations cannot create OriginWeave capabilities or approval.
- A tip-of-tree CDP feature cannot become a release requirement without a supported-version matrix and fallback/compatibility evidence.

## Chrome Manifest V3

Chrome's current extension documentation identifies Manifest V3 as the current extension platform baseline. The migration documentation also notes that Manifest V3 removes remotely hosted code so extension JavaScript is packaged and reviewable with the extension.

**Product implication:** OriginWeave preserves Chromium's Manifest V3 implementation and tests compatibility rather than creating a separate default Rust plugin ecosystem that would require reimplementing Chrome extension APIs. OriginWeave agent authority remains a separate signed policy grant rather than an implicit extension permission.

## WCAG 2.2 and ISO/IEC 40500:2025

WCAG 2.2 is a W3C Recommendation. W3C announced on 21 October 2025 that WCAG 2.2 was approved as **ISO/IEC 40500:2025**. W3C notes that the ISO publication corresponds to the October 2023 WCAG 2.2 version while W3C continues to maintain later errata/updates.

**Product implication:** OriginWeave product UI uses WCAG 2.2 AA as its accessibility design target and tracks current W3C errata. References to ISO/IEC 40500:2025 describe the international-standard alignment; they do not constitute product certification or conformance by themselves.

## NIST AI 600-1

NIST AI 600-1, *Artificial Intelligence Risk Management Framework: Generative Artificial Intelligence Profile*, remains the primary NIST generative-AI profile used by this product baseline.

**Product implication:** OriginWeave uses NIST AI 600-1 as risk-management input for model/provider, untrusted-content, evaluation, provenance and lifecycle controls. It is not a certification claim and cannot substitute for deterministic browser authority or application-specific threat analysis.

## APA 7th references

Chrome for Developers. (n.d.). *Extensions / Manifest V3*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

National Institute of Standards and Technology. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). https://doi.org/10.6028/NIST.AI.600-1

World Wide Web Consortium. (2023). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

World Wide Web Consortium. (2025, October 21). *W3C Web Content Accessibility Guidelines 2.2 approved as ISO/IEC international standard*. https://www.w3.org/press-releases/2025/wcag22-iso-pas/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/
