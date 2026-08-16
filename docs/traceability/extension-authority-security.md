# Extension-to-Agent Security Traceability

- **Documentation status:** Active-PR evidence dossier
- **Canonical owner:** PR #44 (`docs: reconcile architecture documentation fitness`)
- **Protected-main baseline:** `0c376acf059be9ddddddfbde1d0189e4f39ef014`
- **Capability maturity:** **PARTIAL**
- **Governing decision:** Proposed ADR 0013 separates Manifest V3 compatibility from OriginWeave Agent authority.

## 1. Why this dossier exists

Manifest V3 compatibility and OriginWeave Agent authority are intentionally different evidence domains. A Chromium extension may possess Chrome permissions and may be explicitly granted a narrow OriginWeave extension capability without receiving Agent origin grants, Agent action capability, instruction trust, secret-delivery authority, approval, or protected-value access.

This dossier records the current executable composition evidence for that separation. It does not promote active pull requests to protected-main shipped truth and it does not claim the trusted sensitive-data broker from issue #10 is complete.

## 2. Protected-main authority

Protected `main` already provides:

- exact extension/session/context-scoped `ExtensionAgentGrant` evaluation;
- a distinction between `ObserveCurrentContext` and `ProposeTypedAction` extension capabilities;
- deterministic Agent policy evaluation for typed actions;
- fail-closed treatment of `InstructionSource::WebContent`;
- explicit Agent capability and readable/writable-origin gates;
- `FillSecret` policy that rejects raw secret delivery and requires `SecretDelivery::BrokerHandle`; and
- ordinary action-risk approval semantics that remain separate from extension permission.

These foundations are **IMPLEMENTED_ON_PROTECTED_MAIN**. They do not by themselves prove every issue #27 cross-boundary composition case.

## 3. Active executable evidence

### PR #175 — Chrome permission cannot mint Agent action authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `f6c089b8faae015c5b8845296af7cdc3ddcd0c65` adds the fail-closed `chrome_permission_authorizes_agent_action` boundary. Reviewed Chrome compatibility permissions such as `downloads`, `bookmarks`, `history`, `storage`, `tabs`, `scripting`, `sidePanel`, `declarativeNetRequest`, and `declarativeNetRequestWithHostAccess` are classified as compatibility surfaces only; malformed, case-shifted, or unrecognized tokens remain unrecognized. The function never returns successful Agent authorization, so Manifest V3 compatibility evidence cannot mint `Capability::Download` or another Agent action.

This is active-PR evidence only. Until PR #175 integrates into protected `main`, the function must not be listed as a protected-main capability.

### PR #62 — proposal authority cannot widen Agent, instruction, or secret-material authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `e7265a86d63c9e5f047ed6d32c3988b01e53fa13` proves that, after an extension is genuinely allowed to `ProposeTypedAction`:

1. a proposed navigation outside the Agent readable-origin grant is still denied;
2. proposal permission cannot supply the missing Agent `Navigate` capability;
3. extension-produced `WebContent` cannot become a trusted policy instruction;
4. `FillSecret` with `SecretDelivery::RawValue` remains denied as `SecretBrokerRequired`; and
5. secret material attached to a non-secret action remains denied as `UnexpectedSecretMaterial`.

The branch adds no production API and no extension runtime. It is compositional security evidence over protected-main authorities.

### PR #63 — proposal authority cannot manufacture high-risk approval

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `a4595c393f459f57bfe2199ace44271f246751c4` deliberately keeps only the distinct approval-composition proof after duplicate regressions were removed in favor of PR #62 ownership. It proves that, after the same exact proposal grant is admitted and the Agent context independently possesses `FillSecret` plus exact readable/writable origin authority, broker-handle `FillSecret` still reaches the ordinary R3 approval boundary rather than becoming implicitly allowed.

The exact head has successful CI, exact owned production coverage, Security Scan, and SAST evidence and is Ready for review. It has no raw secret bytes and does not create approval evidence, a broker, browser-fill adapter, protected-value store, KMS path, authenticated workload identity, persistence owner, or release claim.

## 4. Security interpretation

The executable authority chain is intentionally non-transitive:

```text
Chromium extension permission
-> explicit extension/session/context grant
-> permission to propose a typed action
-/> Agent capability
-/> Agent readable/writable origin
-/> trusted instruction source
-/> secret-delivery authority
-/> approval
-/> protected-value resolution
```

A future real extension adapter must preserve these separations. Chrome permissions and extension proposal grants are inputs to policy composition, never ambient authority that bypasses the deterministic Agent policy or the sensitive-data broker boundary.

## 5. Remaining issue #27 / #10 boundary

This dossier does **not** close issue #27 or issue #10. Remaining material work includes, among other accepted requirements:

- real managed-extension allow-list and enterprise policy integration;
- native-messaging host boundary and process isolation;
- complete supported-capability release matrix and regression gate;
- authenticated workload/service identity for sensitive-data broker audience;
- protected-value resolution/fill outside model-visible context;
- durable transactional handle lifecycle, retention, encryption/KMS, deletion and audit-export controls; and
- protected-main integration plus fresh acceptance before any active-PR evidence becomes shipped truth.

## 6. Documentation fitness consequence

The existing ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PRs #62, #63, and #175 narrow distinct executable extension-authority evidence gaps without introducing a new trust domain, deployment component, persistence entity, database schema, or independent architecture decision. Proposed ADR 0013 remains Proposed until its own lifecycle authority changes.
