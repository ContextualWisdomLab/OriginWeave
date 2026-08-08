# ADR 0009: Hourly agent credential boundary

- **Status:** Proposed; becomes binding only after independent review and protected merge
- **Date:** 2026-08-08
- **Decision owners:** OriginWeave maintainers

## Context

OriginWeave's hourly product-development workflow can use NVIDIA-hosted model inference, but deterministic repository governance does not require model credentials. GitHub Actions exposes a secret to workflow code only when the workflow explicitly references that secret, and NVIDIA documents API-key authentication for calls to hosted NIM endpoints such as `integrate.api.nvidia.com`. The scheduler therefore needs a narrow boundary that prevents a model credential from becoming ambient authority for deterministic gates, the unprivileged model workspace, post-model validation, or pull-request publication.

The workflow also needs fail-closed network policy, deterministic early exits for existing work, a credential-free evidence path after model execution, and a publication identity that cannot review or merge its own proposal.

## Decision

### Deterministic gates run without the model credential

The `open_pull_request`, `release_blocker`, and `dry_run` decisions execute before any step references `NVIDIA_NIM_API_KEY`. If any of those gates stops development, the model-backed path does not receive the secret. A missing NVIDIA credential is evaluated only after those deterministic gates have selected the model-backed path.

### Required model credential fails closed

Once deterministic governance selects the model-backed path, `NVIDIA_NIM_API_KEY` is a required execution dependency rather than an optional optimization. If the credential is absent, the credential step records `nim_api_key_unavailable` in the job summary and exits nonzero. The workflow must not convert that missing authority into a green run merely because all model, bundle, and publication steps are conditionally skipped. This differs from `open_pull_request`, `release_blocker`, and `dry_run`, which are intentional deterministic safe-stop outcomes before model execution is selected.

### Raw credential authority is limited to two trusted steps

The raw `NVIDIA_NIM_API_KEY` is referenced only by:

1. the conditional credential step, which checks availability and derives a runner-owned leak-detection fingerprint; and
2. the root-run loopback credential broker, which injects the upstream credential into requests to the NVIDIA-hosted service.

The unprivileged OpenCode process never receives the raw key. It receives a synthetic local token and sends model requests only to the credential broker at `127.0.0.1:8765`. The broker alone translates those local requests into authenticated upstream requests.

### Egress remains fail closed

Harden Runner retains `egress-policy: block` with an explicit reviewed endpoint set. The model process runs as UID 65532 with operating-system egress restricted to loopback, so the runner-wide allowlist does not become direct model authority. The broker owns the upstream NVIDIA connection; no broad GitHub or Internet wildcard is introduced to make model execution succeed.

### Post-model validation receives a fingerprint, not the secret

While the raw credential is already authorized in the credential step, the workflow derives its length, SHA-256 digest, and a 64-bit rolling hash into a root-readable fingerprint file. The rolling hash is only a candidate-window prefilter; SHA-256 confirms exact byte equality before a leak is reported. The fingerprint file is deleted before candidate source and PR-message scanning begins. Post-model validation therefore detects an exact accidental credential disclosure without rematerializing `NVIDIA_NIM_API_KEY` into that step.

Untrusted `PR_MESSAGE.md` input is size-bounded before byte-wise fingerprint scanning, and changed source files remain subject to the existing per-file, file-count, changed-line, path, symlink, binary, and credential-disclosure bounds.

### Retry decisions require causal provider evidence

Each model attempt starts from a pristine archive of the exact protected source head. Local broker liveness and upstream provider viability are separate facts: a healthy `/healthz` response proves only that the loopback broker process is alive, not that NVIDIA will accept the credential or serve another model request.

The broker therefore exposes only bounded, credential-free `/statusz` counters. A monotonic request identifier is allocated when an upstream request begins, and the broker records the latest request identifiers that received authentication or authorization rejection (`401` or `403`) and rate limiting (`429`). The runner snapshots the request counter immediately before each model attempt. After a failed attempt, only provider outcomes whose request identifier is newer than that snapshot may influence that attempt's retry decision. This generation binding prevents a late response from an earlier model attempt from poisoning a later clean-room retry.

An observed `401` or `403` is classified as `provider_auth_rejected` and stops same-run cross-model fallback because another model using the same NVIDIA credential and provider cannot repair that authority failure. NVIDIA's own evaluator guidance treats authentication failures such as `401` and `403` as fail-immediately client errors. An observed `429` is classified as `provider_rate_limited` and also stops same-run cross-model fallback. NVIDIA documents `429` as normally retryable, but retryability does not imply that consuming another 35-minute model slot immediately on the same provider is feasible; the next independently revalidated scheduled invocation is the bounded retry boundary. If neither provider condition occurred, broker unavailability, bounded model timeout, and model/tool failure retain their existing classifications.

### Publication authority is separate

