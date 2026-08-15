# Research and Standards Doctoring

This document records external evidence that changes OriginWeave architecture, test design, or release criteria. References use APA 7th style. Draft specifications and preprints are explicitly identified as work in progress.

## Feature doctoring notes

- [VPN profile support: primary-source doctoring](doctoring/vpn-profile-support.md)

## Decision trace

### Browser automation and interoperability

The 1 June 2026 WebDriver BiDi Working Draft defines a bidirectional remote-control protocol, events, commands, and user contexts. Because it remains a W3C Working Draft, OriginWeave places BiDi behind a versioned adapter and Web Platform Tests-derived contract tests rather than make it the internal authority model.

### Browser origin equivalence

The WHATWG URL host parser and Chromium canonicalizer classify shortened decimal, integer, hexadecimal, legacy octal-looking, and mixed-component numeric hosts as IPv4 or broken IPv4 candidates rather than ordinary DNS names. Chromium's regression suite includes values such as `192`, `0xC0a80001`, `030052000001`, and mixed hexadecimal components. A non-final empty `0x` component can participate in Chromium's multi-part IPv4 truncation behavior, but a final `0x` label does not produce an IPv4 number because stripping its prefix leaves no digits; it remains a domain label. Chromium also warns that broken IP-like hosts must not be connected because another resolver could accept them. OriginWeave therefore admits only canonical dotted-decimal IPv4 into its policy origin type, rejects browser-special numeric spellings before DNS validation, and preserves final non-numeric DNS labels such as `0x`.

The exact Chromium regression evidence is pinned to revision `446d05d21720f0b3505ec21057b3e9f909784262`. A mutable `HEAD` reference is not sufficient for a reproducible security contract.

### Resolved destination and redirect safety

Canonical origin identity is not a network-destination authorization. The IANA IPv4 and IPv6 Special-Purpose Address Space registries enumerate blocks whose source, destination, forwardability, globally reachable, and protocol-reserved properties differ. Both registries were last updated on 9 October 2025 and explicitly warn that registry presence does not guarantee routability in a particular local or global context. RFC 6890 established the common special-purpose registry fields, and RFC 8190 replaced the ambiguous `global` field with `globally reachable`.

The separate IANA IPv6 Global Unicast Address Space registry was updated on 10 October 2025. It states that `2000::/3` is the assignable global-unicast block, but every part not listed in the allocation table remains reserved by IANA for future allocation. OriginWeave therefore does not equate membership in `2000::/3` with current public reachability. After special-purpose rules are applied, only the current IANA/RIR allocation prefixes are admitted as `Public`; gaps and the explicitly reserved `2d00::/8` through `3ffe::/16` ranges remain `ProtocolReserved`.

RFC 9637 reserves exactly `3fff::/20` as additional IPv6 documentation space and says that the prefix must not carry real traffic or be globally advertised. OriginWeave therefore classifies `3fff:0000::` through `3fff:0fff:ffff:ffff:ffff:ffff:ffff:ffff` as documentation while keeping adjacent `3fff:1000::/20` and the older unallocated `3ffe::/16` space fail closed.

OriginWeave uses a conservative, security-oriented taxonomy rather than treating resolver success or syntactic global scope as permission. Public web policy admits only addresses classified as public. Loopback, private or unique-local, shared, link-local, metadata, documentation, benchmarking, multicast, broadcast, unspecified, transition, unallocated, and protocol-reserved destinations require explicit managed authority or fail closed.

IANA registries alone do not enumerate every cloud- or workload-local credential and control endpoint that can appear inside otherwise public, shared, link-local, or unique-local address space. Microsoft documents `168.63.129.16` as an Azure platform virtual IP used for VM agent communication, filtered DNS, load-balancer health probes, DHCP, and platform heartbeat traffic. Amazon EKS documents `169.254.170.23` and `fd00:ec2::23` as the Pod Identity Agent endpoints through which workloads request credentials. OriginWeave classifies these endpoints, together with established instance and container metadata endpoints, as `MetadataService` before broader address-range rules. This list is a reviewed security supplement to the IANA taxonomy and must expand only from authoritative platform documentation and regression tests.

Rust 1.97.1 exposes stable address parsing and `Ipv6Addr::to_ipv4_mapped`, but the standard library's `Ipv4Addr::is_global` and `Ipv6Addr::is_global` remain nightly-only. The Rust documentation also notes that `::ffff:127.0.0.1` is not itself the IPv6 loopback address and must first be converted to canonical IPv4 before loopback classification. OriginWeave therefore maintains reviewed registry-bound classification tables in production Rust and canonicalizes IPv4-mapped IPv6 before policy or pin comparison rather than relying on unstable convenience predicates.

