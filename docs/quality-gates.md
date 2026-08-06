# Quality and Release Gates

## Pull-request gates

A PR is mergeable only when all apply:

- one coherent scope and current architecture alignment;
- realistic failing test observed before implementation;
- complete focused and full verification on the exact head;
- production function, line, region, and branch coverage each at 100%;
- all public Rust APIs documented and rustdoc warnings denied;
- format, check, test, Clippy, and documentation jobs pass;
- dependency and GitHub Action references are locked or commit-pinned;
- generated build outputs are ignored and absent from `git ls-files`;
- review threads are resolved with code or evidence;
- required independent approval and repository security checks pass;
- README, architecture, ADR, doctoring, and changelog are updated when affected.

## Safety-kernel gates

A policy, evidence, or resource change additionally requires:

- browser-special numeric-host and canonical-origin equivalence tests;
- R3 and R4 approval tests bound to action kind, target origin, and the complete canonical intent digest;
- proof that generic header and query values are universally redacted before evidence leaves the trusted boundary;
- exact field-count, name, value, path, source URL, and locator boundary tests;
- malformed percent-escape, encoded separator, and literal or encoded dot-segment cases;
- simultaneous RAM, VRAM, frame, batch, model-residency, and admission-pressure tests;
- proof that crossing a hard memory limit reduces the active consumer and rejects new work.

## Browser vertical-slice gates

A browser or scraping feature additionally requires:

- isolated profile and origin capability tests;
- stale document-epoch and navigation tests;
- post-condition verification rather than command-return success;
- prompt-injection and hidden-content cases;
- secret non-disclosure in prompts, traces, logs, and provenance;
- Chromium crash/restart and task-checkpoint recovery;
- bounded request, response, snapshot, download, and artifact sizes;
- DNS-resolution, rebinding, redirect, proxy, private-address, link-local, metadata-endpoint, and partial-connection tests;
- MIME and declared-versus-observed content validation before persistence;
- keyboard and screen-reader-compatible approval and evidence UI;
- repeatable real-site or controlled-browser task benchmarks.

## Performance evidence

Report distributions, not isolated best runs:

- input and action latency;
- compositor frame time and dropped frames;
- process and task peak RSS;
- JavaScript heap and semantic snapshot bytes;
- peak VRAM and CPU fallback reason;
- bytes transferred and disk spill;
- token use and model calls per task;
- task success, unauthorized-action rate, stale-node rate, and recovery success.

## Release gates

A supported release requires:

- explicit semantic version and dated `CHANGELOG.md` section;
- all default-branch checks green after merge;
- SBOM, source and build provenance, checksums, and signed/attested artifacts;
- reproducible build evidence;
- supported-platform compatibility matrix;
- Manifest V3 compatibility evidence for declared extension APIs;
- security threat model and external review appropriate to the release scope;
- privacy, retention, telemetry, upgrade, rollback, and incident procedures;
- documented support and vulnerability-response policy.

No workflow, issue label, test name, or configuration string can substitute for this evidence.
