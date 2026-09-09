"""Discovery entry point for preserved active-PR baseline history contracts."""

from _historical_baseline_contract_loader import export_public, load_historical_contract

_legacy = load_historical_contract(
    "legacy_documentation_active_pr_evidence_contract.py",
    "_originweave_legacy_documentation_active_pr_evidence_contract",
)
export_public(_legacy, globals())
