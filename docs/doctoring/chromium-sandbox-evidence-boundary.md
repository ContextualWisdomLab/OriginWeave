# Chromium sandbox evidence boundary

Status: active verification evidence for PR #288. This note does not claim protected-main delivery or a passing browser gate.

## Problem

OriginWeave uses pinned real Chromium as compatibility and Agent Task evidence. A browser test that succeeds only after disabling Chromium process sandboxing does not prove the security posture expected from the governed-browser runtime. The transferred PR #288 generation originally inherited two real-browser launch paths in `scripts/ci/run_mv3_compatibility.py`: the Agent Task path kept sandboxing enabled, while the ordinary Manifest V3 compatibility path passed `--no-sandbox`. That source defect is repaired: both ordinary Manifest V3 and Agent Task real-browser paths omit `--no-sandbox`.

Chromium's current Linux security guidance states that `--no-sandbox` disables critical security features and recommends installing/configuring a sandbox helper for developer builds instead. Chromium's Linux debugging guidance likewise says sandbox testing is needed on automated waterfall bots rather than routinely running them without the sandbox. The SUID sandbox development guidance documents the helper ownership/mode and `CHROME_DEVEL_SANDBOX` setup used when the normal user-namespace sandbox is unavailable.

## Constraints and rejected alternatives

The scheduled OriginWeave product writer does not own `.github/**`; workflow setup belongs to issue #212. Disabling Ubuntu/AppArmor restrictions runner-wide, retaining `--no-sandbox`, reducing browser trials, or treating ChromeDriver command acknowledgement as product success would weaken the evidence boundary and is rejected.

Copying the complete #43 runner is also rejected. #43 contains unrelated downloads and diagnostic work. The reviewed causal precedent is narrower: commit `a45c83e4d8988fe89920ecb6a9eac469815f5b9b` removes the single `--no-sandbox` launch override and records that a sandbox-incompatible environment must fail instead of weakening Chromium isolation.

## Selected repair path

1. Keep `tests/test_mv3_browser_sandbox_contract.py` as the product-side invariant for both ordinary MV3 and Agent Task real-browser paths, including the documentation/source consistency guard.
2. Preserve the repaired ordinary `_run_browser_pass` launch without `--no-sandbox`; do not import unrelated #43 product delta.
3. Let #212 own the canonical workflow helper setup needed by the pinned Chrome for Testing archive, including root ownership/mode and environment wiring when the chosen Linux sandbox requires it.
4. Run the full repository contract suite on the repaired exact head, then execute fresh pinned-Chromium compatibility and Agent Task trials with sandboxing enabled. A skipped Draft job, command ACK, mock-fixture JSON, source inspection, or predecessor run is not GREEN browser evidence.

## Exact active evidence

- Protected base when the regression was recorded: `87c4daa1830bac5a5228b6036752ad5633232085`.
- Test-first RED generation for the launch defect: `101470a19b370bd30533ab3db330a882a2c25bc3`.
- Product repair commit: `6c072c3cd035319a57977d012007898c29602402`, which removes the ordinary `--no-sandbox` override.
- Diagnostic-restoration/source-verification generation: `0f434bc29d468127412366b6864b733b40b83c4d`; both real-browser option lists omit the forbidden override there. Its reported 173 Python/full-Rust/numeric-coverage verification is source/test evidence, not an actual Chromium execution; the `agent_task` JSON produced by the tests is a mock fixture.
- Documentation-consistency RED commit: `e74acd7b83aac2c7768ba21a2a9b5d9ea2b8a31b`; the new focused contract rejects the stale current-tense `--no-sandbox` statement that this revision repairs.
- Reviewed causal precedent: #43 commit `a45c83e4d8988fe89920ecb6a9eac469815f5b9b`.
- Canonical workflow-owner path: issue #212.
- Prior Agent Task hosted RED remains #70 MV3 run `33887386759`, job `101070423144`; it is predecessor evidence only and does not establish the current head as GREEN.

## References

The Chromium Authors. (2026). *AppArmor user namespace restrictions vs. Chromium developer builds*. Chromium source documentation. https://chromium.googlesource.com/chromium/src/+/main/docs/security/apparmor-userns-restrictions.md

The Chromium Authors. (2026). *Linux SUID sandbox development*. Chromium source documentation. https://chromium.googlesource.com/chromium/src/+/main/docs/linux/suid_sandbox_development.md

The Chromium Authors. (2026). *Tips for debugging on Linux*. Chromium source documentation. https://chromium.googlesource.com/chromium/src/+/main/docs/linux_debugging.md
