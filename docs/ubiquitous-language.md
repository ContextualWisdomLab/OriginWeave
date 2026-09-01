# OriginWeave Ubiquitous Language

This glossary defines terms that must mean the same thing in code, tests, ADRs, APIs, issues, and product documentation. When an external protocol uses a conflicting term, the adapter translates it at the boundary instead of changing the OriginWeave meaning.

| Term | Meaning in OriginWeave | Ownership / invariant |
|---|---|---|
| Origin | Canonical logical web identity: scheme, host, and effective port | `originweave-core`; never means a resolved IP address or an SSRF authorization |
| Destination | Concrete network address proposed or approved for connection | `originweave-destination`; must be classified and explicitly authorized |
| Resolution Snapshot | Non-empty, origin-bound set of approved resolver results | `originweave-destination`; later DNS answers may contract but not expand without reauthorization |
| Connection Plan | Bounded, single-use authority to attempt one or more exact direct TCP connections | `originweave-network`; contains no hostname-resolution authority |
| Direct TCP Connection | Connected stream whose operating-system peer has been checked against the requested canonical socket | `originweave-network`; proves peer equality, not TLS identity |
| TLS Service Identity | WebPKI-authenticated service identity for the canonical HTTPS origin on an existing verified TCP stream | `originweave-tls`; cannot reconnect or re-resolve |
| Session Mode | Human, Assist, Agent Task, or Crawler execution posture | `originweave-core`; determines the authority model, not just UI presentation |
| Execution Purpose | Explicit reason for a governed execution, such as a user-delegated task or public crawl | `originweave-core`; must agree with the allowed session mode |
| Capability | Explicit permission category required by an `ActionKind` | `originweave-core`; absence is denial, never inferred from ambient browser state |
| Action Kind | Closed typed operation that an agent may request | `originweave-core`; production agents do not receive unrestricted JavaScript as a standard action |
| Action Request | Complete typed request binding action, source/target origin, instruction source, secret-delivery mode, and intent digest | `originweave-core`; input to policy, not proof of authorization |
| Action Intent Digest | Immutable digest of the canonical complete user/enterprise intent relevant to approval | `originweave-core`; approval must bind to it exactly |
| Approval Scope | Exact action + target origin + intent digest tuple to which approval applies | `originweave-core`; scopes are not fungible across actions or destinations |
| Approval Evidence | User or enterprise evidence authorizing one exact `ApprovalScope` | `originweave-core`; stale or mismatched evidence fails closed |
| Policy Decision | `Allow`, `Deny`, or `RequireApproval` result for a typed request and explicit context | `originweave-policy`; adapters do not manufacture this decision |
| Instruction Source | Provenance category describing where an instruction originated | `originweave-core`; web/page content is data and cannot promote itself to trusted authority |
| Secret Delivery | Whether an action receives no secret or an opaque broker handle | `originweave-core`; raw secret values must not enter model context or generic action payloads |
| Risk Class | R0–R5 action-risk classification used by policy | `originweave-core`; R5 is non-delegable under the current contract |
| Evidence | Bounded, credential-free or value-redacted proof emitted by a boundary about the invariant it actually checked | `originweave-evidence` and producing contexts; evidence from one boundary cannot stand in for another |
| Resource Budget | Explicit bounded allowance for task/resource consumption | `originweave-resource`; hard pressure must reduce the active consumer and reject unsafe admission |
| Mitigation Plan | Deterministic cumulative response to observed resource pressure | `originweave-resource`; may combine actions rather than collapse simultaneous pressures into one flag |
| MCP Route | Validated MCP protocol-version/method/tool binding that maps to an OriginWeave `ActionKind` | `originweave-mcp` active PR #272; proves routing consistency only, not authorization |
| MCP Tool | External MCP-visible name for one supported typed OriginWeave action | `originweave-mcp`; unknown or malformed names fail closed |
| Browser Agent Protocol (BAP) | OriginWeave browser-agent protocol contract and lifecycle vocabulary | `originweave-bap`; it is not a synonym for MCP, WebDriver BiDi, or CDP |
| Adapter | Boundary translating an external protocol/runtime representation into OriginWeave contracts | Protocol/infrastructure context; must not leak provider DTOs into domain contracts |
| Anti-Corruption Layer | Translation boundary that prevents an external model or foreign bounded context from redefining OriginWeave domain terms | Required at protocol/provider edges when vocabularies or authority models differ |
| Shared Kernel | Deliberately tiny, versioned contract surface shared by bounded contexts | `originweave-core` is the current kernel; additions require stable cross-context meaning, not convenience |
| Protected-main truth | Behavior and documentation integrated into protected `main` with its required governance evidence | Open PRs, predecessor checks, model comments, and planned modules are not protected-main truth |
| Active-PR evidence | Code, tests, checks, or documentation that exists on an open PR exact head but is not yet shipped | Must remain labeled as active until protected integration |

## Terms that must not be collapsed

These distinctions are security and product invariants, not editorial preferences:

- **Origin ≠ destination ≠ TCP peer ≠ TLS service identity.** Each is proven at a different boundary.
- **MCP route ≠ policy decision ≠ browser execution.** Protocol validity cannot grant capability or approval.
- **Capability ≠ approval.** Possessing the capability to request an action does not satisfy approval requirements for its risk class.
- **Action request ≠ observed post-condition.** Dispatch success is not proof that the intended browser state was reached.
- **Evidence ≠ authority.** Evidence records what a boundary proved; possession of an evidence object does not create new permission.
- **Crawler policy ≠ access authorization.** Robots decisions govern crawl behavior but do not create authentication or data-access rights.
- **Planned context ≠ current implementation.** A name in the architecture roadmap is not shipped ownership until code and tests reach protected `main`.

## Naming rules

Public Rust types, modules, tests, API fields, database objects, and documentation should use these terms directly where they express the same concept. External names such as MCP `tools/call`, WebDriver BiDi command names, CDP domains, provider SDK objects, Chromium internals, or persistence DTOs stay at their adapters. Do not rename a domain concept merely to match an external provider vocabulary.

Generic names such as `utils`, `helpers`, `common`, `services`, `models`, `misc`, `legacy`, or an unqualified `data`/`security`/`browser` directory do not establish domain ownership. A new package or module should name the stable responsibility it owns or live inside the already accepted bounded context that owns that responsibility.
