"""Load preserved historical baseline contracts against extracted evidence receipts."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from types import ModuleType

TESTS = Path(__file__).resolve().parent
ROOT = TESTS.parent
ARCHIVE = ROOT / "docs/evidence/product-technical-gap-baseline-through-2026-09-09.md"
ARCHIVE_CHANGELOG = ROOT / "docs/evidence/CHANGELOG-through-2026-09-09.md"


def load_historical_contract(filename: str, module_name: str) -> ModuleType:
    """Load one legacy module against immutable baseline/changelog evidence inputs."""
    path = TESTS / filename
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load historical contract: {filename}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.BASELINE = ARCHIVE
    if hasattr(module, "CHANGELOG"):
        module.CHANGELOG = ARCHIVE_CHANGELOG
    return module


def export_public(module: ModuleType, namespace: dict[str, object]) -> None:
    """Expose the preserved test classes/helpers through the original discovery module."""
    for name in dir(module):
        if not name.startswith("_"):
            namespace[name] = getattr(module, name)
