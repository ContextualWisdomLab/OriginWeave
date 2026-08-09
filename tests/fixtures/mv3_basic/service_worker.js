"use strict";

const workerStartPromise = (async () => {
  const values = await chrome.storage.local.get("originweave_worker_start_count");
  const previous = Number(values.originweave_worker_start_count ?? 0);
  const next = Number.isSafeInteger(previous) && previous >= 0 ? previous + 1 : 1;
  await chrome.storage.local.set({ originweave_worker_start_count: next });
  return next;
})();

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
  Promise.all([ensureWorkerState(), workerStartPromise]).then(
    ([worker, workerStartCount]) => {
      sendResponse({ reply: "pong", worker, workerStartCount });
    }
  );
  return true;
});
