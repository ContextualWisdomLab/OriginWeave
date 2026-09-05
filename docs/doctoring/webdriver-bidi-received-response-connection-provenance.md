# WebDriver BiDi received-response connection provenance

Status: Draft implementation evidence for PR #255. This note is not an ADR and does not grant browser, policy, process, profile, or Agent authority.

## Problem

A WebDriver BiDi command id is local-end correlation data, not transport identity. Before this repair, `session.end` stored the private generation of connection A when sending command id `7`, but response parsing accepted a bare assembled text message. A valid response with id `7` read from a separately verified connection B could therefore consume A's outstanding command and inherit A's stored generation. Later teardown evidence could appear internally consistent even though the protocol acknowledgment actually arrived on B.

The test-first commit `0a94765e0c631a043febec0973fd043304877537` reproduces this with two real loopback TCP/WebSocket connections using the same WebDriver session id and command id. On the then-current implementation the foreign response was accepted and the A correlation was consumed. This is source-level RED lineage; no hosted RED is claimed because later repair commits superseded that exact generation before terminal hosted execution.

## Constraints and rejected alternatives

WebDriver BiDi uses the command `id` only so the local end can identify a command response; commands may finish out of order, and the id is opaque to the remote end. The current specification's command-processing algorithm sends the resulting WebSocket message over the same `connection` on which the command was received. OriginWeave therefore needs local evidence of the receiving connection in addition to the response id.

Passing a caller-supplied connection generation into response parsing was rejected because it would turn provenance into forgeable metadata. Passing a bare assembled text message plus a separately supplied established connection was also rejected because callers could accidentally pair a message from B with connection A. Adding a connection id to JSON was rejected because it would invent protocol data not defined by WebDriver BiDi.

## Selected boundary

`WebDriverBiDiWebSocketMessageReader` consumes one established WebSocket and owns its message assembler. Every text fragment admitted by that reader is therefore read from the same non-cloneable verified transport. `Pending` and interleaved control outcomes retain the reader so fragmented state cannot be moved onto another connection. Once a text message is complete, the reader emits `WebDriverBiDiReceivedTextMessage` carrying the private process-local connection generation and returns the established transport separately because no partial fragments remain.

`session.end` response admission now requires that received-message type. Correlation validates command kind, stored connection provenance, and received connection generation before removing the outstanding id. Missing provenance and a different received connection both fail closed without consuming the command. Protocol ACK remains distinct from transport closure, Chromium process exit, profile deletion, and task completion.

This placement also preserves RFC 6455 fragmentation semantics: a message may span multiple frames, control frames may appear between fragments, recipients must support fragmented and unfragmented messages, and message fragments are delivered in order on the WebSocket connection. The reader therefore binds the assembled-message transaction to one connection rather than treating individual application payloads as free-standing evidence.

## Evidence and remaining risk

The repair adds a realistic two-connection regression for a foreign `session.end` success response and focused reader coverage for fragmented text, an interleaved control frame, malformed server framing, and binary-message rejection. Exact hosted CI/coverage/security results belong to the live PR head and must be read from GitHub; predecessor results do not transfer.

The current scope deliberately does not infer browser-process ownership from the WebSocket connection and does not make protocol success an operational teardown post-condition. Process-exit and profile-removal evidence remain unavailable until their canonical runtime owners provide non-forgeable contracts.

## References

Fette, I., & Melnikov, A. (2011). *The WebSocket Protocol* (RFC 6455). Internet Engineering Task Force. https://www.rfc-editor.org/rfc/rfc6455

W3C Browser Testing and Tools Working Group. (2026). *WebDriver BiDi*. World Wide Web Consortium. https://w3c.github.io/webdriver-bidi/
