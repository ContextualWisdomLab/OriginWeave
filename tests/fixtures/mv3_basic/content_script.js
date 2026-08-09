"use strict";

(async () => {
  document.documentElement.dataset.originweaveContentScript = "ready";

  await chrome.storage.local.set({ originweave_content: "ready" });
  const stored = await chrome.storage.local.get("originweave_content");
  document.documentElement.dataset.originweaveStorage =
    stored.originweave_content === "ready" ? "ready" : "missing";

  const response = await chrome.runtime.sendMessage("originweave-ping");
  document.documentElement.dataset.originweaveWorkerReply = response?.reply ?? "missing";
  document.documentElement.dataset.originweaveWorkerState = response?.worker ?? "missing";
})();
