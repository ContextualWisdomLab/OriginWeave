# Manifest V3 compatibility evidence baseline

- **Status:** Active implementation evidence for issue #27
- **Reviewed:** 2026-08-16
- **Pinned browser:** Chrome for Testing `150.0.7871.129`, Chromium revision `r1639810`

OriginWeave uses Chromium as its compatibility kernel, so browser-extension compatibility must be demonstrated with executable Chromium evidence rather than inferred from architecture alone. The protected-main lane exercises a controlled unpacked Manifest V3 extension against one exact Chrome for Testing build and proves service-worker, content-script, storage, declarative-network-request, tabs, windows, scripting, commands, side-panel, bookmarks/history read compatibility, restart persistence, repeatability, and one real WebDriver click/post-condition. Active stacked compatibility work adds downloads, bounded bookmark/history mutation, profile isolation, explicit extension update/version-migration evidence, and an exact content-script isolated-world check. OriginWeave does **not claim 100% Chrome extension compatibility**.

The checked-in fixture is intentionally local-only. Its host permission is limited to loopback HTTP used by the deterministic test server. It contains no remote code, user credential, model call, external content, native-messaging host, or production PII. Chrome permissions remain distinct from the explicit OriginWeave extension-to-Agent grant implemented in `originweave-core`. Compatibility mutation tests create only controlled synthetic state inside the ephemeral test profile and must clean it up; successful API compatibility never grants the OriginWeave Agent ambient bookmarks/history/downloads authority.

## Supported-capability evidence matrix

This matrix separates protected-main executable evidence from active, non-shipped evidence and from genuinely unproven surfaces. A row marked **ACTIVE_PR** is never a release claim; exact head/run provenance belongs in `docs/evidence/2026-08-10-active-pr-maturity.md` and must be refreshed when the branch changes.

| Compatibility surface | Evidence maturity | Current evidence boundary | Known gap / non-claim |
|---|---|---|---|
| Manifest V3 unpacked extension load | **PROTECTED_MAIN** | Exact pinned Chromium fixture loads through the dedicated compatibility workflow. | No Chrome Web Store distribution or arbitrary third-party extension-install claim. |
| Service worker start/restart + event response | **PROTECTED_MAIN** | Worker startup count and message response are observed across a real browser restart. | Suspend timing and the full Chrome event catalog are not exhaustively covered. |
| Content-script injection | **PROTECTED_MAIN** | Controlled content script mutates bounded DOM evidence on loopback. | Injection alone does not prove JavaScript isolated-world semantics. |
| Content-script isolated-world separation | **ACTIVE_PR #61** | Page main-world and extension isolated-world JavaScript assign the same sentinel name to distinct values; compatibility reports ready only while the page still reads `page` and the content script reads `extension` in real pinned Chromium. | One deterministic fixture proof only; no arbitrary page-JavaScript bridge or Agent authority. |
| `chrome.storage.local` + restart persistence | **PROTECTED_MAIN** | State is initialized on the first browser pass and required to persist on restart. | No OriginWeave-owned durable application database is implied. |
| `declarativeNetRequest` | **PROTECTED_MAIN** | Controlled local rule blocks its fixture request in pinned Chromium. | No claim for every DNR rule/action combination. |
| `tabs`, `windows`, `scripting`, `commands`, `sidePanel` | **PROTECTED_MAIN** | Each declared API is exercised in real Chromium and required by the repeatability gate. | Chrome API permission does not become Agent capability. |
| Bookmarks read compatibility | **PROTECTED_MAIN** | Protected-main fixture exercises the declared bookmarks surface. | Ambient human-profile bookmark authority is not granted. |
| Bookmarks create/read/delete lifecycle | **ACTIVE_PR #56** | Controlled synthetic bookmark is created, read back, and removed in the ephemeral compatibility profile. | Compatibility only; no Agent bookmark capability. |
| History read compatibility | **PROTECTED_MAIN** | Protected-main fixture exercises bounded history search in the isolated profile. | No model-visible browsing-history content or default-profile access. |
| History add/read/delete lifecycle | **ACTIVE_PR #59** | Controlled synthetic loopback visit is added, exactly read back, deleted in `finally`, and required to be absent afterward. | Compatibility only; no Agent history capability. |
| Downloads | **ACTIVE_PR #43** | Controlled loopback payload is downloaded and validated through pinned Chromium. | No general download persistence, unsafe filename, or Agent filesystem authority claim. |
| Per-trial Agent Task profile isolation | **ACTIVE_PR #49** | Compatibility trials use isolated ephemeral profiles rather than ambient human state. | Full production Agent Task browser orchestration remains issue #28 work. |
| Extension update/version migration | **ACTIVE_PR #60** | Trial-local extension copy transitions `1.0.0` → `1.0.1` on the same ephemeral profile; versioned storage state is required to migrate and real pinned-Chromium evidence reports the update-migration surface. | No Chrome Web Store updater, enterprise deployment channel, arbitrary downgrade, or protected-main release claim. |
| Managed enterprise extension policy | **PLANNED** | No protected-main executable compatibility proof yet. | Do not infer managed-policy support from Chromium ancestry alone. |
| Native messaging | **PLANNED / SECURITY-GATED** | No compatibility claim. | Future support requires an explicit host-managed allow-list and process boundary. |
| Google-only services, proprietary codecs, DRM, Web Store licensing | **OUT_OF_SCOPE FOR COMPATIBILITY CLAIM** | Deliberately excluded from the open compatibility claim. | Chromium/API compatibility must not be conflated with Google service or licensing equivalence. |

