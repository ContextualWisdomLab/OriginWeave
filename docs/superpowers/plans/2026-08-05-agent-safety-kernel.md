# OriginWeave Agent Safety Kernel Implementation Plan

## Completed sequence

1. Bootstrap a Rust 2024 workspace and pin Rust 1.97.1.
2. Add placeholder crates for core, policy, resource, and evidence boundaries.
3. Write integration tests before public APIs exist and confirm the expected compile failure.
4. Implement normalized origins and typed governance contracts.
5. Implement deterministic fail-closed policy evaluation.
6. Implement validated resource budgets and interactive-first directives.
7. Implement redacted network evidence and provenance validation.
8. Enforce format, check, tests, strict Clippy, and rustdoc warnings.
9. Add a pinned-nightly exact branch-coverage lane while retaining Rust 1.97.1 as the supported compiler.
10. Add architecture records, research references, roadmap, security, contribution, and release documentation.

## Merge checklist

- [ ] production functions, lines, regions, and branches are each 100%;
- [ ] all exact-head CI, SAST, and security checks pass;
- [ ] no unresolved review threads remain;
- [ ] required independent approval is present;
- [ ] PR description reflects the final code and residual risks;
- [ ] draft status is removed only after the preceding evidence exists.

## Next plan after merge

Implement one isolated Chromium vertical slice: create an ephemeral user context, navigate, collect an accessibility-tree observation, assign a document epoch, execute one typed click after policy approval, verify a deterministic post-condition, and emit an evidence bundle. This requires a separate approved design and implementation plan.
