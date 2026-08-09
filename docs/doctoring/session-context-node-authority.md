# Browser Session and Context Node Authority

## Governing question

What minimum identity must OriginWeave retain before a future browser adapter may act on a previously observed node?

## Primary-spec evidence

WebDriver defines each browsing context as having a unique window handle. Its known-element algorithm resolves an element reference with both the active session and that session's current browsing context. It returns `no such element` when the reference is unknown in that scope and `stale element reference` when the resolved node is no longer attached to the active document.

WebDriver BiDi makes browsing contexts explicit protocol objects and associates navigation behavior with the selected context. Because these external protocol identifiers and their lifecycle remain adapter concerns, OriginWeave must not expose them directly as durable product authority.

## Product decision

The Rust safety kernel uses independently validated opaque identities for:

- the active browser automation session;
- the independently navigable browsing context;
- the canonical origin observed in that context;
- the actionable document lifetime; and
- the adapter-local node identifier.

A future adapter must translate WebDriver, WebDriver BiDi, CDP, renderer, frame-tree, or DOM identifiers through a session-scoped registry. It must allocate a fresh document epoch when navigation or document replacement makes earlier references unsafe. Immediately before an action, it must compare every authority component and fail closed on a session, context, origin, or epoch mismatch.

This contract is deliberately stricter than testing only whether a local node number still exists. Same-origin tabs, frames, isolated profiles, or parallel sessions may reuse the same local identifier while referring to different nodes. It also does not convert a valid node handle into an action capability, origin grant, destination grant, approval, or proof of a successful post-condition.

## Required regression evidence

- zero-valued session, context, epoch, and node identifiers are rejected;
- a handle validates only in the exact session and context that produced it;
- same-origin cross-session and cross-context reuse is rejected;
- cross-origin reuse is rejected;
- stale document epochs are rejected;
- errors remain deterministic and implement `std::error::Error`;
- all public contracts remain documented and production behavior remains exactly covered.

## References

World Wide Web Consortium. (2026). *WebDriver*. Retrieved August 9, 2026, from https://w3c.github.io/webdriver/

World Wide Web Consortium. (2026). *WebDriver BiDi*. Retrieved August 9, 2026, from https://w3c.github.io/webdriver-bidi/
