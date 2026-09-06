# ADR 0107: Versioned browser and agent protocol adapters

- Status: Proposed
- Date: 2026-08-09
- Supersedes: none
- Superseded by: none

## Context

OriginWeave must interoperate with Chromium and external automation/agent ecosystems without allowing any one unstable protocol to define the product. WebDriver BiDi is standards-track but evolving, Chrome DevTools Protocol includes tip-of-tree surfaces without backwards-compatibility guarantees, WebMCP is experimental, and Model Context Protocol is an external tool/context protocol rather than browser authority. Directly exposing these surfaces as the OriginWeave API would couple customers to provider churn and blur policy boundaries.

## Decision drivers

- Stable OriginWeave semantics across browser/provider upgrades.
- Standards-first interoperability where mature enough.
- Ability to use Chromium-specific capabilities without making experimental CDP the sole authority.
- Explicit trust boundaries for WebMCP and MCP content/tools.
- Conformance and compatibility testing per adapter version.

## Assumptions and authority boundaries

The OriginWeave Protocol and Rust control plane own product semantics. WebDriver BiDi, Chrome DevTools Protocol, WebMCP, and Model Context Protocol are adapters or evidence/tool transports. Adapter messages are validated and cannot directly grant capabilities, approvals, secrets, origin authority, or evidence truth.

## Options considered

1. CDP as the public product API: rejected because Chromium-specific and unstable tip-of-tree surfaces create vendor/version lock-in.
2. WebDriver BiDi only: rejected because not every required Chromium/experimental capability is standardized yet.
3. Expose all upstream protocols directly: rejected because clients would inherit incompatible authority models.
4. Versioned internal protocol with explicit BiDi/CDP/WebMCP/MCP adapters: selected.

## Decision

OriginWeave exposes its own versioned protocol for session, observation, query, typed action, policy/evidence, secret-handle, resource, and lifecycle semantics. Browser and ecosystem adapters map that protocol to supported WebDriver BiDi, stable or pinned CDP, WebMCP, and MCP surfaces. Prefer standards-track BiDi when it satisfies the contract. Use Chromium-specific CDP only behind versioned adapter capability declarations. Treat WebMCP outputs as untrusted page/tool observations. Treat MCP as an external integration boundary, not a source of OriginWeave authority. Experimental/tip-of-tree surfaces are optional and must have fallback or explicit unsupported behavior.

MCP version negotiation is independent of the OriginWeave Protocol version. As of this review, MCP `2026-07-28` is the current released protocol generation; a future MCP change does not silently alter OriginWeave task, approval, secret, tenant, or browser semantics. MCP tool/resource content remains untrusted input and any server-to-client/user interaction capability is mediated by the same OriginWeave policy/approval boundaries as other adapter traffic.

## Consequences

### Proposed refinement: registry-to-transport session provenance (2026-09-06)

In the context of dispatching a registry-bound subscription on an established transport, facing
a valid context from session A being sent over session B, we decided for a read-only comparison
against the existing canonical external-session mapping and against a duplicate reverse registry
or registration from transport text, to reject mismatches before correlation and command bytes,
accepting one bounded lookup per subscription and rejection of adapters using unrelated session aliases.

This supplements original-registry and current-context checks rather than replacing them. The
existing connection-generation binding continues to govern replies and teardown. Unknown or retired
session mappings fail closed without creating state. Matching protocol text is not browser-process
authentication or navigation causality. The actual loopback RED at `b4702cd5` and its successor
check the no-correlation/no-wire boundary; status remains Proposed pending exact-head gates and
protected parent-first integration, not a release or architecture approval.

### Proposed refinement: consuming subscription teardown (2026-09-06)

In the context of ending a navigation subscription, facing a borrowed receipt that permits continued
event admission and teardown messages crossing connections, we decided for consuming the existing
non-cloneable receipt and retaining its connection identity through dispatch and acknowledgment,
and against shared revocation flags or matching session text alone, to close local admission before
teardown without duplicating lifecycle state, accepting that even construction or transport failure
requires a new subscription before local admission can resume.

This source proposal reuses the existing received-message wrapper and connection-aware correlation
owner. It adds no registry, dependency, reconnection or mutable revocation service. A different
connection is rejected before pending-command insertion or wire emission; a foreign success or error
cannot consume the original pending command. An unrelated outstanding command remains untouched.
The receipt and teardown command cannot be cloned. Previously admitted observations are not revoked,
and acknowledgment does not prove that buffered events have drained or that browser cleanup occurred.

The real-socket lifetime failure is retained in commit `2c45cea8`; its successor is the constructor's
`E0382` compile-fail example, because the repaired API makes the offending receipt reuse unrepresentable.
Commit `ce6f6fd4` records three separate real-socket transport failures. The existing admission-to-teardown
path and escaped-identifier round trip remain runtime checks. Shared flags would require extra checks
at every lifetime consumer while ownership already enforces this transition. Session or identifier
equality cannot distinguish two connections. Status remains Proposed, pending exact-head gates and
parent-first protected integration; this is not a browser-runtime acceptance or release decision.

### Proposed refinement: sealed command dispatch history (2026-09-06)