RFC 9110 models a redirect `Location` as a new target URI and describes reconstructing the request for that target, including removal or reconsideration of resource-specific fields such as `Authorization` and `Cookie`. OriginWeave accordingly treats every redirect as a new authorization boundary: target origin, target-bound resolution snapshot, secure-scheme downgrade, complete-target digest, and hop capacity are re-evaluated before state changes. A digest of the complete canonical target permits path- and query-sensitive cycle detection without retaining credential-bearing URIs in policy evidence.

The pure destination kernel performs no DNS lookup and opens no socket. It approves non-empty origin-bound address sets, authorizes only addresses in the pinned set, permits a refreshed DNS answer to contract to a non-empty subset, and rejects any newly introduced address as a possible rebinding event.

### Proxy server identity and route authority

Chromium models a proxy server as an address together with the proxy scheme used to communicate with it, rather than as the target web origin. Its proxy documentation defines DIRECT, HTTP, HTTPS, SOCKSv4, SOCKSv5, and QUIC schemes; the networked schemes use default ports 80, 443, 1080, 1080, and 443 respectively, and SOCKSv5 accepts both `socks://` and `socks5://` URI forms. Chromium also documents ordinary remote cleartext HTTP proxies such as `http://foo:8080`. OriginWeave therefore models `ProxyServer` separately from `Origin`: an HTTP proxy can be valid routing infrastructure even though an ordinary remote HTTP web origin remains forbidden by OriginWeave's web-origin policy.

Proxy-server identity remains only routing authority. Its scheme, canonical host, and effective port do not grant permission to resolve or connect to that proxy, authenticate to it, issue CONNECT, trust a TLS certificate, or reach the final target. Those remain separate destination, TCP peer, TLS identity, and application-policy boundaries. PAC source authority is likewise separate from the proxy server returned by PAC evaluation. The Chromium proxy evidence used for this contract is pinned to revision `a3e71ebfa307d8760eb68b777e2998a869940092` so later documentation edits cannot silently change the reviewed compatibility basis.

### Direct TCP peer binding

RFC 9293 consolidates the current Standards Track Transmission Control Protocol and identifies a connection through its endpoint sockets. This does not make a TCP connection equivalent to a web origin or a TLS identity, but it establishes the concrete remote IP address and port that must be compared with destination authority.

Rust 1.97.1 documents `TcpStream::connect_timeout` as attempting a connection to one supplied `SocketAddr`, with a timeout applied to that individual address. Unlike hostname-based connection APIs, this call does not give the adapter another collection of addresses to resolve or select. `TcpStream::peer_addr` reports the remote socket address of an established stream.

OriginWeave therefore creates a separate direct-only network kernel. A non-cloneable plan accepts one canonical `SocketAddr` already authorized by a `ResolutionSnapshot`, rejects port zero and unbounded timeouts or attempts, calls `connect_timeout` with that exact address, and checks `peer_addr` before exposing the stream. Requested and observed peers must match in both IP and port. IPv4-mapped IPv6 is rejected at this layer when the snapshot authorized its canonical IPv4 form.

This proof is deliberately narrower than safe browser navigation. It does not validate TLS server names, certificates, certificate chains, or ALPN; it does not authorize a proxy or PAC route; it does not parse HTTP or bound response resources; and it does not prove that Chromium's Network Service consumed the verified stream. Those remain separate merge-gated adapters. TCP peer equality is transport evidence, not application identity.

### TLS service identity

RFC 9846 is the current Standards Track TLS 1.3 specification and obsoletes RFC 8446. It defines a secure channel over a reliable, ordered byte stream and explicitly leaves application service-identity interpretation to the integrating protocol. It points application protocols to RFC 9525. RFC 9846 also reiterates that 0-RTT has weaker forward-secrecy and replay properties than ordinary 1-RTT application data. OriginWeave therefore cites RFC 9846 as the current TLS 1.3 authority, permits TLS 1.2 only for application interoperability, prefers TLS 1.3 through rustls ordering, and disables 0-RTT in the first slice.

RFC 9325 is the current Best Current Practice for secure TLS use. It requires implementations not to fall back from TLS 1.2 to older versions, recommends TLS 1.3 support and preference, and retains TLS 1.2 for application interoperability under its additional requirements. OriginWeave consequently configures only TLS 1.3 and TLS 1.2 and exposes no SSL, TLS 1.0, or TLS 1.1 path.

RFC 5280 defines the Internet PKIX certificate and CRL profile, including certificate validity, certification-path requirements, subject alternative names, key usages, and name constraints. A successful TCP connection or syntactically valid certificate is not sufficient; the entire certification path must validate against a configured trust anchor and trusted time. The first OriginWeave slice uses rustls WebPKI validation with an explicit immutable trust-root bundle and fixed `TimeProvider`. It does not acquire ambient operating-system roots or claim revocation validation when no OCSP or CRL evidence was configured.

