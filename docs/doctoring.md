# Research and Standards Doctoring

This document records external evidence that changes OriginWeave architecture, test design, or release criteria. References use APA 7th style. Draft specifications and preprints are explicitly identified as work in progress.

## Decision trace

### Browser automation and interoperability

The 1 June 2026 WebDriver BiDi Working Draft defines a bidirectional remote-control protocol, events, commands, and user contexts. Because it remains a W3C Working Draft, OriginWeave places BiDi behind a versioned adapter and Web Platform Tests-derived contract tests rather than make it the internal authority model.

### Browser origin equivalence

The WHATWG URL host parser and Chromium canonicalizer classify shortened decimal, integer, hexadecimal, legacy octal-looking, and mixed-component numeric hosts as IPv4 or broken IPv4 candidates rather than ordinary DNS names. Chromium's regression suite includes values such as `192`, `0xC0a80001`, `030052000001`, and mixed hexadecimal components. A non-final empty `0x` component can participate in Chromium's multi-part IPv4 truncation behavior, but a final `0x` label does not produce an IPv4 number because stripping its prefix leaves no digits; it remains a domain label. Chromium also warns that broken IP-like hosts must not be connected because another resolver could accept them. OriginWeave therefore admits only canonical dotted-decimal IPv4 into its policy origin type, rejects browser-special numeric spellings before DNS validation, and preserves final non-numeric DNS labels such as `0x`. Origin normalization is still not an SSRF boundary; resolved-address, rebinding, redirect, proxy, and metadata-endpoint policy remains a separate Chromium-adapter requirement.

### Crawling policy

RFC 9309 standardizes robots parsing, matching, error handling, and caching. It also states that robots rules are not access authorization. OriginWeave therefore requires robots evidence for public crawler mode while maintaining authentication, terms, rate, privacy, and retention policy as separate controls.

### Provenance and capture

W3C PROV-O supplies interoperable Entity, Activity, Agent, derivation, attribution, and responsibility concepts. ISO 28500:2017, confirmed in 2023, defines WARC storage for protocol payloads, control information, metadata, transformations, duplicate detection, integrity, and segmentation. OriginWeave uses source hashes and locators in the safety kernel, then adds WARC and PROV adapters as separately testable modules.

### AI risk and prompt injection

NIST AI 600-1 provides generative-AI lifecycle risk guidance. WASP demonstrates that web-navigation agents can follow low-effort indirect prompt injections. OriginWeave therefore separates trusted instructions, untrusted observations, and protected secrets at type and process boundaries rather than rely on prompting alone.

### Web-agent observation and evaluation

Mind2Web reports that raw real-world HTML is often too large for direct LLM use and that filtering improves effectiveness and efficiency. OriginWeave prioritizes typed tools, structured data, redacted network responses, accessibility/DOM/layout semantics, and only then visual fallback. WebArena motivates repeatable task-success and failure-recovery benchmarks instead of anecdotal demonstrations.

### Learned test-time orchestration

Sakana AI announced the Fugu early beta on 24 April 2026 and the broader Fugu and Fugu Ultra commercial release on 22 June 2026. The released service exposes a multi-agent orchestration system through one OpenAI-compatible model API. Fugu dynamically decides whether to solve directly or coordinate a deeper pool of expert models. Sakana AI identifies the ICLR 2026 TRINITY and Conductor papers as the methodological foundation and publishes a separate Fugu technical report. Product-page benchmark claims remain first-party commercial evidence; they are not treated as independent scientific replication.

TRINITY uses a compact learned coordinator to select models and assign Thinker, Worker, and Verifier roles over multiple turns. Conductor learns communication topologies and focused natural-language instructions and can form recursive coordination structures.

These results motivate explicit OriginWeave configuration for model routing, workflow stage, decomposition, recursion depth, permitted access, role assignment, and role-specific reasoning effort. They do not justify always using multiple agents. OriginWeave must compare bounded single-model, routed-model, and deeper multi-agent configurations through task-success, safety, variance, token, and compute ablations. No learned coordinator may expand browser capabilities, origins, approvals, secrets, or deterministic policy.

## References

Autio, C., Schwartz, R., Dunietz, J., Jain, S., Stanley, M., Tabassi, E., Hall, P., & Roberts, K. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.AI.600-1

Chromium Authors. (2026). *URL canonicalizer unit tests* [Source code]. Chromium. https://chromium.googlesource.com/chromium/src/+/HEAD/url/url_canon_unittest.cc

Deng, X., Gu, Y., Zheng, B., Chen, S., Stevens, S., Wang, B., Sun, H., & Su, Y. (2023). *Mind2Web: Towards a generalist agent for the web*. arXiv. https://doi.org/10.48550/arXiv.2306.06070

Evtimov, I., Zharmagambetov, A., Grattafiori, A., Guo, C., & Chaudhuri, K. (2025). *WASP: Benchmarking web agent security against prompt injection attacks*. arXiv. https://doi.org/10.48550/arXiv.2504.18575

Fugu Team, Sakana AI. (2026). *Sakana Fugu technical report* [Technical report]. arXiv. https://doi.org/10.48550/arXiv.2606.21228

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO Standard No. 28500:2017). https://www.iso.org/standard/68004.html

Koster, M., Illyes, G., Zeller, H., & Sassman, L. (2022). *Robots Exclusion Protocol* (RFC 9309). Internet Engineering Task Force. https://doi.org/10.17487/RFC9309

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2025). *Learning to orchestrate agents in natural language with the Conductor* [Preprint]. arXiv. https://doi.org/10.48550/arXiv.2512.04388

Sakana AI. (2026, April 24). *Sakana Fugu: A multi-agent orchestration system as a foundation model*. https://sakana.ai/fugu-beta/

Sakana AI. (2026, June 22). *Sakana Fugu: One model to command them all*. https://sakana.ai/fugu-release/

Web Hypertext Application Technology Working Group. (2026). *URL standard*. https://url.spec.whatwg.org/

World Wide Web Consortium. (2013). *PROV-O: The PROV ontology*. https://www.w3.org/TR/prov-o/

World Wide Web Consortium. (2026, June 1). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2025). *TRINITY: An evolved LLM coordinator* [Preprint]. arXiv. https://doi.org/10.48550/arXiv.2512.04695

Zhou, S., Xu, F. F., Zhu, H., Zhou, X., Lo, R., Sridhar, A., Cheng, X., Ou, T., Bisk, Y., Fried, D., Alon, U., & Neubig, G. (2023). *WebArena: A realistic web environment for building autonomous agents*. arXiv. https://doi.org/10.48550/arXiv.2307.13854
