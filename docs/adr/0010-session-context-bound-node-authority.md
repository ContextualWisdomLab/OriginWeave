# ADR-0010: Bind observed node authority to browser sessions and contexts

- Status: Accepted
- Date: 2026-08-09

## Context

A browser adapter may expose an implementation-local node identifier that is meaningful only inside one automation session and one current browsing context. Binding that identifier only to a canonical origin and document epoch is insufficient: two isolated sessions or two same-origin tabs or frames can reuse the same local identifier while referring to different nodes.

The WebDriver element-retrieval algorithm resolves a node reference using the session and the session's current browsing context. It reports a stale element when the resolved node is no longer attached to the active document. WebDriver BiDi likewise treats browsing contexts as explicit protocol identities and makes navigation state context-specific. OriginWeave therefore must not treat an adapter-local node number, origin, or epoch as independently durable authority.

## Decision

OriginWeave represents actionable observed-node authority with five explicit values:

1. a nonzero `BrowserSessionId` for the active browser automation session;
2. a nonzero `BrowsingContextId` for one independently navigable tab or frame context;
3. the exact canonical `Origin` observed for that context;
4. a nonzero `DocumentEpoch` allocated for the actionable document lifetime; and
5. a nonzero adapter-local node identifier.

`ObservedNodeHandle` stores all five values. Immediately before acting, an adapter must compare the handle with the current session, browsing context, canonical origin, and document epoch. Any mismatch fails closed before the adapter issues a browser action.

The numeric identifiers are internal opaque registry identities. They are not raw WebDriver, WebDriver BiDi, CDP, renderer, process, frame-tree, or DOM identifiers, and they are not durable across process restarts. A protocol adapter must translate external identifiers through a validated, session-scoped registry and allocate collision-free internal identities. It must rotate the document epoch whenever the actionable document lifetime changes, including navigation, document replacement, or a same-document mutation classified by `SameDocumentMutationKind` as able to change the target's identity or user-visible action semantics.

## Consequences

- A node handle cannot cross automation sessions, tabs, frames, origins, navigations, or document replacements by accident.
- Same-origin contexts with colliding adapter-local node identifiers remain distinct.
- The Rust core stays independent of Chromium, WebDriver, selectors, script execution, network access, storage, credentials, and model providers.
- Future WebDriver BiDi and CDP adapters must own external-to-internal identity translation, registry lifecycle, epoch rotation, and immediate pre-action validation.
- A valid handle proves only observation authority. It does not grant a browser capability, origin permission, resolved-destination authority, transport authority, approval, or successful post-condition.

## References

World Wide Web Consortium. (2026). *WebDriver*. Retrieved August 9, 2026, from https://w3c.github.io/webdriver/

World Wide Web Consortium. (2026). *WebDriver BiDi*. Retrieved August 9, 2026, from https://w3c.github.io/webdriver-bidi/