RFC 9525 specifies service identity in TLS and obsoletes RFC 6125. DNS identity is checked only through `dNSName` subjectAltName; Common Name fallback is no longer valid. Literal IP identities use `iPAddress` subjectAltName. OriginWeave derives the reference identity only from the canonical HTTPS `Origin`, sends SNI only for DNS identity, and requires an exact IP SAN for IPv4 or IPv6 literals. A real loopback integration test includes a certificate whose Common Name says `localhost` but whose SAN names another host and proves that the connection is rejected.

The implementation pins rustls 0.23.42, the latest stable 0.23 release reviewed for this slice. Its default features are disabled and the explicit `ring`, `std`, and `tls12` features are enabled. Rustls supplies TLS 1.2 and TLS 1.3, WebPKI verification, explicit cryptographic providers, client/server connection state machines, ALPN, fixed-time providers, and safe defaults without obsolete protocol versions. OriginWeave still configures the policy explicitly: only TLS 1.3 and TLS 1.2, no session resumption, no early data, no secret extraction, `NoKeyLog`, no client certificate, no certificate compression, explicit roots, and a bounded ALPN allow-list.

The same `DirectTcpConnection` is consumed rather than replaced by a hostname-based convenience client. Its operating-system peer is checked before plan construction, before the handshake, during each handshake iteration, and after completion. Socket read and write timeouts are set to the remaining monotonic deadline and restored before the authenticated stream is exposed. This binds the service identity, exact peer, and elapsed-time evidence to one stream.

Credential-free TLS evidence records the canonical origin, TCP peers, reference identity, TLS version, cipher-suite identifier, explicit ALPN result, leaf certificate and SubjectPublicKeyInfo hashes, hashes and bounds for the server-presented certificates, explicit trust-bundle identity and hash, trusted verification time, leaf validity interval, revocation configuration, and handshake duration. Rustls exposes the peer-presented certificates but not a reconstructed internal certification path, so OriginWeave does not mislabel the hashes as a complete validated-path export.

The test-only rcgen 0.14.8 dependency creates a local CA and deterministic certificate-policy scenarios. It is not part of production arithmetic or trust. Tests cover trusted DNS identity, Common Name non-fallback, wrong name, untrusted root, expired and not-yet-valid validity, exact IPv4 and IPv6 SAN identity, TLS 1.2 and TLS 1.3, required and optional ALPN, and equality between TLS origin and TCP authority.

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

These results motivate explicit OriginWeave configuration for model routing, workflow stage, decomposition, recursion depth, permitted access, role assignment, and role-specific reasoning effort. They do not justify always using multiple agents. OriginWeave must compare bounded single-model, routed-model, and deeper multi-agent configurations through task-success, safety, variance, token, and compute ablations. No learned coordinator may expand browser capabilities, origins, destinations, approvals, secrets, or deterministic policy.

## References

Amazon Web Services. (n.d.). *Set up the Amazon EKS Pod Identity Agent*. Retrieved August 6, 2026, from https://docs.aws.amazon.com/eks/latest/userguide/pod-id-agent-setup.html

Autio, C., Schwartz, R., Dunietz, J., Jain, S., Stanley, M., Tabassi, E., Hall, P., & Roberts, K. (2024). *Artificial intelligence risk management framework: Generative artificial intelligence profile* (NIST AI 600-1). National Institute of Standards and Technology. https://doi.org/10.6028/NIST.AI.600-1

Bonica, R., Cotton, M., Haberman, B., & Vegoda, L. (2017). *Updates to the special-purpose IP address registries* (RFC 8190). Internet Engineering Task Force. https://doi.org/10.17487/RFC8190

Chromium Authors. (n.d.). *Proxy support in Chrome* [Source documentation]. Chromium. https://chromium.googlesource.com/chromium/src/+/a3e71ebfa307d8760eb68b777e2998a869940092/net/docs/proxy.md

Chromium Authors. (2026). *URL canonicalizer unit tests* [Source code]. Chromium. https://chromium.googlesource.com/chromium/src/+/446d05d21720f0b3505ec21057b3e9f909784262/url/url_canon_unittest.cc

Cooper, D., Santesson, S., Farrell, S., Boeyen, S., Housley, R., & Polk, W. (2008). *Internet X.509 public key infrastructure certificate and certificate revocation list (CRL) profile* (RFC 5280). Internet Engineering Task Force. https://doi.org/10.17487/RFC5280

Cotton, M., Vegoda, L., Bonica, R., & Haberman, B. (2013). *Special-purpose IP address registries* (RFC 6890). Internet Engineering Task Force. https://doi.org/10.17487/RFC6890

Deng, X., Gu, Y., Zheng, B., Chen, S., Stevens, S., Wang, B., Sun, H., & Su, Y. (2023). *Mind2Web: Towards a generalist agent for the web*. arXiv. https://doi.org/10.48550/arXiv.2306.06070

