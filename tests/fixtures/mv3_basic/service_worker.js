"use strict";

const DOWNLOAD_PAYLOAD = "OriginWeave deterministic MV3 download fixture.\n";
const DOWNLOAD_POLL_ATTEMPTS = 100;
const DOWNLOAD_POLL_INTERVAL_MS = 50;
const INITIAL_FIXTURE_VERSION = "1.0.0";
const UPDATED_FIXTURE_VERSION = "1.0.1";
const INITIAL_SCHEMA_VERSION = 1;
const UPDATED_SCHEMA_VERSION = 2;

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

async function ensureStorageMigrationState() {
  const extensionVersion = chrome.runtime.getManifest().version;
  const values = await chrome.storage.local.get("originweave_fixture_schema_version");
  const currentSchemaVersion = values.originweave_fixture_schema_version;

  if (extensionVersion === INITIAL_FIXTURE_VERSION) {
    if (currentSchemaVersion === undefined) {
      await chrome.storage.local.set({
        originweave_fixture_schema_version: INITIAL_SCHEMA_VERSION,
      });
      return { extensionVersion, storageMigration: "initialized" };
    }
    if (currentSchemaVersion === INITIAL_SCHEMA_VERSION) {
      return { extensionVersion, storageMigration: "current" };
    }
    return { extensionVersion, storageMigration: "invalid" };
  }

  if (extensionVersion === UPDATED_FIXTURE_VERSION) {
    if (currentSchemaVersion === INITIAL_SCHEMA_VERSION) {
      await chrome.storage.local.set({
        originweave_fixture_schema_version: UPDATED_SCHEMA_VERSION,
      });
      return { extensionVersion, storageMigration: "migrated" };
    }
    if (currentSchemaVersion === UPDATED_SCHEMA_VERSION) {
      return { extensionVersion, storageMigration: "migrated" };
    }
  }

  return { extensionVersion, storageMigration: "invalid" };
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

async function exerciseDownload(sender) {
  const sourceUrl = sender?.tab?.url;
  if (typeof sourceUrl !== "string") {
    return { ready: false, diagnostic: "download-source-rejected" };
  }

  let parsed;
  try {
    parsed = new URL(sourceUrl);
  } catch (_error) {
    return { ready: false, diagnostic: "download-source-rejected" };
  }
  if (
    parsed.protocol !== "http:" ||
    parsed.hostname !== "127.0.0.1" ||
    parsed.pathname !== "/page.html" ||
    parsed.username !== "" ||
    parsed.password !== ""
  ) {
    return { ready: false, diagnostic: "download-source-rejected" };
  }

  const url = new URL("download.txt", sourceUrl).href;
  let downloadId;
  try {
    downloadId = await chrome.downloads.download({
      url,
      filename: "originweave-mv3/download.txt",
      conflictAction: "uniquify",
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

async function exerciseBookmarkMutation(sender) {
  const sourceUrl = sender?.tab?.url;
  if (typeof sourceUrl !== "string") {
    return false;
  }

  let parsed;
  try {
    parsed = new URL(sourceUrl);
  } catch (_error) {
    return false;
  }
  if (
    parsed.protocol !== "http:" ||
    parsed.hostname !== "127.0.0.1" ||
    parsed.pathname !== "/page.html" ||
    parsed.username !== "" ||
    parsed.password !== ""
  ) {
    return false;
  }

  const title = "OriginWeave MV3 compatibility bookmark";
  let bookmarkId;
  try {
    const created = await chrome.bookmarks.create({ title, url: sourceUrl });
    if (typeof created?.id !== "string" || created.id.length === 0) {
      return false;
    }
    bookmarkId = created.id;
  } catch (_error) {
    return false;
  }

  let bookmarkMutationReady = false;
  try {
    const nodes = await chrome.bookmarks.get(bookmarkId);
    bookmarkMutationReady =
      Array.isArray(nodes) &&
      nodes.length === 1 &&
      nodes[0]?.id === bookmarkId &&
      nodes[0]?.title === title &&
      nodes[0]?.url === sourceUrl;
  } catch (_error) {
    bookmarkMutationReady = false;
  } finally {
    try {
      await chrome.bookmarks.remove(bookmarkId);
    } catch (_error) {
      bookmarkMutationReady = false;
    }
  }
  return bookmarkMutationReady;
}

async function exerciseHistoryMutation(sender) {
  const sourceUrl = sender?.tab?.url;
  if (typeof sourceUrl !== "string") {
    return false;
  }

  let parsed;
  try {
    parsed = new URL(sourceUrl);
  } catch (_error) {
    return false;
  }
  if (
    parsed.protocol !== "http:" ||
    parsed.hostname !== "127.0.0.1" ||
    parsed.pathname !== "/page.html" ||
    parsed.username !== "" ||
    parsed.password !== ""
  ) {
    return false;
  }

  const historyUrl = new URL("history-entry.html", sourceUrl).href;
  let historyMutationReady = false;
  try {
    await chrome.history.addUrl({ url: historyUrl });
    const items = await chrome.history.search({
      text: historyUrl,
      startTime: 0,
      maxResults: 10,
    });
    historyMutationReady =
      Array.isArray(items) && items.some((item) => item?.url === historyUrl);
  } catch (_error) {
    historyMutationReady = false;
  } finally {
    try {
      await chrome.history.deleteUrl({ url: historyUrl });
      const remainingItems = await chrome.history.search({
        text: historyUrl,
        startTime: 0,
        maxResults: 10,
      });
      if (
        !Array.isArray(remainingItems) ||
        remainingItems.some((item) => item?.url === historyUrl)
      ) {
        historyMutationReady = false;
      }
    } catch (_error) {
      historyMutationReady = false;
    }
  }
  return historyMutationReady;
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

  const bookmarkMutationReady = await exerciseBookmarkMutation(sender);
  const bookmarksReady = bookmarkMutationReady;

  const historyMutationReady = await exerciseHistoryMutation(sender);
  const historyReady = historyMutationReady;

  const downloadResult = await exerciseDownload(sender);
  const downloadsReady = downloadResult.ready;

  return {
    tabs: tabReady ? "ready" : "missing",
    windows: windowReady ? "ready" : "missing",
    scripting: scriptingReady ? "ready" : "missing",
    commands: commandsReady ? "ready" : "missing",
    sidePanel: sidePanelReady ? "ready" : "missing",
    bookmarks: bookmarksReady ? "ready" : "missing",
    history: historyReady ? "ready" : "missing",
    downloads: downloadsReady ? "ready" : "missing",
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
  Promise.all([
    ensureWorkerState(),
    workerStartPromise,
    ensureStorageMigrationState(),
    exerciseCoreApis(sender),
  ]).then(
    ([worker, workerStartCount, migrationState, coreApis]) => {
      sendResponse({
        reply: "pong",
        worker,
        workerStartCount,
        extensionVersion: migrationState.extensionVersion,
        storageMigration: migrationState.storageMigration,
        ...coreApis,
      });
    },
    () => {
      sendResponse({
        reply: "pong",
        worker: "installed",
        workerStartCount: 0,
        extensionVersion: "missing",
        storageMigration: "invalid",
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
