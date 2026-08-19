# Extension-to-Agent Security Traceability

- **Documentation status:** Active-PR evidence dossier
- **Canonical owner:** PR #44 (`docs: reconcile architecture documentation fitness`)
- **Protected-main baseline:** `0841d2ab3d8b5e60a03c0a8e818cf438e2716829`
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

Exact current head `3690bf0a351b77957071f5399e9a31cec5f39e0b` proves that, after an extension is genuinely allowed to `ProposeTypedAction`:

1. a proposed navigation outside the Agent readable-origin grant is still denied;
2. proposal permission cannot supply the missing Agent `Navigate` capability;
3. extension-produced untrusted content remains rejected as instruction authority;
4. `FillSecret` with `SecretDelivery::RawValue` remains denied as `SecretBrokerRequired`; and
5. secret material attached to a non-secret action remains denied as `UnexpectedSecretMaterial`.

The branch adds no production API and no extension runtime. It is compositional security evidence over protected-main authorities.

### PR #63 — proposal authority cannot manufacture high-risk approval

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `b80963fb81bed1ac4a01c1118f498af9765e2b79` deliberately keeps only the distinct approval-composition proof after duplicate regressions were removed in favor of PR #62 ownership. It proves that, after the same exact proposal grant is admitted and the Agent context independently possesses `FillSecret` plus exact readable/writable origin authority, broker-handle `FillSecret` still reaches the ordinary R3 approval boundary rather than becoming implicitly allowed.

This lane adds no broker, browser-fill adapter, protected-value store, KMS path, authenticated workload identity, persistence owner, or approval evidence.

### Origin-bound extension grant evaluation

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

The current origin-binding slice requires `ExtensionAgentGrant` and `ExtensionAccessRequest` to carry the same canonical origin. A same-session, same-context request for `https://other.example` or `https://app.example:8443` against a grant for `https://app.example` is `DenyOriginMismatch`. Exclusive trusted-time expiry is evaluated after that origin match: `now >= expires_at` is `DenyExpired`. This does not install an extension, parse Chrome messages, bind task identity, or mint Agent capabilities from Manifest V3 permissions.

### PR #82 — exact extension-to-native-host authority

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `c639cd78e3acad235be4cbbfdef67b84ce7ddbfa` provides bounded native-host names plus exact extension-ID/host-name grants and request identity getters. Chrome `nativeMessaging` permission remains separate from OriginWeave Agent authority. The lane does not parse an installed host manifest, read operating-system registration, launch a process, frame stdio, or trust native-host output.

### PR #154 — bounded native-messaging framing

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

Exact current head `4a71b7dd357974f791f8d7f4a0be5c4c0b9ea1b1`, stacked on #82, preserves current extension authority while bounding native-endian message framing with direction-specific payload ceilings, exact frame length, and UTF-8 text validation before later JSON/untrusted-observation handling. It does not prove host-manifest installation, process ownership, sandboxing, stdio provenance, or Agent authority.

### PR #169 — complete bounded host-manifest parsing and authority validation

**Capability maturity:** `IMPLEMENTED_ON_ACTIVE_PR`

This Draft, stacked on #154, now parses the admitted native-host manifest document completely at the reviewed schema boundary. Raw ingress must first be non-empty valid UTF-8, remain within the OriginWeave 64 KiB pre-parser safety budget before `String` allocation, and have one outer object-shaped envelope after the four JSON whitespace characters are ignored. Complete parsing then accepts exactly Chrome's reviewed required `name`, `description`, `path`, `type`, and `allowed_origins` members plus optional boolean `supports_native_initiated_connections`; rejects duplicate decoded member names, unknown or missing members, wrong JSON types, malformed escapes and surrogate pairs, malformed arrays, trailing commas, control characters, and trailing JSON data; and decodes JSON strings before authority-bearing values reach the existing validators.

`description` is required and type-checked because it belongs to the reviewed manifest schema but is intentionally not retained as authority. The parsed contract accepts only exact `stdio`, requires a non-empty bounded raw `allowed_origins` list, validates exact canonical `chrome-extension://<id>/` origins without wildcard or suffix normalization, collapses duplicate exact origins without widening authority, bounds executable-path allocation, preserves platform-specific path semantics, allows only an exact host plus explicitly listed extension identity, and retains the optional `supports_native_initiated_connections` declaration as data. The raw-document and executable-path limits are OriginWeave resource-governance limits, not Chrome or operating-system maximum claims.

Successful parsing still does not prove that the manifest is installed, authentic, OS-owned, policy-enabled, or bound to the process that will exchange native messages. A retained `supports_native_initiated_connections: true` value proves only that the parsed document declared the reviewed optional field; it does not prove Chromium feature or enterprise-policy enablement, installed registration, executable/process identity, native-initiated connection provenance, message trust, or Agent authority. This slice still does not read or authenticate filesystem/registry registration, canonicalize or attest an installed executable path, resolve a Windows relative path against an authenticated manifest directory, prove installer/OS ownership, spawn/sandbox/supervise a host process, authenticate the stdio peer, trust host message semantics, expose protected values, or grant Agent actions. Those remain separately reviewed runtime boundaries.

## 4. Security interpretation

The executable authority chains are intentionally non-transitive:

```text
Chromium extension permission
-> explicit extension/session/context/origin/time-bounded grant
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
-> bounded UTF-8, object-shaped manifest-document ingress
-> complete bounded reviewed JSON/schema parsing
-> validated exact host-manifest allow-list and executable-path intent
-> optional native-initiated-connection declaration retained as data only
-> bounded native-messaging framing
-/> Chromium feature / enterprise-policy enablement
-/> installed-host ownership / executable-process identity
-/> process identity / sandbox authority
-/> trusted message provenance
-/> Agent authority
-/> protected-value access
```

A future real extension/native-host adapter must preserve these separations. Chrome permissions, extension grants, bounded and fully parsed manifest documents, validated host-manifest fields, optional connection-direction declarations, and framed native bytes are inputs to explicit policy/provenance composition, never ambient authority that bypasses deterministic Agent or sensitive-data controls.

## 5. Remaining issue #27 / #10 boundary

This dossier does **not** close issue #27 or issue #10. Remaining material work includes, among other accepted requirements:

- trusted platform-specific native-host registration discovery, ownership checks, and installed-path validation, including Windows relative-path resolution against an authenticated manifest directory;
- process sandboxing, lifecycle supervision, authenticated stdio peer attribution, crash recovery, and fail-closed untrusted-message handling;
- real managed-extension allow-list and enterprise policy integration, including independent validation of any Chromium native-initiated-connection feature/policy state before use;
- complete supported-capability release matrix and regression gate;
- authenticated workload/service identity for sensitive-data broker audience;
- protected-value resolution/fill outside model-visible context;
- durable transactional handle lifecycle, retention, encryption/KMS, deletion and audit-export controls; and
- protected-main integration plus fresh acceptance before any active-PR evidence becomes shipped truth.

## 6. Documentation fitness consequence

The existing ADR/PRD/TRD/Architecture/UML/ERD graph remains **DESIGN-SUFFICIENT / PROTECTED-MAIN-PARTIAL**. PRs #62, #63, #82, #154, and #169 narrow distinct executable extension/native-messaging authority gaps without introducing an OriginWeave-owned database schema or a new architectural trust domain beyond Proposed ADR 0013. ADR 0013 remains Proposed until its own lifecycle authority changes.