Eddy, W. M. (Ed.). (2022). *Transmission Control Protocol (TCP)* (RFC 9293). Internet Engineering Task Force. https://doi.org/10.17487/RFC9293

Evtimov, I., Zharmagambetov, A., Grattafiori, A., Guo, C., & Chaudhuri, K. (2025). *WASP: Benchmarking web agent security against prompt injection attacks*. arXiv. https://doi.org/10.48550/arXiv.2504.18575

Fielding, R., Nottingham, M., & Reschke, J. (2022). *HTTP semantics* (RFC 9110). Internet Engineering Task Force. https://doi.org/10.17487/RFC9110

Fugu Team, Sakana AI. (2026). *Sakana Fugu technical report* [Technical report]. arXiv. https://doi.org/10.48550/arXiv.2606.21228

Huston, G., & Buraglio, N. (2024). *Expanding the IPv6 documentation space* (RFC 9637). Internet Engineering Task Force. https://doi.org/10.17487/RFC9637

Internet Assigned Numbers Authority. (2025, October 9). *IPv4 special-purpose address space*. https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml

Internet Assigned Numbers Authority. (2025, October 9). *IPv6 special-purpose address space*. https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml

Internet Assigned Numbers Authority. (2025, October 10). *IPv6 global unicast address space*. https://www.iana.org/assignments/ipv6-unicast-address-assignments/ipv6-unicast-address-assignments.xhtml

International Organization for Standardization. (2017). *Information and documentation—WARC file format* (ISO Standard No. 28500:2017). https://www.iso.org/standard/68004.html

Koster, M., Illyes, G., Zeller, H., & Sassman, L. (2022). *Robots Exclusion Protocol* (RFC 9309). Internet Engineering Task Force. https://doi.org/10.17487/RFC9309

Microsoft. (2025, July 25). *Azure IP address 168.63.129.16 overview*. Microsoft Learn. https://learn.microsoft.com/azure/virtual-network/what-is-ip-address-168-63-129-16

Nielsen, S., Cetin, E., Schwendeman, P., Sun, Q., Xu, J., & Tang, Y. (2025). *Learning to orchestrate agents in natural language with the Conductor* [Preprint]. arXiv. https://doi.org/10.48550/arXiv.2512.04388

Rescorla, E. (2026). *The Transport Layer Security (TLS) protocol version 1.3* (RFC 9846). Internet Engineering Task Force. https://doi.org/10.17487/RFC9846

Rustls Project Developers. (2026). *rustls 0.23.42* [Computer software]. https://docs.rs/rustls/0.23.42/rustls/

Saint-Andre, P., & Salz, R. (2023). *Service identity in TLS* (RFC 9525). Internet Engineering Task Force. https://doi.org/10.17487/RFC9525

Sakana AI. (2026, April 24). *Sakana Fugu: A multi-agent orchestration system as a foundation model*. https://sakana.ai/fugu-beta/

Sakana AI. (2026, June 22). *Sakana Fugu: One model to command them all*. https://sakana.ai/fugu-release/

Sheffer, Y., Saint-Andre, P., & Fossati, T. (2022). *Recommendations for secure use of Transport Layer Security (TLS) and Datagram Transport Layer Security (DTLS)* (RFC 9325). Internet Engineering Task Force. https://doi.org/10.17487/RFC9325

The Rust Project Developers. (2026). *Ipv4Addr in std::net* (Rust 1.97.1) [Software documentation]. https://doc.rust-lang.org/stable/std/net/struct.Ipv4Addr.html

The Rust Project Developers. (2026). *Ipv6Addr in std::net* (Rust 1.97.1) [Software documentation]. https://doc.rust-lang.org/stable/std/net/struct.Ipv6Addr.html

The Rust Project Developers. (2026). *TcpStream in std::net* (Rust 1.97.1) [Software documentation]. https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html

Web Hypertext Application Technology Working Group. (2026). *URL standard*. https://url.spec.whatwg.org/

World Wide Web Consortium. (2013). *PROV-O: The PROV ontology*. https://www.w3.org/TR/prov-o/

World Wide Web Consortium. (2026, June 1). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260601/

Xu, J., Sun, Q., Schwendeman, P., Nielsen, S., Cetin, E., & Tang, Y. (2025). *TRINITY: An evolved LLM coordinator* [Preprint]. arXiv. https://doi.org/10.48550/arXiv.2512.04695

Zhou, S., Xu, F. F., Zhu, H., Zhou, X., Lo, R., Sridhar, A., Cheng, X., Ou, T., Bisk, Y., Fried, D., Alon, U., & Neubig, G. (2023). *WebArena: A realistic web environment for building autonomous agents*. arXiv. https://doi.org/10.48550/arXiv.2307.13854
