"""Binding documentation contracts for resolved-destination security policy."""

from __future__ import annotations

import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class DestinationGovernanceTests(unittest.TestCase):
    """Prevent documentation from weakening executable destination guarantees."""

    def test_binding_documents_state_exact_destination_contracts(self) -> None:
        """Public-only, origin-bound, and per-hop guarantees must remain explicit."""

        required_terms = {
            "README.md": (
                "permits only public destinations by default",
                "pins approved dns address sets",
                "for each redirect",
            ),
            "ARCHITECTURE.md": (
                "the default web policy admits only addresses classified as public",
                "resolution snapshots are non-empty and bound to one logical origin",
                "every redirect must have a read-origin grant",
            ),
            "docs/product-roadmap.md": (
                "public-only default",
                "origin-bound approved resolution snapshots",
                "per-hop redirect",
            ),
        }
        for relative, terms in required_terms.items():
            document = (ROOT / relative).read_text(encoding="utf-8").lower()
            for term in terms:
                self.assertIn(term, document, relative)

    def test_registry_maintenance_is_recurring_and_release_blocking(self) -> None:
        """Security registries and platform endpoints require scheduled review."""

        maintenance = (ROOT / "docs/registry-maintenance.md").read_text(
            encoding="utf-8"
        ).lower()
        for term in [
            "before every release and at least monthly",
            "iana ipv4 special-purpose address space",
            "iana ipv6 special-purpose address space",
            "iana ipv6 global unicast address space",
            "rfc 9637",
            "168.63.129.16",
            "169.254.170.23",
            "fd00:ec2::23",
            "exact 100% production function, line, region, and branch coverage",
        ]:
            self.assertIn(term, maintenance)

    def test_chromium_canonicalizer_reference_is_immutable(self) -> None:
        """Origin-equivalence evidence must be reproducible after upstream changes."""

        doctoring = (ROOT / "docs/doctoring.md").read_text(encoding="utf-8")
        immutable_commit = "446d05d21720f0b3505ec21057b3e9f909784262"
        self.assertIn(immutable_commit, doctoring)
        self.assertNotIn("chromium/src/+/HEAD/url/url_canon_unittest.cc", doctoring)


if __name__ == "__main__":
    unittest.main()
