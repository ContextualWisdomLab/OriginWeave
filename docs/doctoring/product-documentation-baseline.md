# Product Documentation Baseline — Research and Standards Doctoring

- **Reviewed:** 2026-08-09
- **Purpose:** Fresh primary-source evidence for the PRD/TRD/UML/ERD/data-governance documentation baseline
- **Relationship:** Supplements [`../doctoring.md`](../doctoring.md); it does not supersede the deeper network/TLS/research bibliography there.

This addendum exists because product-level documentation introduced explicit interoperability, accessibility, data-governance, and assurance-status claims that need a current evidence snapshot. The production design still requires feature-specific evidence in the governing ADR and main doctoring document.

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

## NIST zero-trust architecture and data access

NIST SP 800-207 states that zero trust grants no implicit trust based solely on physical/network location or asset ownership and focuses authorization on resources. NIST SP 800-207A extends the model to cloud-native and multi-location applications and explicitly emphasizes application/service identity plus granular application-level authorization policy.

**Product implications:**

- Sensitive-data disclosure is a resource-access decision, not an ambient consequence of being inside the network or browser session.
- Tenant, task, field, purpose, destination, classification, service identity, and current policy state are independent authorization dimensions where applicable.
- A trusted broker or browser adapter rechecks authority immediately before disclosure instead of treating a previously issued opaque handle as transferable bearer authority.
- Service-to-service paths must authenticate workload/service identity and prevent confused-deputy disclosure.

## KISA CSAP assurance boundary

KISA's current CSAP program description states that the certification applies to cloud computing services evaluated against the applicable cloud security certification criteria and that the certification mark is for services that have actually obtained certification.

**Product implication:** OriginWeave may design features and evidence for CSAP readiness, but source code, tests, or a control mapping cannot truthfully claim that a deployed service is CSAP certified. Certification claims remain bound to the assessed service boundary and current program requirements.

## AICPA Trust Services Criteria assurance boundary

The AICPA 2017 Trust Services Criteria with revised 2022 points of focus provide criteria for evaluating controls relevant to security, availability, processing integrity, confidentiality, and privacy in attestation or consulting engagements.

**Product implication:** OriginWeave uses the criteria as a control/evidence design input. Product capability, configured control, operating control, collected evidence, management assertion, and independent examination result remain distinct evidence classes. Documentation must not collapse them into a generic “SOC 2 compliant” claim.

## APA 7th references

American Institute of Certified Public Accountants. (2023). *2017 trust services criteria for security, availability, processing integrity, confidentiality, and privacy (with revised points of focus—2022)*. AICPA & CIMA. https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022

Chandramouli, R., & Butcher, Z. (2023). *A zero trust architecture model for access control in cloud-native applications in multi-location environments* (NIST Special Publication 800-207A). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207A

Chrome for Developers. (n.d.). *Extensions / Manifest V3*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

Korea Internet & Security Agency. (n.d.). *클라우드 보안인증제 제도소개*. Retrieved August 9, 2026, from https://isms.kisa.or.kr/main/csap/intro/index.jsp

National Institute of Standards and Technology. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). https://doi.org/10.6028/NIST.AI.600-1

Rose, S., Borchert, O., Mitchell, S., & Connelly, S. (2020). *Zero trust architecture* (NIST Special Publication 800-207). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.SP.800-207

World Wide Web Consortium. (2023). *Web Content Accessibility Guidelines (WCAG) 2.2*. https://www.w3.org/TR/WCAG22/

World Wide Web Consortium. (2025, October 21). *W3C Web Content Accessibility Guidelines 2.2 approved as ISO/IEC international standard*. https://www.w3.org/press-releases/2025/wcag22-iso-pas/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/
