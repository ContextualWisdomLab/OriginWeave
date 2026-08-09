"use strict";

async function ensureWorkerState() {
  const values = await chrome.storage.local.get("originweave_worker");
  if (values.originweave_worker !== "installed") {
    await chrome.storage.local.set({ originweave_worker: "installed" });
  }
  return "installed";
}

chrome.runtime.onInstalled.addListener(() => {
  void ensureWorkerState();
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message !== "originweave-ping") {
    return false;
  }
  ensureWorkerState().then((worker) => {
    sendResponse({ reply: "pong", worker });
  });
  return true;
});
