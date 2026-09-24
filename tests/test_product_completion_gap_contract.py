"""Discovery entry point for preserved commercial-completion history contracts."""

from _historical_baseline_contract_loader import export_public, load_historical_contract

_legacy = load_historical_contract(
    "legacy_product_completion_gap_contract.py",
    "_originweave_legacy_product_completion_gap_contract",
)
export_public(_legacy, globals())
