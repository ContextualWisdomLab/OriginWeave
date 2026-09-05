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

The `ebac126d1632c94775c2454423575275eec45def` follow-up removed the unused private correlated-response generation accessor, not the stored provenance or its validation. Rust 1.97.1 rustfmt, strict Clippy, workspace tests, rustdoc with `-D warnings`, and all 141 Python contracts then passed locally. The existing production coverage checker reported 1082/1082 functions, 11025/11025 lines, 14071/14071 regions, and 1202/1202 branches. However, pinned `cargo-llvm-cov 0.8.6` emitted `warning: --branch option is unstable and it may be changed in the future`. These numerical coverage results are not warning-free acceptance evidence. Keep the warning visible and the gate outstanding; do not suppress it, remove branch measurement, or change the denominator to manufacture a clean result.

## Current-parent integration

The ordinary integration of closure parent `b11b6c9bccd8335b58a4fb599f8ad29ac419637f` retains the complete `ebac126d...` connection-provenance repair. The parent restores the previously uncollected command-correlation release test and keeps peer fixtures alive until opening-exchange assertions finish. Native discovery first reported zero inherited release tests and failed the expected-one assertion; the integrated tree collects and passes that test. Sender registration, connection-bound message assembly, response admission before correlation consumption, closure-generation comparison and exhaustion handling are preserved without replacing their production or child-test blobs.

The earlier quality paragraph now identifies `ebac126d...`, the actual commit that removed the unused accessor. Its predecessor `63cbca0...` still contained that accessor and unformatted statements; the later repair's passing results cannot be assigned backward. Fresh verification of the integrated tree remains separate from those predecessor measurements, queued hosted checks, warning-free instrumentation and operational process/profile cleanup evidence.

Fresh integration verification executes 13 focused received-message, response and teardown tests, all 142 Python contracts, compileall, and the complete Rust 1.97.1 format/check/workspace-test/strict-Clippy/rustdoc gates. Pinned-nightly numerical coverage is 1082/1082 functions, 11032/11032 lines, 14086/14086 regions and 1202/1202 branches; the branch-instrumentation warning remains visible. These exact-tree local results neither authenticate a Chromium process nor establish hosted security acceptance, counted review approval or complete operational teardown.

## Evidence and remaining risk

### Pointer-click child integration

PR #256 predecessor `9f2e6f29be46371762e3031a97c1cac04720694f` lacked the current connection-provenance implementation and collected zero command-correlation release tests under native discovery. The expected-one assertion failed before ordinary integration of parent `3e7057443d7c9532ff526acb5eefe8cd4778c767`; the inherited contract then collected and passed. Its core command implementation, exports and four pointer-click tests are byte-identical to the predecessor. The sole child delta in the parent's correlation module remains the documented `PointerClick` command kind. Serialization does not prove a browser click, authorize input, or establish an observed page post-condition.

Fresh child verification passed all four pointer-click command tests, 13 received-message/response/teardown tests, 142 Python contracts, compileall, and the complete Rust 1.97.1 format/check/workspace-test/strict-Clippy/rustdoc gates. Pinned coverage measured 1088 functions, 11077 lines, 14146 regions and 1210 branches at 100%, with the unstable branch-option warning retained. These are local integrated-tree results; hosted checks, independent approval where required and protected-main delivery remain unproven.

The repair includes a realistic two-connection regression for a foreign `session.end` success response and focused reader coverage for fragmented text, an interleaved control frame, malformed server framing, binary-message rejection, event/null-id correlation, and connection-generation exhaustion after verified peer admission. The original inline response-substitution and closure-substitution findings are resolved by the current source boundary, but exact-head CI and security workflows remain authoritative before any integration claim.

The current scope deliberately does not infer browser-process ownership from the WebSocket connection and does not make protocol success an operational teardown post-condition. Process-exit and profile-removal evidence remain unavailable until their canonical runtime owners provide non-forgeable contracts.

### Pointer-click transport child integration

PR #257 predecessor `ea2b5b78868917219c46f1304558b92490a7f6fe` collected zero inherited command-correlation release tests and failed the expected-one assertion. Ordinary adoption of `ced4a851ca66c08d895a098725c7f0ad3ecf0c38` restores that executable contract and the current connection-provenance implementation. The module/export conflict is resolved by retaining both the typed click sender and connection-bound message reader. The sender and its two child test files remain byte-identical to the predecessor; no generic JSON dispatch, input authorization, response admission or click post-condition is added.

Fresh verification passed five pointer-click transport tests, five foreign-response/teardown tests, all 142 Python contracts, compileall and the complete Rust 1.97.1 quality gates. Pinned coverage measured 1093 functions, 11131 lines, 14202 regions and 1214 branches at 100%; the unstable branch-option warning remains. No focused retry was needed in this run. These results neither remove a platform-level socket-observation race in an unchanged fixture nor establish hosted acceptance, browser ownership or an observed click post-condition.

### Pointer-click response child integration

PR #258 predecessor `f2ceabb3ea50b1959e936503c50cae12f3e6e480` failed the expected-one native release-contract assertion because discovery collected zero tests. Ordinary adoption of current #257 `f4a8f2cbf515bea348f500b59615a9581c1b96a2` restores the inherited executable contract and current provenance prerequisites. The response implementation, its four loopback tests and the child-owned bounded socket-observation adjustment remain byte-identical to the predecessor. The crate documentation retains both the parent's connection-bound teardown description and this child's typed protocol-response boundary; no browser post-condition, generic dispatch or new response authority is introduced.

Fresh integrated-tree verification passed nine click response/send tests, five foreign-response/teardown tests, 142 Python contracts, compileall and the complete Rust 1.97.1 quality gates. Pinned coverage measured 1100 functions, 11185 lines, 14272 regions and 1214 branches at 100%, with the unstable branch-option warning retained. The current typed click response is protocol correlation only; the inherited `session.end` received-connection proof does not automatically authenticate this separate response API or prove browser navigation.

## References

Fette, I., & Melnikov, A. (2011). *The WebSocket Protocol* (RFC 6455). Internet Engineering Task Force. https://www.rfc-editor.org/rfc/rfc6455

The Rust Project Developers. (n.d.). *AtomicU64 in std::sync::atomic*. Rust standard library documentation. Retrieved September 5, 2026, from https://doc.rust-lang.org/std/sync/atomic/type.AtomicU64.html

W3C Browser Testing and Tools Working Group. (2026, September 3). *WebDriver BiDi* (W3C Working Draft). World Wide Web Consortium. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
