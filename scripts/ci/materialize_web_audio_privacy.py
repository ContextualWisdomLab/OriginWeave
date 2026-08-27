#!/usr/bin/env python3
"""Materialize the reviewed Web Audio privacy slice and delete this helper."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(relative: str, old: str, new: str) -> None:
    """Replace one exact repository anchor and fail closed on drift."""

    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one anchor in {relative}, found {count}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "crates/originweave-fingerprint/src/lib.rs",
    "mod coherence;\nmod stealth;\nmod ua_hints;",
    "mod coherence;\nmod stealth;\nmod ua_hints;\nmod web_audio_guard;",
)
replace_once(
    "crates/originweave-fingerprint/src/lib.rs",
    "pub use ua_hints::{\n    ClientHintsError, HintsArchitecture, HintsBitness, HintsPlatform, UaBrand, UaClientHints,\n};",
    "pub use ua_hints::{\n    ClientHintsError, HintsArchitecture, HintsBitness, HintsPlatform, UaBrand, UaClientHints,\n};\npub use web_audio_guard::{\n    WebAudioDecision, WebAudioFingerprintPolicy, WebAudioPolicyError,\n};",
)
replace_once(
    "crates/originweave-fingerprint/Cargo.toml",
    "[dependencies]\nsha2 = \"=0.10.9\"",
    "[dependencies]\noriginweave-core = { path = \"../originweave-core\" }\nsha2 = \"=0.10.9\"",
)
replace_once(
    "Cargo.lock",
    'name = "originweave-fingerprint"\nversion = "0.1.0"\ndependencies = [\n "sha2",\n]',
    'name = "originweave-fingerprint"\nversion = "0.1.0"\ndependencies = [\n "originweave-core",\n "sha2",\n]',
)
replace_once(
    ".github/workflows/mv3-compatibility.yml",
    '      - "crates/originweave-core/**"\n      - "scripts/ci/run_mv3_compatibility.py"',
    '      - "crates/originweave-core/**"\n      - "crates/originweave-fingerprint/**"\n      - "extensions/originweave-privacy-guard/**"\n      - "scripts/ci/run_mv3_compatibility.py"\n      - "scripts/ci/run_web_audio_privacy.py"',
)
replace_once(
    ".github/workflows/mv3-compatibility.yml",
    '      - "tests/fixtures/mv3_basic/**"\n      - "tests/test_mv3_compatibility_contract.py"\n      - "docs/doctoring/mv3-compatibility.md"',
    '      - "tests/fixtures/mv3_basic/**"\n      - "tests/fixtures/web_audio_privacy/**"\n      - "tests/test_mv3_compatibility_contract.py"\n      - "tests/test_web_audio_privacy_contract.py"\n      - "docs/adr/0114-default-deny-web-audio-fingerprinting.md"\n      - "docs/doctoring/mv3-compatibility.md"\n      - "docs/doctoring/web-audio-privacy.md"',
)
replace_once(
    ".github/workflows/mv3-compatibility.yml",
    '          python3 scripts/ci/run_mv3_compatibility.py | tee mv3-compatibility.json',
    '          python3 scripts/ci/run_mv3_compatibility.py | tee mv3-compatibility.json\n          python3 scripts/ci/run_web_audio_privacy.py | tee web-audio-privacy.json',
)
replace_once(
    ".github/workflows/mv3-compatibility.yml",
    '            mv3-compatibility.json',
    '            mv3-compatibility.json\n            web-audio-privacy.json',
)
replace_once(
    "docs/adr/README.md",
    "| [0113](0113-cross-surface-platform-coherence.md) | Cross-surface platform coherence | Proposed | presentation-platform, UA-token, and UA-CH-platform triad agreement |",
    "| [0113](0113-cross-surface-platform-coherence.md) | Cross-surface platform coherence | Proposed | presentation-platform, UA-token, and UA-CH-platform triad agreement |\n| [0114](0114-default-deny-web-audio-fingerprinting.md) | Default-deny Web Audio fingerprinting | Proposed | exact-origin Web Audio construction authority and pre-document privacy guard |",
)
replace_once(
    "docs/README.md",
    "- [ADR 0113: Cross-surface platform coherence](adr/0113-cross-surface-platform-coherence.md)",
    "- [ADR 0113: Cross-surface platform coherence](adr/0113-cross-surface-platform-coherence.md)\n- [ADR 0114: Default-deny Web Audio fingerprinting](adr/0114-default-deny-web-audio-fingerprinting.md)",
)
replace_once(
    "CHANGELOG.md",
    "### Added\n",
    "### Added\n- Added a default-deny Web Audio fingerprinting boundary for isolated Agent and Crawler profiles: exact-origin grants capped at 128 unique canonical origins, a deterministic Rust-rendered MAIN-world `document_start` guard, and a pinned-Chromium top-document/child-frame proof that blocks online, offline, prefixed, and AudioWorklet construction entry points (see ADR 0114).\n",
)
replace_once(
    "CHANGELOG.md",
    "### Security\n",
    "### Security\n- Web Audio constructors now fail with a fixed `NotAllowedError` in the managed default profile unless a trusted policy grants the exact canonical origin; the guard has no storage, network, messaging, model, or secret authority and does not affect ordinary media-element playback.\n",
)
replace_once(
    "docs/product-technical-gap-baseline.md",
    "| Cross-surface platform coherence | stacked `feat/stealth-profile-coherence` on #234 | Proposed ADR 0113 binds the presentation-platform, UA-token, and UA-CH-platform triad through `PresentationPlatform::hints_platform` and `require_hints_coherence`; control-plane contract only, no browser claim |",
    "| Cross-surface platform coherence | stacked `feat/stealth-profile-coherence` on #234 | Proposed ADR 0113 binds the presentation-platform, UA-token, and UA-CH-platform triad through `PresentationPlatform::hints_platform` and `require_hints_coherence`; control-plane contract only, no browser claim |\n| Web Audio privacy guard | draft #236 stacked on #235 | Proposed ADR 0114 adds a default-deny exact-origin Web Audio policy, reviewed MV3 MAIN-world `document_start` guard, and pinned-Chromium top-document/child-frame evidence; it remains active-PR evidence until prerequisites, exact-head checks, independent review, protected-main integration, profile-builder binding, and signed release are complete |",
)

for required in (
    "crates/originweave-fingerprint/src/web_audio_guard.rs",
    "extensions/originweave-privacy-guard/manifest.json",
    "extensions/originweave-privacy-guard/web_audio_guard.js",
    "scripts/ci/run_web_audio_privacy.py",
    "tests/test_web_audio_privacy_contract.py",
    "docs/adr/0114-default-deny-web-audio-fingerprinting.md",
    "docs/doctoring/web-audio-privacy.md",
):
    if not (ROOT / required).is_file():
        raise RuntimeError(f"required materialized file missing: {required}")

(ROOT / "scripts/ci/materialize_web_audio_privacy.py").unlink()
(ROOT / ".github/workflows/one-shot-web-audio-privacy.yml").unlink()