`OPENCODE_PR_TOKEN` is used only after a candidate bundle has been independently re-applied and verified and after live repository state is rechecked. That publication identity may create the branch and pull request but is not an approval or merge authority. The scheduler may not use publication credentials to satisfy independent review.

## Consequences

- Deterministic open-PR, release-blocker, and dry-run paths do not receive the NVIDIA model secret.
- After the model-backed path is selected, an absent NVIDIA credential fails the job instead of producing a successful no-op.
- The model has no raw NVIDIA credential, Git metadata, GitHub token, OIDC token, or direct non-loopback egress.
- A compromised or malformed candidate bundle cannot force post-model validation to receive the raw upstream credential.
- Broker failure is distinguishable from model failure and produces bounded diagnostics without exposing request bodies or credentials.
- Provider authentication rejection and rate limiting are distinguishable from model/tool failure without exposing response bodies, request paths, or credentials.
- Same-provider fallback stops when current-attempt evidence proves that another immediate model attempt is not feasible; transient rate limiting is re-evaluated by a later fresh scheduler invocation rather than by an in-run cooldown wait.
- Request-generation counters prevent predecessor-attempt provider outcomes from being attributed to a later pristine fallback.
- Fallback attempts are isolated from predecessor edits and cannot justify repeated, disproven repairs.
- Pull-request publication remains operationally separate from review and merge authority.
- The stricter boundary may stop development when credentials, provider viability, broker health, reviewer provisioning, or repository state are unavailable; that is intentional fail-closed behavior.

## Operational proof required after protected merge

The change is not considered operationally proven by pull-request CI alone. On the exact protected-main head, evidence must show all of the following:

1. with an open PR, the workflow exits through `open_pull_request` before credential materialization;
2. with no open PR or release blocker and not in `dry_run`, an absent `NVIDIA_NIM_API_KEY` fails at the credential step rather than ending green, while a valid credential reaches the conditional model path or a later explicit deterministic gate;
3. a controlled model failure records its cause, restores pristine source for any feasible next attempt, and stops immediately if the credential broker is unavailable;
4. controlled provider `401`/`403` evidence is attributed only to the current request generation and stops same-run fallback as `provider_auth_rejected`;
5. controlled provider `429` evidence is attributed only to the current request generation and stops same-run fallback as `provider_rate_limited`, leaving retry to a later fresh invocation;
6. fail-closed Harden Runner egress remains effective without adding unproved endpoints; and
7. publication, when reached, uses `OPENCODE_PR_TOKEN` only after exact bundle verification and live repository-state rechecks.

## Alternatives rejected

- **Inject `NVIDIA_NIM_API_KEY` at job or deterministic-gate scope.** Rejected because stopped deterministic paths would receive authority they do not need.
- **Treat a missing model credential as a successful skipped run.** Rejected because deterministic governance has already selected model-backed development; silently skipping every remaining step would create a false-green execution result and conceal an unavailable required dependency.
- **Give the raw key directly to OpenCode.** Rejected because model/tool execution is an untrusted boundary and does not need upstream credentials.
- **Rematerialize the raw key during bundle scanning.** Rejected because credential-free validation can use an exact one-way fingerprint instead.
- **Disable leak scanning.** Rejected because it weakens the security gate rather than repairing the credential boundary.
- **Use broker `/healthz` alone to authorize model fallback.** Rejected because process liveness does not prove provider authentication, authorization, or capacity.
- **Parse OpenCode prose or logs for provider HTTP status.** Rejected because free-form model/tool output is an unstable and untrusted control boundary; the trusted broker observes the authoritative upstream status directly.
- **Issue an extra provider preflight before every fallback.** Rejected because it consumes provider capacity and can disagree with the actual model request; generation-bound telemetry from the request already made is stronger evidence.
- **Retry `429` inside the same run after an arbitrary sleep.** Rejected because the hourly recurrence provides a bounded fresh retry with full state revalidation, while an in-run cooldown consumes finite verification budget without proving capacity has returned.
- **Broaden network egress to simplify retries.** Rejected because no repository-specific runtime evidence justifies a wider authority set.
- **Use `OPENCODE_PR_TOKEN` for approval or merge.** Rejected because publication and independent governance are intentionally separate authorities.

## References

GitHub. (2026). *Secrets*. GitHub Docs. https://docs.github.com/en/actions/concepts/security/secrets

GitHub. (2026). *Workflow syntax for GitHub Actions*. GitHub Docs. https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax

NVIDIA. (2026). *Authentication and API keys*. NVIDIA NeMo Retriever documentation. https://docs.nvidia.com/nemo/retriever/latest/extraction/ngc-api-key/index.html

NVIDIA. (2026). *Raise client error interceptor*. NVIDIA NeMo Evaluator SDK. https://docs.nvidia.com/nemo/evaluator/latest/libraries/nemo-evaluator/interceptors/raise-client-error.html

NVIDIA. (2026). *API reference: NVIDIA NIM for large language models*. NVIDIA Docs. https://docs.nvidia.com/nim/large-language-models/latest/api-reference.html