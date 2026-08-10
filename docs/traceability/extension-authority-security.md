# Extension-to-Agent Security Traceability

- **Documentation status:** Active-PR evidence dossier
- **Canonical owner:** PR #44 (`docs: reconcile architecture documentation fitness`)
- **Protected-main baseline:** `67af7c87589edc2039545af335c95064d9b8391c`
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

### PR #62 — proposal authority cannot widen Agent authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact head `277df965602a97b1c221df2fc7a228ff5ac6c540` proves that, after an extension is genuinely allowed to `ProposeTypedAction`:

1. a proposed navigation outside the Agent readable-origin grant is still denied;
2. proposal permission cannot supply the missing Agent `Navigate` capability; and
3. extension-produced untrusted content remains rejected as instruction authority.

The branch adds no production API and no extension runtime. It is compositional security evidence over protected-main authorities.

### PR #63 — proposal authority cannot widen secret authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact initial head `3059fead1ef0b6cf2f7df765b03c4b00a669b9cf` adds a separate integration regression showing that, after the same exact proposal grant is admitted:

1. `FillSecret` with `SecretDelivery::RawValue` remains denied as `SecretBrokerRequired`;
2. broker-handle `FillSecret` still reaches the ordinary R3 approval boundary rather than becoming implicitly allowed; and
3. broker material attached to a non-secret `Observe` action remains denied as `UnexpectedSecretMaterial`.

This evidence carries no raw secret bytes and does not create a broker, browser-fill adapter, protected-value store, KMS path, authenticated workload identity, persistence owner, or release claim.

## 4. Security interpretation

The executable authority chain is therefore intentionally non-transitive:

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

The existing ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PRs #62 and #63 narrow the executable extension-authority evidence gap without introducing a new trust domain, deployment component, persistence entity, database schema, or independent architecture decision. Proposed ADR 0013 remains Proposed until its own lifecycle authority changes.