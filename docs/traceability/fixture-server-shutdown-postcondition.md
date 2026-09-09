# Fixture-server shutdown post-condition

## Problem

The controlled MV3 and Agent Task browser lanes own two loopback `ThreadingHTTPServer` instances. `_stop_fixture_server` previously called `shutdown()`, `server_close()`, and `thread.join(timeout=5)` and then returned without observing whether the helper thread had actually terminated. A timed `join` is only a bounded wait; it does not itself prove the target thread stopped. That left cleanup evidence vulnerable to the same acknowledgement-versus-post-condition error that the browser action lane already rejects.

## Constraints

- Keep both fixture servers loopback-only and preserve the existing reverse-order cleanup attempt.
- Do not add retries, unbounded waits, process-wide thread manipulation, or workflow changes.
- Do not weaken the three-trial browser evidence denominator or sandbox requirements.
- Do not move WebDriver/ChromeDriver protocol diagnostic authority out of PR #148 or workflow/sandbox authority out of issue #212.
- Cleanup diagnostics must remain stable and must not include filesystem paths, page content, credentials, or remote-driver text.

## Decision

After the existing five-second `join`, `_stop_fixture_server` now checks `thread.is_alive()`. If the helper is still alive, cleanup fails closed with the fixed diagnostic `fixture server thread did not stop`.

A focused regression uses a stalled thread double to prove that `shutdown()`, `server_close()`, and the bounded join are attempted but are not accepted as successful cleanup unless liveness becomes false. A companion success case proves that an observed stopped thread is accepted.

## Alternatives rejected

An unbounded `join()` was rejected because a failed fixture server could hang the CI lane indefinitely. Repeating `shutdown()` or sleeping before a second join was rejected because it would obscure the causal cleanup defect and add timing-dependent behavior. Silently recording the thread as cleaned after a timed join was rejected because command completion is not a post-condition.

## Risk and effect

The new check can turn a previously silent helper-thread leak into an explicit test failure. That is intentional: a browser evidence run is not complete while its owned fixture service remains live. The change does not alter browser navigation, semantic observation, native interaction, sandbox configuration, profile cleanup, workflow topology, or trial counts.

## Exact evidence

- Test-first commit: `586bf780a6ebeb565feb0e5325afae3053937496`.
- Minimal production repair: `aed62d356721ee5abf3cc84414c87bea15eaa09e`.
- Compare from prior checkpoint `037d2fc45fba99c0c375be4f8431df44bdcc94f7` is two ordinary commits ahead and zero behind; before this documentation commit, the only changed paths are `tests/test_fixture_server_shutdown_postcondition_contract.py` and `scripts/ci/run_mv3_compatibility.py`, with the production delta limited to two added lines.

## Follow-up acceptance

Keep PR #288 Draft until exact-head repository gates execute and issue #212 supplies the authorized current-generation sandbox-helper workflow. The pinned Chrome/ChromeDriver `150.0.7871.129` replay must still prove the existing browser surfaces and cleanup. Draft-policy skips are not executable GREEN.
