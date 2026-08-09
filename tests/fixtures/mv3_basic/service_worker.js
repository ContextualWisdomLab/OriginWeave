"use strict";

chrome.runtime.onInstalled.addListener(async () => {
  await chrome.storage.local.set({ originweave_worker: "installed" });
});

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message !== "originweave-ping") {
    return false;
  }
  chrome.storage.local.get("originweave_worker").then((values) => {
    sendResponse({
      reply: "pong",
      worker: values.originweave_worker ?? "missing",
    });
  });
  return true;
});
