# Release SBOM JSON nesting-depth invariant

- Status: Proposed
- Owning issue: #201
- Active repair: #221
- Intended Release owner: `originweave-release` after the #240 owner lineage is integrated

PR #221 currently sets `MAX_SPDX_JSON_NESTING_DEPTH = 256` in the provisional SPDX 3.0.1 JSON-LD verifier. The string-aware preflight rejects depth 257 with `too_deep_json_structure` before `json.loads`, admits the exact depth-256 boundary to normal JSON grammar validation, and does not count bracket-looking characters inside JSON strings toward the depth budget.

This limit is an OriginWeave resource-admission policy, not an SPDX semantic requirement. RFC 8259 section 9 permits JSON implementations to set a maximum nesting depth. The final Rust-owned Release implementation must preserve or deliberately strengthen the same explicit pre-materialization depth invariant together with the existing byte, container, structure-token/member, graph, and numeric limits; generic parser recursion must not become the effective boundary.

The depth lineage is review `5097073272`, test-only candidate `16a2e1eedf121c6e89eb9019db0906e8ea49f0ce`, and production/test repair `93f1d13e38c07dbe1217bbfa50d18d5eb696f094`. The test-only candidate did not acquire a hosted runner, so it is candidate test-first evidence rather than an observed hosted RED.

As of September 3, 2026, #221 still targets SPDX 3.0.1. SPDX 3.1-RC1 is a release candidate and does not implicitly change this contract; adopting it requires an explicit version, schema, serialization, and ontology migration.

## References

Bray, T. (2017). *The JavaScript Object Notation (JSON) Data Interchange Format* (RFC 8259; STD 90). Internet Engineering Task Force. https://www.rfc-editor.org/rfc/rfc8259

SPDX Workgroup. (2026). *SPDX specification 3.0.1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.0.1/serializations/

SPDX Workgroup. (2026). *SPDX specification 3.1-RC1: Model and serializations*. The Linux Foundation. https://spdx.github.io/spdx-spec/v3.1-RC1/serializations/
