"""Architectural fitness tests for bounded-context ownership."""

from __future__ import annotations

import pathlib
import tomllib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class DomainBoundaryTests(unittest.TestCase):
    """Keep protocol adapters out of the shared domain-contract kernel."""

    def test_mcp_protocol_contract_has_its_own_adapter_crate(self) -> None:
        """MCP routing DTOs belong to the MCP adapter, not originweave-core."""

        workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertIn("crates/originweave-mcp", workspace["workspace"]["members"])
        self.assertFalse((ROOT / "crates/originweave-core/src/mcp.rs").exists())

        mcp_manifest = tomllib.loads(
            (ROOT / "crates/originweave-mcp/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertEqual(
            set(mcp_manifest.get("dependencies", {})),
            {"originweave-core"},
        )

        policy_manifest = tomllib.loads(
            (ROOT / "crates/originweave-policy/Cargo.toml").read_text(encoding="utf-8")
        )
        self.assertIn("originweave-mcp", policy_manifest["dependencies"])

        policy_source = (ROOT / "crates/originweave-policy/src/lib.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("originweave_core::mcp", policy_source)
        self.assertIn("originweave_mcp::ValidatedMcpToolCall", policy_source)


if __name__ == "__main__":
    unittest.main()
