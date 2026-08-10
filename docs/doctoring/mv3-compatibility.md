# Manifest V3 compatibility evidence baseline

- **Status:** Active implementation evidence for issue #27
- **Reviewed:** 2026-08-11
- **Pinned browser:** Chrome for Testing `150.0.7871.129`, Chromium revision `r1639810`

OriginWeave uses Chromium as its compatibility kernel, so browser-extension compatibility must be demonstrated with executable Chromium evidence rather than inferred from architecture alone. This first bounded lane exercises a controlled unpacked Manifest V3 extension against one exact Chrome for Testing build. It covers an extension service worker, content-script injection, `chrome.storage.local`, declarative network blocking, and one real WebDriver click/post-condition. Active stacked compatibility work additionally exercises declared Chrome API surfaces such as downloads, bookmark mutation, and controlled history mutation. It does **not claim 100% Chrome extension compatibility** and does not make claims about Chrome Web Store distribution, Google-only services, proprietary codecs, DRM, native messaging, enterprise policy, restart/update migration, or every Chrome extension API.

The checked-in fixture is intentionally local-only. Its host permission is limited to loopback HTTP used by the deterministic test server. It contains no remote code, user credential, model call, external content, native-messaging host, or production PII. Chrome permissions remain distinct from the explicit OriginWeave extension-to-Agent grant implemented in `originweave-core`. Compatibility mutation tests create only controlled synthetic state inside the ephemeral test profile and must clean it up; successful API compatibility never grants the OriginWeave Agent ambient bookmarks/history/downloads authority.

For history compatibility specifically, the current official Chrome Extensions API documents the `history` manifest permission and Promise-returning `chrome.history.addUrl`, `chrome.history.search`, and `chrome.history.deleteUrl` methods. This living vendor reference establishes API semantics only. OriginWeave release evidence continues to depend on the exact pinned Chromium fixture and exact-head CI result rather than inferring compatibility from documentation.

The CI lane downloads the exact Chrome/ChromeDriver version from the official Chrome for Testing public bucket, records SHA-256 receipts for the downloaded archives, verifies the runtime-reported browser version, and emits bounded JSON compatibility evidence. A future release-quality compatibility matrix should additionally pin published artifact digests or equivalent immutable supply-chain identity when the upstream distribution exposes that identity in an authoritative machine-readable form.

## Primary references — APA 7th

Chrome for Developers. (n.d.). *Extensions / Manifest V3*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/develop/migrate/what-is-mv3

Chrome for Developers. (2023, May 2). *Extension service worker basics*. Google. https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/basics

Chrome for Developers. (2023, May 2). *The extension service worker lifecycle*. Google. https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle

Chrome for Developers. (n.d.). *chrome.declarativeNetRequest*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/reference/api/declarativeNetRequest

Chrome for Developers. (n.d.). *chrome.history*. Google. Retrieved August 11, 2026, from https://developer.chrome.com/docs/extensions/reference/api/history

Chrome for Developers. (n.d.). *Manifest file format*. Google. Retrieved August 9, 2026, from https://developer.chrome.com/docs/extensions/reference/manifest

Bynens, M. (2023, June 12). *Chrome for Testing*. Chrome for Developers. https://developer.chrome.com/docs/automation-and-testing/chrome-for-testing

Google Chrome Labs. (2026, July 21). *Chrome for Testing availability*. https://googlechromelabs.github.io/chrome-for-testing/
