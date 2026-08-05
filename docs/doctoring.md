# Research and Standards Doctoring

This document records external evidence that changes OriginWeave architecture, test design, or release criteria. References use APA 7th style. Draft specifications are explicitly identified as work in progress.

## Decision trace

### Browser automation and interoperability

WebDriver BiDi defines a bidirectional remote-control protocol and user contexts. Because the current document is a W3C Working Draft, OriginWeave will place BiDi behind a versioned adapter and contract tests rather than make it the internal authority model.

### Crawling policy

RFC 9309 standardizes robots parsing, matching, error handling, and caching. It also states that robots rules are not access authorization. OriginWeave therefore requires robots evidence for public crawler mode while maintaining authentication, terms, rate, privacy, and retention policy as separate controls.

### Provenance and capture

W3C PROV-O supplies interoperable Entity, Activity, Agent, derivation, attribution, and responsibility concepts. ISO 28500:2017 defines WARC storage for protocol payloads, control information, metadata, transformations, duplicate detection, integrity, and segmentation. OriginWeave uses source hashes and locators in the safety kernel, then plans separate WARC and PROV adapters.

### AI risk and prompt injection

NIST AI 600-1 provides generative-AI lifecycle risk guidance. WASP demonstrates that web-navigation agents can begin following low-effort indirect prompt injections even when end-to-end attacker success remains lower. OriginWeave therefore separates trusted instructions, untrusted observations, and protected secrets at the type and process boundaries rather than rely on prompting alone.

### Web-agent observation and evaluation

Mind2Web reports that raw real-world HTML is often too large for direct LLM use and that filtering improves effectiveness and efficiency. OriginWeave prioritizes typed tools, structured data, network responses, accessibility/DOM/layout semantics, and only then visual fallback. WebArena and related environments motivate repeatable task-success and failure-recovery benchmarks instead of anecdotal demos.

### Test-time compute orchestration

FUGU studies routing and optimization of test-time compute. Conductor studies cognitive orchestration for multi-agent collaboration. TRINITY studies reasoning while acting. OriginWeave will treat model routing, workflow stage, recursion depth, decomposition, access lists, and role-specific reasoning effort as explicit configuration and will require ablations before deeper orchestration is declared superior. These preprints guide experimentation; they do not override the deterministic browser policy engine.

## References

Autio, C., Schwartz, R., Dunietz, J., Jain, S., Stanley, M., Tabassi, E., Hall, P., & Roberts, K. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.AI.600-1

Deng, X., Gu, Y., Zheng, B., Chen, S., Stevens, S., Wang, B., Sun, H., & Su, Y. (2023). *Mind2Web: Towards a generalist agent for the web*. arXiv. https://doi.org/10.48550/arXiv.2306.06070

Evtimov, I., Zharmagambetov, A., Grattafiori, A., Guo, C., & Chaudhuri, K. (2025). *WASP: Benchmarking web agent security against prompt injection attacks*. arXiv. https://doi.org/10.48550/arXiv.2504.18575

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO Standard No. 28500:2017). https://www.iso.org/standard/68004.html

Koster, M., Illyes, G., Zeller, H., & Sassman, L. (2022). *Robots Exclusion Protocol* (RFC 9309). Internet Engineering Task Force. https://doi.org/10.17487/RFC9309

World Wide Web Consortium. (2013). *PROV-O: The PROV ontology*. https://www.w3.org/TR/prov-o/

World Wide Web Consortium. (2026, June 1). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/

Zhou, S., Xu, F. F., Zhu, H., Zhou, X., Lo, R., Sridhar, A., Cheng, X., Ou, T., Bisk, Y., Fried, D., Alon, U., & Neubig, G. (2023). *WebArena: A realistic web environment for building autonomous agents*. arXiv. https://doi.org/10.48550/arXiv.2307.13854

## Provisional orchestration references

The following recent preprints require bibliographic metadata revalidation before a release makes claims based on them:

- *FUGU: A framework for intelligent routing and optimization in test-time compute* (arXiv:2510.12841).
- *Conductor: Cognitive orchestration for multi-agent collaboration* (arXiv:2512.04388).
- *TRINITY: Teaching language models to reason while acting* (arXiv:2511.22611).
