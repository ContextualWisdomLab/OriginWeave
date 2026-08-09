"use strict";

const DOWNLOAD_PAYLOAD = "OriginWeave deterministic MV3 download fixture.\n";
const DOWNLOAD_POLL_ATTEMPTS = 100;
const DOWNLOAD_POLL_INTERVAL_MS = 50;

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

async function waitForDownload(downloadId, expectedUrl) {
  const expectedBytes = new TextEncoder().encode(DOWNLOAD_PAYLOAD).byteLength;
  let observedDownload = false;
  for (let attempt = 0; attempt < DOWNLOAD_POLL_ATTEMPTS; attempt += 1) {
    let items;
    try {
      items = await chrome.downloads.search({ id: downloadId, limit: 1 });
    } catch (_error) {
      return { ready: false, diagnostic: "download-search-missing" };
    }
    if (!Array.isArray(items) || items.length !== 1) {
      await new Promise((resolve) => setTimeout(resolve, DOWNLOAD_POLL_INTERVAL_MS));
      continue;
    }
    observedDownload = true;
    const item = items[0];
    if (item.state === "interrupted") {
      return { ready: false, diagnostic: "download-interrupted" };
    }
    if (item.state === "complete") {
      if (item.url !== expectedUrl) {
        return { ready: false, diagnostic: "download-url-mismatch" };
      }
      if (item.bytesReceived !== expectedBytes || item.totalBytes !== expectedBytes) {
        return { ready: false, diagnostic: "download-byte-count-mismatch" };
      }
      if (item.exists === false) {
        return { ready: false, diagnostic: "download-exists-false" };
      }
      return { ready: true, diagnostic: "download-complete-ready" };
    }
    await new Promise((resolve) => setTimeout(resolve, DOWNLOAD_POLL_INTERVAL_MS));
  }
  return {
    ready: false,
    diagnostic: observedDownload ? "download-timeout" : "download-search-missing",
  };
}

async function exerciseDownload() {
  const url = chrome.runtime.getURL("download.txt");
  let downloadId;
  try {
    downloadId = await chrome.downloads.download({
      url,
      filename: "originweave-mv3/download.txt",
      conflictAction: "overwrite",
      saveAs: false,
    });
  } catch (_error) {
    return { ready: false, diagnostic: "download-start-rejected" };
  }
  if (!Number.isInteger(downloadId)) {
    return { ready: false, diagnostic: "download-start-rejected" };
  }
  return waitForDownload(downloadId, url);
}

async function exerciseCoreApis(sender) {
  const tabId = sender?.tab?.id;
  if (!Number.isInteger(tabId)) {
    throw new Error("fixture message is not bound to a browser tab");
  }

  const tabs = await chrome.tabs.query({});
  const tabReady = tabs.some((tab) => tab.id === tabId);

  const currentWindow = await chrome.windows.getCurrent({ populate: false });
  const windowReady = Number.isInteger(currentWindow?.id);

  const injection = await chrome.scripting.executeScript({
    target: { tabId },
    func: () => {
      document.documentElement.dataset.originweaveScriptingExecuted = "ready";
      return "ready";
    },
  });
  const scriptingReady =
    Array.isArray(injection) && injection.some((result) => result.result === "ready");

  const commands = await chrome.commands.getAll();
  const commandsReady = commands.some(
    (command) => command.name === "originweave-fixture-command"
  );

  const sidePanelOptions = await chrome.sidePanel.getOptions({ tabId });
  const sidePanelReady = sidePanelOptions?.path === "side_panel.html";

  const bookmarkTree = await chrome.bookmarks.getTree();
  const bookmarksReady = Array.isArray(bookmarkTree) && bookmarkTree.length > 0;

  const historyItems = await chrome.history.search({
    text: "",
    startTime: 0,
    maxResults: 10,
  });
  const historyReady = Array.isArray(historyItems);

  const downloadResult = await exerciseDownload();

  return {
    tabs: tabReady ? "ready" : "missing",
    windows: windowReady ? "ready" : "missing",
    scripting: scriptingReady ? "ready" : "missing",
    commands: commandsReady ? "ready" : "missing",
    sidePanel: sidePanelReady ? "ready" : "missing",
    bookmarks: bookmarksReady ? "ready" : "missing",
    history: historyReady ? "ready" : "missing",
    downloads: downloadResult.ready ? "ready" : "missing",
    downloadsDiagnostic: downloadResult.diagnostic,
  };
}

chrome.runtime.onInstalled.addListener(() => {
  void ensureWorkerState();
});

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message !== "originweave-ping") {
    return false;
  }
  Promise.all([ensureWorkerState(), workerStartPromise, exerciseCoreApis(sender)]).then(
    ([worker, workerStartCount, coreApis]) => {
      sendResponse({ reply: "pong", worker, workerStartCount, ...coreApis });
    },
    () => {
      sendResponse({
        reply: "pong",
        worker: "installed",
        workerStartCount: 0,
        tabs: "missing",
        windows: "missing",
        scripting: "missing",
        commands: "missing",
        sidePanel: "missing",
        bookmarks: "missing",
        history: "missing",
        downloads: "missing",
        downloadsDiagnostic: "download-not-evaluated",
      });
    }
  );
  return true;
});
