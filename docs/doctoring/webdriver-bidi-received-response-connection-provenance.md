# WebDriver BiDi received-response connection provenance

Status: Draft implementation evidence for PR #255. This note is not an ADR and does not grant browser, policy, process, profile, or Agent authority.

## Problem

A WebDriver BiDi command id is local-end correlation data, not transport identity. Before this repair, `session.end` stored the private generation of connection A when sending command id `7`, but response parsing accepted a bare assembled text message. A valid response with id `7` read from a separately verified connection B could therefore consume A's outstanding command and inherit A's stored generation. Later teardown evidence could appear internally consistent even though the protocol acknowledgment actually arrived on B.

The test-first commit `0a94765e0c631a043febec0973fd043304877537` reproduces this with two real loopback TCP/WebSocket connections using the same WebDriver session id and command id. On the then-current implementation the foreign response was accepted and the A correlation was consumed. This is source-level RED lineage; no hosted RED is claimed because later repair commits superseded that exact generation before terminal hosted execution.

## Constraints and rejected alternatives

WebDriver BiDi uses the command `id` only so the local end can identify a command response; commands may finish out of order, and the id is opaque to the remote end. The 3 September 2026 W3C Working Draft's command-processing algorithms retain the WebSocket connection as an explicit input/output boundary. OriginWeave therefore needs local evidence of the receiving connection in addition to the response id.

Passing a caller-supplied connection generation into response parsing was rejected because it would turn provenance into forgeable metadata. Passing a bare assembled text message plus a separately supplied established connection was also rejected because callers could accidentally pair a message from B with connection A. Adding a connection id to JSON was rejected because it would invent protocol data not defined by WebDriver BiDi.

## Selected boundary

`WebDriverBiDiWebSocketMessageReader` consumes one established WebSocket and owns its message assembler. Every text fragment admitted by that reader is therefore read from the same non-cloneable verified transport. `Pending` and interleaved control outcomes retain the reader so fragmented state cannot be moved onto another connection. Once a text message is complete, the reader emits `WebDriverBiDiReceivedTextMessage` carrying the private process-local connection generation and returns the established transport separately because no partial fragments remain.

`session.end` response admission requires that received-message type. Correlation validates command kind, stored connection provenance, and received connection generation before removing the outstanding id. Missing provenance and a different received connection both fail closed without consuming the command. Protocol ACK remains distinct from transport closure, Chromium process exit, profile deletion, and task completion.

This placement also preserves RFC 6455 fragmentation semantics: a message may span multiple frames, control frames may appear between fragments, recipients must support fragmented and unfragmented messages, and message fragments are delivered in order on the WebSocket connection. The reader therefore binds the assembled-message transaction to one connection rather than treating individual application payloads as free-standing evidence.

## Quality-gate follow-up

A complete local verification snapshot at `e7bfec4488b7cb4776df7b546cacb46c8c9eb13e` established a second RED lane: the behavioral repair passed its focused connection-provenance tests, but rustfmt and strict Clippy were not clean and production coverage still missed connection-bound event/null-id errors, connection-generation exhaustion propagation, and a post-correlation provenance fallback that could not be reached after successful connection-bound correlation. Those failures are quality-gate evidence, not hosted GREEN.

The follow-up keeps the security invariant and removes the gate-specific causes rather than excluding them. `feba41f13fb39e83fc8b377b8a55bd7c27348f9d` exercises event and null-id envelopes through the real `session.end` response path without consuming the outstanding command. The teardown provenance comparison is expressed without the nested `if` rejected by strict Clippy. Successful connection-bound correlation now carries the already-validated received-message generation directly into `WebDriverBiDiSessionEndResult`, eliminating the impossible optional-provenance fallback instead of excluding it from coverage. Connection establishment accepts a private generation-counter seam so the exhaustion error can be exercised after exact peer verification, and the allocator uses `AtomicU64::try_update`; Rust documents `try_update` as available since 1.95.0, which remains compatible with OriginWeave's declared Rust 1.97 MSRV. The added exhaustion regression uses a real loopback TCP stream plus a verified peer result and requires `ConnectionGenerationExhausted` after one connect and one peer-inspection call.

Formatting-only test-name repairs preserve the realistic transport scenarios while restoring conventional rustfmt layout. Exact hosted CI/coverage/security results still belong to the live PR head and must be read from GitHub; predecessor results do not transfer.

The `63cbca0a98cf9496af981819d98029e656fc4342` follow-up removed the unused private correlated-response generation accessor, not the stored provenance or its validation. Rust 1.97.1 rustfmt, strict Clippy, workspace tests, rustdoc with `-D warnings`, and all 141 Python contracts then passed locally. The existing production coverage checker reported 1082/1082 functions, 11025/11025 lines, 14071/14071 regions, and 1202/1202 branches. However, pinned `cargo-llvm-cov 0.8.6` emitted `warning: --branch option is unstable and it may be changed in the future`. These numerical coverage results are not warning-free acceptance evidence. Keep the warning visible and the gate outstanding; do not suppress it, remove branch measurement, or change the denominator to manufacture a clean result.

## Evidence and remaining risk

The repair includes a realistic two-connection regression for a foreign `session.end` success response and focused reader coverage for fragmented text, an interleaved control frame, malformed server framing, binary-message rejection, event/null-id correlation, and connection-generation exhaustion after verified peer admission. The original inline response-substitution and closure-substitution findings are resolved by the current source boundary, but exact-head CI and security workflows remain authoritative before any integration claim.

The current scope deliberately does not infer browser-process ownership from the WebSocket connection and does not make protocol success an operational teardown post-condition. Process-exit and profile-removal evidence remain unavailable until their canonical runtime owners provide non-forgeable contracts.

## References

Fette, I., & Melnikov, A. (2011). *The WebSocket Protocol* (RFC 6455). Internet Engineering Task Force. https://www.rfc-editor.org/rfc/rfc6455

The Rust Project Developers. (n.d.). *AtomicU64 in std::sync::atomic*. Rust standard library documentation. Retrieved September 5, 2026, from https://doc.rust-lang.org/std/sync/atomic/type.AtomicU64.html

W3C Browser Testing and Tools Working Group. (2026, September 3). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
