# Fixture-server shutdown post-condition

## Problem

The controlled MV3 and Agent Task browser lanes own two loopback `ThreadingHTTPServer` instances. `_stop_fixture_server` previously called `shutdown()`, `server_close()`, and `thread.join(timeout=5)` and then returned without observing whether the helper thread had actually terminated. A timed `join` is only a bounded wait; it does not itself prove the target thread stopped. That left cleanup evidence vulnerable to the same acknowledgement-versus-post-condition error that the browser action lane already rejects.

A second evidence-ordering defect remained after the liveness repair: `main()` serialized and printed a success-shaped compatibility document before either fixture server entered its shutdown `finally` block. A later cleanup failure therefore produced a non-zero process result but could leave an already-emitted JSON document that looked like completed browser acceptance. For evidence consumers, successful publication must be downstream of owned teardown rather than merely adjacent to it.

## Constraints

- Keep both fixture servers loopback-only and preserve the existing reverse-order cleanup attempt.
- Do not add retries, unbounded waits, process-wide thread manipulation, or workflow changes.
- Do not weaken the three-trial browser evidence denominator or sandbox requirements.
- Do not move WebDriver/ChromeDriver protocol diagnostic authority out of PR #148 or workflow/sandbox authority out of issue #212.
- Cleanup diagnostics must remain stable and must not include filesystem paths, page content, credentials, or remote-driver text.
- Preserve bounded JSON evidence for a browser/trial gate failure even though successful publication moves behind cleanup.

## Decision

After the existing five-second `join`, `_stop_fixture_server` checks `thread.is_alive()`. If the helper is still alive, cleanup fails closed with the fixed diagnostic `fixture server thread did not stop`.

A focused regression uses a stalled thread double to prove that `shutdown()`, `server_close()`, and the bounded join are attempted but are not accepted as successful cleanup unless liveness becomes false. A companion success case proves that an observed stopped thread is accepted.

Successful compatibility JSON is now emitted only after both reverse-order fixture shutdown calls return successfully. Gate-failure paths still emit their bounded evidence immediately before raising, so the diagnostic artifact for an unsuccessful browser/trial run is not lost. If a run otherwise satisfies the browser gates but either owned fixture server fails cleanup, no success-shaped JSON is published. The success-path `duration_ms` is refreshed after teardown so its measured interval includes the owned fixture-server shutdown boundary.

This follows Python's documented `Thread.join(timeout)` contract: `join()` returns `None` whether the target terminated or the timeout expired, so callers must inspect `is_alive()` after a timed join to determine whether the timeout occurred.

## Alternatives rejected

An unbounded `join()` was rejected because a failed fixture server could hang the CI lane indefinitely. Repeating `shutdown()` or sleeping before a second join was rejected because it would obscure the causal cleanup defect and add timing-dependent behavior. Silently recording the thread as cleaned after a timed join was rejected because command completion is not a post-condition.

Publishing success JSON before cleanup and relying only on the process exit code was rejected because the artifact itself is a first-class evidence surface and may be retained or inspected separately from the runner status. Suppressing all pre-cleanup JSON was also rejected because failed browser/trial gates still need bounded diagnostic evidence. The chosen ordering distinguishes unsuccessful diagnostic publication from successful acceptance publication without changing browser behavior.

## Risk and effect

The liveness check can turn a previously silent helper-thread leak into an explicit test failure. The publication-order repair can remove a success-shaped JSON artifact from runs that would previously have printed it and then failed during teardown. Both effects are intentional: a browser evidence run is not complete while an owned fixture service remains live. The changes do not alter browser navigation, semantic observation, native interaction, sandbox configuration, profile cleanup, workflow topology, or trial counts.

## Exact evidence

- Fixture liveness test-first commit: `586bf780a6ebeb565feb0e5325afae3053937496`.
- Fixture liveness minimal production repair: `aed62d356721ee5abf3cc84414c87bea15eaa09e`.
- Success-publication test-first commit: `2c7e5df3c13229614419733491cdf85b0d1838eb`.
- Success-publication minimal production repair: `d3de2a26c51f2d0dbf2362c8da36330cf5b2b443`.
- Review finding anchoring the publication defect: `5148970300` on exact predecessor `63d57081cb591a93661e483a9fb7b7712adb3b2d`.
- The new publication regression proves three boundaries: successful evidence follows both fixture stops; a cleanup failure leaves no success-shaped evidence; and a browser/trial gate failure retains bounded evidence before raising.

## Primary runtime reference

Python Software Foundation. (2026). *threading — Thread-based parallelism* (Python 3.14.7 documentation). https://docs.python.org/3/library/threading.html#threading.Thread.join

## Follow-up acceptance

Keep PR #288 Draft until exact-head repository gates execute and issue #212 supplies the authorized current-generation sandbox-helper workflow. The pinned Chrome/ChromeDriver `150.0.7871.129` replay must still prove the existing browser surfaces and cleanup. Successful JSON publication is now downstream of fixture-server teardown, but Draft-policy skips remain non-evidence and are not executable GREEN.