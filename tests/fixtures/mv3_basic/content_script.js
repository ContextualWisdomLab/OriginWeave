"use strict";

(async () => {
  document.documentElement.dataset.originweaveContentScript = "ready";

  const previous = await chrome.storage.local.get("originweave_content");
  const storagePersisted = previous.originweave_content === "ready";
  if (!storagePersisted) {
    await chrome.storage.local.set({ originweave_content: "ready" });
  }
  const stored = await chrome.storage.local.get("originweave_content");
  document.documentElement.dataset.originweaveStorage =
    stored.originweave_content === "ready" ? "ready" : "missing";
  document.documentElement.dataset.originweaveStoragePersistence =
    storagePersisted ? "persisted" : "initialized";

  const response = await chrome.runtime.sendMessage("originweave-ping");
  document.documentElement.dataset.originweaveWorkerReply = response?.reply ?? "missing";
  document.documentElement.dataset.originweaveWorkerState = response?.worker ?? "missing";
  document.documentElement.dataset.originweaveWorkerStartCount = String(
    response?.workerStartCount ?? "missing"
  );
})();
