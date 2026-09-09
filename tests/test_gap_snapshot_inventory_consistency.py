"""Discovery entry point for preserved gap-snapshot history contracts."""

from _historical_baseline_contract_loader import export_public, load_historical_contract

_legacy = load_historical_contract(
    "legacy_gap_snapshot_inventory_consistency.py",
    "_originweave_legacy_gap_snapshot_inventory_consistency",
)
export_public(_legacy, globals())
