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
  document.documentElement.dataset.originweaveExtensionVersion =
    response?.extensionVersion ?? "missing";
  document.documentElement.dataset.originweaveStorageMigration =
    response?.storageMigration ?? "missing";
  document.documentElement.dataset.originweaveTabs = response?.tabs ?? "missing";
  document.documentElement.dataset.originweaveWindows = response?.windows ?? "missing";
  document.documentElement.dataset.originweaveScripting = response?.scripting ?? "missing";
  document.documentElement.dataset.originweaveCommands = response?.commands ?? "missing";
  document.documentElement.dataset.originweaveSidePanel = response?.sidePanel ?? "missing";
  document.documentElement.dataset.originweaveBookmarks = response?.bookmarks ?? "missing";
  document.documentElement.dataset.originweaveBookmarksDiagnostic =
    response?.bookmarksDiagnostic ?? "bookmark-not-evaluated";
  document.documentElement.dataset.originweaveHistory = response?.history ?? "missing";
  document.documentElement.dataset.originweaveDownloads = response?.downloads ?? "missing";
  document.documentElement.dataset.originweaveDownloadsDiagnostic =
    response?.downloadsDiagnostic ?? "download-not-evaluated";
})();
