# WebDriver BiDi committed-navigation unsubscribe traceability

- **Status:** Active-PR evidence; not protected-main truth
- **Reviewed standard:** W3C WebDriver BiDi Working Draft, 3 September 2026
- **Bounded context:** Navigation / WebDriver BiDi adapter
- **Consumer:** exact typed receipt from OriginWeave's committed-navigation `session.subscribe` boundary

## Problem

The committed-navigation subscription slice retains one bounded opaque `session.Subscription` identifier, but the predecessor unsubscribe implementation was still coupled to the earlier generic command-correlation API. After the parent stack introduced exact command-family correlation, that implementation could no longer compile or preserve the invariant that a response for one BiDi command family must not retire another outstanding command that happens to reuse the same numeric id.

The predecessor failure contract also registered correlation before rejecting an invalid frame deadline. That contradicts the current local no-write invariant used by the adjacent subscription, pointer-click, status, and session-end transports: a deadline rejected before any command bytes can be emitted must not reserve an outstanding remote-effect correlation.

## Standard boundary

The 3 September 2026 WebDriver BiDi Working Draft defines `session.unsubscribe` with `session.UnsubscribeParameters = session.UnsubscribeByAttributesRequest / session.UnsubscribeByIDRequest`. The by-id request carries one or more opaque `session.Subscription` values in `subscriptions`, and `session.UnsubscribeResult` is `EmptyResult`.

OriginWeave uses only the by-id form and accepts the identifier only through its already validated typed `session.subscribe` result. This intentionally narrower adapter does not expose arbitrary event names, context sets, user-context sets, or caller-supplied ambient subscription identifiers.

## Decision

1. Give committed-navigation unsubscribe its own `WebDriverBiDiCommandKind::NavigationCommittedUnsubscribe` provenance rather than reusing the subscription kind or a generic correlation path.
2. Reject an invalid frame timeout before registering correlation or writing command bytes.
3. Register the exact unsubscribe kind immediately before the first possible remote side effect.
4. Retire that exact correlation only when the frame owner reports a local `MalformedFrame` preflight failure proving that command bytes were not emitted. Preserve correlation after partial or otherwise ambiguous write failures.
5. Admit success or protocol-error responses only through the matching unsubscribe command kind. A response with the same numeric id but a different command family fails closed without consuming the outstanding command.
6. Treat `EmptyResult` success as protocol acknowledgment only. It does not prove that already-in-flight subscribed events have drained, that navigation state is unchanged, or that any browser/process/profile cleanup completed.

## Rejected alternatives

- **Reuse `NavigationCommittedSubscription` for unsubscribe:** rejected because numeric command ids are routing values, not command provenance; a typed consumer must not cross-consume a different command family.
- **Restore generic `register_command` / `correlate_response`:** rejected because it would reopen the command-kind confusion repaired by the parent stack.
- **Keep correlation after an invalid local deadline:** rejected because no remote side effect can have begun; retaining a phantom outstanding command would reduce the bounded 256-command budget and make id reuse falsely ambiguous.
- **Retire correlation after any frame error:** rejected because a partial or complete command may have reached the remote end after I/O begins.
- **Treat unsubscribe ACK as event-drain evidence:** rejected because the protocol acknowledgment is not a post-condition proving absence of already-in-flight events.

## Executable evidence required on the exact head

- loopback TCP → RFC 6455 opening exchange → typed committed-navigation subscribe → opaque subscription receipt → by-id unsubscribe → exact correlated `EmptyResult` success;
- opaque identifier escaping across quote, backslash, control and Unicode text without logging the identifier itself;
- command-id range and duplicate outstanding-id rejection;
- invalid deadline rejection with zero newly outstanding unsubscribe correlations and no command write;
- malformed and unknown-id responses leaving the exact unsubscribe correlation outstanding;
- same-id wrong-command-kind response rejection without correlation consumption;
- matched protocol error consuming only its exact unsubscribe command;
- local `MalformedFrame` preflight retirement versus ambiguous frame-write retention;
- repository formatting, full Rust tests, strict Clippy, rustdoc, and owned-production function/line/region/branch coverage at 100% before any GREEN claim.

## Authority and follow-up

This adapter performs no policy authorization, destination approval, browser authentication, action dispatch, semantic observation, or durable evidence escalation. The browser-domain owner remains OriginWeave; WebDriver BiDi remains an adapter. Integration into protected main remains parent-first and non-destructive, and exact-head hosted evidence does not transfer from predecessor heads.

### References — APA 7th

World Wide Web Consortium. (2026, September 3). *WebDriver BiDi* (W3C Working Draft). https://www.w3.org/TR/2026/WD-webdriver-bidi-20260903/
