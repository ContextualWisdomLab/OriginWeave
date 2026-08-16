# Extension-to-Agent Security Traceability

- **Documentation status:** Active-PR evidence dossier
- **Canonical owner:** PR #44 (`docs: reconcile architecture documentation fitness`)
- **Protected-main baseline:** `0c376acf059be9ddddddfbde1d0189e4f39ef014`
- **Capability maturity:** **PARTIAL**
- **Governing decision:** Proposed ADR 0013 separates Manifest V3 compatibility from OriginWeave Agent authority.

## 1. Why this dossier exists

Manifest V3 compatibility and OriginWeave Agent authority are intentionally different evidence domains. A Chromium extension may possess Chrome permissions and may be explicitly granted a narrow OriginWeave extension capability without receiving Agent origin grants, Agent action capability, instruction trust, secret-delivery authority, approval, protected-value access, or ambient native-process authority.

This dossier records current executable composition evidence for that separation. It does not promote active pull requests to protected-main shipped truth and it does not claim the trusted sensitive-data broker from issue #10 or the native-host process adapter from issue #27 is complete.

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

### PR #62 — proposal authority cannot widen Agent, instruction, or secret-material authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `e7265a86d63c9e5f047ed6d32c3988b01e53fa13` proves that, after an extension is genuinely allowed to `ProposeTypedAction`:

1. a proposed navigation outside the Agent readable-origin grant is still denied;
2. proposal permission cannot supply the missing Agent `Navigate` capability;
3. extension-produced untrusted content remains rejected as instruction authority;
4. `FillSecret` with `SecretDelivery::RawValue` remains denied as `SecretBrokerRequired`; and
5. secret material attached to a non-secret action remains denied as `UnexpectedSecretMaterial`.

The branch adds no production API and no extension runtime. It is compositional security evidence over protected-main authorities.

### PR #63 — proposal authority cannot manufacture high-risk approval

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `a4595c393f459f57bfe2199ace44271f246751c4` deliberately keeps only the distinct approval-composition proof after duplicate regressions were removed in favor of PR #62 ownership. It proves that, after the same exact proposal grant is admitted and the Agent context independently possesses `FillSecret` plus exact readable/writable origin authority, broker-handle `FillSecret` still reaches the ordinary R3 approval boundary rather than becoming implicitly allowed.

This lane adds no broker, browser-fill adapter, protected-value store, KMS path, authenticated workload identity, persistence owner, or approval evidence.

### PR #82 — exact extension-to-native-host authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `427d2f32431139dc7ed59e60df00fd9d0c4eeba0` provides bounded native-host names plus exact extension-ID/host-name grants and request identity getters. Chrome `nativeMessaging` permission remains separate from OriginWeave Agent authority. The lane does not parse an installed host manifest, read operating-system registration, launch a process, frame stdio, or trust native-host output.

### PR #154 — bounded native-messaging framing

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `d2e1ae8d654703b76897db202980fec82d26babc`, stacked on #82, bounds native-endian message framing with direction-specific payload ceilings, exact frame length, and UTF-8 text validation before later JSON/untrusted-observation handling. It does not prove host-manifest installation, process ownership, sandboxing, stdio provenance, or Agent authority.

### PR #169 — validated host-manifest authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Current Draft head `6d85f1a15e1501f48ce2d12d560323a36b38719b`, stacked on #154, adds test-first host-manifest authority. It accepts only exact `stdio`, requires a non-empty bounded raw `allowed_origins` list, validates exact canonical `chrome-extension://<id>/` origins without wildcard or suffix normalization, collapses duplicate exact origins without widening authority, and allows only an exact host plus explicitly listed extension identity.

The implementation intentionally treats caller-supplied validated manifest fields as one authority input only. It does not read JSON/filesystem/registry state, canonicalize or attest executable paths, prove installer/OS ownership, spawn/sandbox/supervise a host process, authenticate the stdio peer, parse/trust host JSON, expose protected values, or grant Agent actions. Those remain separately reviewed runtime boundaries.

## 4. Security interpretation

The executable authority chains are intentionally non-transitive:

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

and:

```text
Chrome nativeMessaging permission
-> exact extension/host grant
-> validated exact host-manifest allow-list
-> bounded native-messaging framing
-/> installed-host ownership
-/> process identity / sandbox authority
-/> trusted message provenance
-/> Agent authority
-/> protected-value access
```

A future real extension/native-host adapter must preserve these separations. Chrome permissions, extension grants, host-manifest fields, and framed native bytes are inputs to explicit policy/provenance composition, never ambient authority that bypasses deterministic Agent or sensitive-data controls.

## 5. Remaining issue #27 / #10 boundary

This dossier does **not** close issue #27 or issue #10. Remaining material work includes, among other accepted requirements:

- trusted platform-specific native-host registration discovery and ownership/path validation;
- process sandboxing, lifecycle supervision, authenticated stdio peer attribution, crash recovery, and untrusted-message handling;
- real managed-extension allow-list and enterprise policy integration;
- complete supported-capability release matrix and regression gate;
- authenticated workload/service identity for sensitive-data broker audience;
- protected-value resolution/fill outside model-visible context;
- durable transactional handle lifecycle, retention, encryption/KMS, deletion and audit-export controls; and
- protected-main integration plus fresh acceptance before any active-PR evidence becomes shipped truth.

## 6. Documentation fitness consequence

The existing ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PRs #62, #63, #82, #154, and #169 narrow distinct executable extension/native-messaging authority gaps without introducing an OriginWeave-owned database schema or a new architectural trust domain beyond Proposed ADR 0013. ADR 0013 remains Proposed until its own lifecycle authority changes.