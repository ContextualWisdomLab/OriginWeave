"""Discovery entry point for preserved live-gap evidence-integrity contracts."""

from _historical_baseline_contract_loader import export_public, load_historical_contract

_legacy = load_historical_contract(
    "legacy_live_gap_evidence_integrity_contract.py",
    "_originweave_legacy_live_gap_evidence_integrity_contract",
)
export_public(_legacy, globals())
