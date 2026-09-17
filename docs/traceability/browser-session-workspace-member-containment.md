# Browser Session workspace-member containment

- **Status:** active-PR security evidence for PR #317; not protected-main behavior
- **Owner:** OriginWeave Browser Session bounded context
- **Canonical contract:** `tests/test_browser_session_trusted_adapter_boundary.py`
- **Related evidence:** `docs/traceability/browser-session-trusted-adapter-boundary.md`, `docs/THREAT_MODEL.md`

## Problem

The trusted-adapter contract treats repository-local Cargo package/source topology as the review boundary for code that can participate in Browser Session lifecycle composition. Local production `path` dependencies and custom target sources already fail closed when their resolved paths escape the repository review root, but explicit `[workspace].members` did not apply the same containment check.

Cargo's workspace contract allows `members` to name package directories rather than restricting them to a `crates/*` convention. The Cargo Book also documents `package.workspace` specifically for member packages that are not under the workspace root. A relative member such as `../external-browser-adapter` can therefore identify code outside the repository tree. Merely checking that its `Cargo.toml` exists is insufficient for a repository-scoped TCB review contract.

## RED and repair

- **Structural RED `44b6d2716ade55c3793698080bfda01ff51a23c1`** adds a hostile workspace fixture whose explicit member resolves to `../external-browser-adapter`. The previous `_workspace_member_manifests()` accepted the external manifest because it checked only existence.
- **Minimal causal repair `a807accfddea31a5739f47dcfb992057f7292c03`** resolves each explicit member manifest and requires it to remain under the repository review root before it may enter the canonical production package closure. A member that escapes the root now raises instead of silently expanding the trusted composition surface.
- **Independent-review finding on `afb8ab82adcb1da564930798f682f10b38cf818b`** accepted the containment implementation, including canonical path resolution, but identified one P2 coverage gap: the committed hostile fixture covered `..` escape but not a repository-local symlink whose target resolves outside the review root.
- **Review-driven regression `ddf17ba21581473c20fdd5c7f7b3223240edd262`** adds `tests/test_browser_session_workspace_member_symlink_contract.py`. The fixture creates a member path inside the workspace that is a directory symlink to an external package and requires the canonical scanner to reject the resolved external manifest. It imports the single-writer scanner rather than creating another topology implementation.

The repair changes repository security coverage only. No production Rust, Browser Session runtime semantics, WebDriver BiDi authority, Chromium behavior, or existing allowlist entry changed.

## Decision

External Cargo workspace members are rejected by this repository contract even if Cargo itself can model such membership. The product security boundary is intentionally narrower than Cargo's general project-layout flexibility: code outside the repository cannot be proven by OriginWeave's exact-head review, provenance, branch protection, or release evidence.

Alternative approaches were rejected:

- **Accept the external member because Cargo accepts it:** rejected because repository review/provenance cannot establish the external source generation.
- **Copy external code into the scanner's evidence:** rejected because that turns mutable external source into an implicit dependency instead of a released/versioned owner contract.
- **Permit an allowlisted filesystem path outside the repository:** rejected because the path is not an immutable release identity and would bypass normal PR/release provenance.

If OriginWeave later needs an external browser adapter, it must arrive through a released/versioned dependency or another canonical owner boundary rather than an unversioned workspace-member filesystem edge.

## Authoritative reference

The Cargo Book, *Workspaces*, documents `[workspace].members` as package-directory entries, automatic in-workspace path-dependency membership, and `package.workspace` for explicitly identifying a workspace root when the member is not beneath it: https://doc.rust-lang.org/cargo/reference/workspaces.html

## Remaining acceptance

This repair is structural security evidence on the Draft #317 branch. It does not transfer executable GREEN from another generation. The current exact head still requires focused independent review of the review-driven symlink regression, and #229 current exact-head repository/security evidence remains the lineage prerequisite. After authorized ordinary/non-force ancestry reconciliation, #317 must obtain fresh exact-head checks and review before #318 → #321 → #316 restacking. Real Chromium acceptance remains downstream under #299 and the canonical `.github` MV3 workflow/sandbox owner path.
