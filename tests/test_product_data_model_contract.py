"""Regression contracts for the conceptual OriginWeave product data/evidence model."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class ProductDataModelContractTests(unittest.TestCase):
    """Keep durable identities separate across session, source, action, and evidence domains."""

    def test_conceptual_erd_covers_required_product_entities(self) -> None:
        erd = (ROOT / "docs/erd/README.md").read_text(encoding="utf-8")
        for entity in (
            "tenant_record",
            "agent_session",
            "browser_profile",
            "page_snapshot",
            "semantic_node",
            "network_exchange",
            "content_record",
            "extraction_schema",
            "extracted_value",
            "action_event",
            "policy_decision",
            "approval_evidence",
            "provenance_record",
            "download_artifact",
            "task_checkpoint",
            "resource_budget",
            "secret_handle",
            "extension_grant",
        ):
            with self.subTest(entity=entity):
                self.assertIn(entity, erd)

    def test_conceptual_erd_does_not_claim_unimplemented_persistence(self) -> None:
        erd = (ROOT / "docs/erd/README.md").read_text(encoding="utf-8")
        for phrase in (
            "Conceptual unless",
            "does not yet implement every entity as a relational table",
            "Planned durable record",
            "Adapter-owned representation",
        ):
            with self.subTest(phrase=phrase):
                self.assertIn(phrase, erd)

    def test_extraction_schema_is_distinct_from_values_and_source_content(self) -> None:
        erd = (ROOT / "docs/erd/README.md").read_text(encoding="utf-8")
        for relationship in (
            "extraction_schema ||--o{ extracted_value",
            "content_record ||--o{ extracted_value",
            "source_resource ||--o{ content_record",
        ):
            with self.subTest(relationship=relationship):
                self.assertIn(relationship, erd)


if __name__ == "__main__":
    unittest.main()