The release-quality capability matrix must remain coupled to executable evidence. Adding a row to documentation never creates support; declaring a new supported capability must first add a realistic regression test and pinned-Chromium proof. Conversely, if a declared protected-main capability regresses, the release gate must fail rather than silently downgrading the matrix.

## History API primary evidence

For history compatibility specifically, the current official Chrome Extensions API documents the `history` manifest permission and Promise-returning `chrome.history.addUrl`, `chrome.history.search`, and `chrome.history.deleteUrl` methods. This living vendor reference establishes API semantics only. OriginWeave release evidence continues to depend on the exact pinned Chromium fixture and exact-head CI result rather than inferring compatibility from documentation.

## Downloads API primary evidence

For downloads compatibility specifically, the current official Chrome Extensions API documents the `downloads` manifest permission and the `chrome.downloads` methods that initiate, monitor, search, and inspect downloads. This living vendor reference establishes API semantics only. Active PR #43 exercises one controlled loopback payload through pinned Chromium and retains only allow-listed stage diagnostics. That proof is not Agent filesystem authority, general download persistence, unsafe-filename handling, or a release claim that every `chrome.downloads` method works.

## Update-migration evidence boundary

Restart persistence and extension update migration are separate compatibility claims. A successful restart proves only that state survives a new browser process. The active update-migration lane additionally uses a trial-local copy of the checked-in fixture, preserves the same extension path and ephemeral profile across passes, changes only the controlled manifest version from `1.0.0` to `1.0.1`, observes `chrome.runtime.getManifest().version`, and requires the fixture schema marker to migrate from version 1 to version 2. The checked-in fixture is not rewritten by the test. This establishes one deterministic unpacked-extension version transition; it does not establish Chrome Web Store update behavior, enterprise rollout semantics, downgrade behavior, or arbitrary third-party extension migration safety.

## Isolated-world evidence boundary

Content-script injection and content-script JavaScript isolation are separate compatibility claims. Active PR #61 writes `window.originweaveWorldSentinel = "page"` in the fixture page's main world and repeatedly publishes that value through one controlled DOM attribute. The content script assigns the same global name to `"extension"` in its own execution world, waits a bounded interval, and only reports the existing compatibility surface ready when it simultaneously observes the page's published `page` value and its own `extension` value. If both scripts share one JavaScript global namespace, the page publisher changes to `extension` and real-browser compatibility fails. DOM sharing here is deliberate test evidence, not permission for arbitrary page content to become trusted instruction or Agent authority.

## Supply-chain and repeatability evidence

The CI lane downloads the exact Chrome/ChromeDriver version from the official Chrome for Testing public bucket, records SHA-256 receipts for the downloaded archives, verifies the runtime-reported browser version, and emits bounded JSON compatibility evidence. A future release-quality matrix should additionally pin published artifact digests or equivalent immutable supply-chain identity when the upstream distribution exposes that identity in an authoritative machine-readable form.

## Primary references — APA 7th

Chrome for Developers. (n.d.). *Extensions / Manifest V3*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3

Chrome for Developers. (2023, May 2). *Extension service worker basics*. Google. https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/basics

Chrome for Developers. (2023, May 2). *The extension service worker lifecycle*. Google. https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle

Chrome for Developers. (n.d.). *chrome.declarativeNetRequest*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest

Chrome for Developers. (n.d.). *chrome.downloads*. Google. Retrieved August 16, 2026, from https://developer.chrome.com/docs/extensions/reference/api/downloads

Chrome for Developers. (n.d.). *chrome.history*. Google. Retrieved August 11, 2026, from https://developer.chrome.com/docs/extensions/reference/api/history

Chrome for Developers. (n.d.). *Manifest file format*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/reference/manifest

Bynens, M. (2023, June 12). *Chrome for Testing*. Chrome for Developers. https://developer.chrome.com/docs/automation-and-testing/chrome-for-testing

Google Chrome Labs. (2026, July 21). *Chrome for Testing availability*. https://googlechromelabs.github.io/chrome-for-testing/
