"""Repository contract for the MCP adapter dependency direction."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class McpAdapterRepositoryContractTests(unittest.TestCase):
    """Keep MCP transport types outside shared domain authority."""

    def test_mcp_adapter_isolated_from_shared_domain_contracts(self) -> None:
        """MCP may depend inward on policy; policy must not depend outward on MCP."""

        self.assertFalse((ROOT / "crates/originweave-core/src/mcp.rs").exists())

        mcp_manifest = tomllib.loads(
            (ROOT / "crates/originweave-mcp/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertEqual(
            set(mcp_manifest.get("dependencies", {})),
            {"originweave-core", "originweave-policy"},
        )

        policy_manifest = tomllib.loads(
            (ROOT / "crates/originweave-policy/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertEqual(set(policy_manifest.get("dependencies", {})), {"originweave-core"})

        policy_source = (ROOT / "crates/originweave-policy/src/lib.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("originweave_mcp", policy_source)
        self.assertNotIn("ValidatedMcpToolCall", policy_source)
        self.assertNotIn("Mcp", policy_source)

        mcp_source = (ROOT / "crates/originweave-mcp/src/lib.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("originweave_policy", mcp_source)
        self.assertIn("evaluate_mcp", mcp_source)


if __name__ == "__main__":
    unittest.main()
