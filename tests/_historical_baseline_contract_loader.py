"""Load preserved historical baseline contracts against extracted evidence receipts."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from types import ModuleType

TESTS = Path(__file__).resolve().parent
ROOT = TESTS.parent
EVIDENCE = ROOT / "docs/evidence"
ARCHIVE = EVIDENCE / "product-technical-gap-baseline-through-2026-09-09.md"
ARCHIVE_CHANGELOG = EVIDENCE / "CHANGELOG-through-2026-09-09.md"
ARCHIVE_FITNESS = EVIDENCE / "DOCUMENTATION_FITNESS-through-2026-09-09.md"
ARCHIVE_MATURITY = EVIDENCE / "active-pr-maturity-through-2026-09-09.md"
ARCHIVE_AGENTS = EVIDENCE / "AGENTS-through-2026-09-09.md"
ARCHIVE_EVIDENCE_SCRIPT = EVIDENCE / "collect-live-merge-evidence-through-2026-09-09.sh"
ARCHIVE_ROOT = EVIDENCE / "historical-contract-root-through-2026-09-09"


def load_historical_contract(filename: str, module_name: str) -> ModuleType:
    """Load one legacy module against immutable predecessor-bound evidence inputs."""
    path = TESTS / filename
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load historical contract: {filename}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.BASELINE = ARCHIVE
    module.ROOT = ARCHIVE_ROOT
    if hasattr(module, "CHANGELOG"):
        module.CHANGELOG = ARCHIVE_CHANGELOG
    if hasattr(module, "FITNESS"):
        module.FITNESS = ARCHIVE_FITNESS
    if hasattr(module, "MATURITY"):
        module.MATURITY = ARCHIVE_MATURITY
    if hasattr(module, "AGENTS"):
        module.AGENTS = ARCHIVE_AGENTS
    if hasattr(module, "EVIDENCE_SCRIPT"):
        module.EVIDENCE_SCRIPT = ARCHIVE_EVIDENCE_SCRIPT
    return module


def export_public(module: ModuleType, namespace: dict[str, object]) -> None:
    """Expose the preserved test classes/helpers through the original discovery module."""
    for name in dir(module):
        if not name.startswith("_"):
            namespace[name] = getattr(module, name)