In the context of successive browser commands on one verified connection, facing retained or
buffered replies completing a later command with the same wire identifier, we decided for one
connection-owned strictly increasing typed-command namespace and mutually exclusive raw-text and
typed-command lanes, and against receive-order counters, consuming response wrappers alone, or a
resettable correlation-table ledger, to prevent ambiguity between local dispatches with constant
memory, accepting that callers must choose increasing IDs and use separate connections for raw text.

WebDriver BiDi permits identifier reuse; this is an OriginWeave local policy, not a protocol mandate.
The first typed ID may be zero and the JavaScript-safe maximum is usable once; exhaustion requires
an explicit new connection rather than wraparound. Pong frames and message reads preserve the mode
and last ID. All typed senders use the shared frame owner. Raw text cannot precede typed dispatch or
be inserted while typed responses are pending. The nonconsuming TCP stream borrow is removed because
a cloned handle could write outside that owner; the consuming raw-stream handoff remains available
but cannot reconstruct an upgradeable connection. No generic HTTP/TLS stream contract changes.

The real locally-revoked opening-write test moves into the connection owner's unit module so it can
retain its OS-socket failure checks without publishing a cloneable socket. Preflight failures still
retire only the newly registered command; ambiguous I/O failures retain pending correlation. A
bounded tombstone set would eventually force arbitrary eviction or reconnection, while an unbounded
set creates lifetime memory growth. Receive ordering cannot distinguish a buffered old reply first
read after resend, and consuming wrappers misses retirement before parsing. This proposal does not
authenticate the remote browser, reject invented peer replies, or prove navigation causality.
The adapter remains non-shipped, and this refinement remains Proposed pending governance and gates.

### Proposed refinement: original registry identity (2026-09-06)

In the context of a connection-bound navigation subscription whose local session/context numbers
can also exist in another registry, facing the risk that a genuine receipt or admitted event
changes unrelated document authority, we decided for a core-owned opaque registry witness retained
from command construction through event admission to the shared document-mutation boundary, and
against numeric/text equality, a constructor-only check, or globally renumbering every browser
identifier, to preserve exact-owner authority before side effects, accepting one small allocation
per registry and reference-counted witnesses while commands or observations remain live.

The proposal reuses current context-liveness and expected-epoch checks. It does not freeze a
context-wide subscription at its initial document epoch, authenticate the browser session associated
with a stream, or turn the witness into a durable ID or capability grant. The witness has no public
constructor or serialization. It cannot preserve a removed context or recreate a retired registry.
The four real-socket failures recorded at `b3ffeac9` exercise send, receipt admission, event admission
and a correctly admitted observation presented to a different mutation target. Status remains
Proposed; local source acceptance does not approve the architecture or complete hosted, protected-main
or browser compatibility gates.

OriginWeave carries adapter maintenance and version negotiation but gains a durable customer API. Multiple browser/control transports can coexist. New upstream capabilities do not silently change risk or action semantics. Compatibility matrices become release artifacts.

## Failure and degraded behavior

Adapter negotiation failure disables only affected capabilities. Unsupported or schema-incompatible messages fail closed with typed errors. OriginWeave must not bypass a failed adapter by exposing raw CDP or arbitrary JavaScript to an autonomous model. A standards adapter may fall back to a pinned vendor adapter only when the same OriginWeave semantic and security contract is proven.

## Security / privacy / governance impact

Protocol validation occurs before messages influence policy. Tool/page-provided strings remain untrusted. Secret handles never become raw secret protocol payloads; only the separately authorized trusted broker-to-browser delivery path may materialize the value, and that value does not pass through MCP, WebMCP, BiDi observation, or model-visible CDP output. Adapter version/provenance is recorded for audit and incident reconstruction.

## Tests and acceptance evidence

Require version-negotiation tests, schema/property tests, malformed-message tests, BiDi/CDP semantic parity tests for shared capabilities, WebMCP prompt-injection tests, MCP authority-separation and version-change tests, browser-version compatibility matrices, and end-to-end proof that unsupported capabilities fail without side effects.

## Migration and rollback

Adapters are independently versioned and can be canaried. Clients migrate through OriginWeave Protocol compatibility rules, not upstream protocol rewrites. Rollback pins a previously supported adapter/browser/protocol pair and records that pair in provenance.

## Open follow-ups

Define internal protocol versioning rules, adapter capability descriptors, minimum supported BiDi level, CDP pin policy, and MCP/WebMCP schema isolation.

## Supersession / reversal conditions

Supersede if one mature standard gains all required capabilities, stable compatibility, explicit security semantics, and broad implementation support sufficient to replace the internal abstraction without exposing customers to upstream churn.

## References

Chrome DevTools Protocol. (2026). *Chrome DevTools Protocol — latest (tip-of-tree)*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/

Chrome DevTools Protocol. (2026). *WebMCP domain*. Chromium. Retrieved August 9, 2026, from https://chromedevtools.github.io/devtools-protocol/tot/WebMCP/

Parra, D. S., & Delimarsky, D. (2026, July 28). *The 2026-07-28 specification*. Model Context Protocol Blog. https://blog.modelcontextprotocol.io/posts/2026-07-28/

World Wide Web Consortium. (2026, June 29). *WebDriver BiDi* [Working Draft]. https://www.w3.org/TR/2026/WD-webdriver-bidi-20260629/

## Related documents

See `docs/API_CONTRACT.md`, `docs/TRD.md`, `docs/doctoring/product-documentation-baseline.md`, and `docs/DATA_GOVERNANCE.md`.
