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

  return {
    tabs: tabReady ? "ready" : "missing",
    windows: windowReady ? "ready" : "missing",
    scripting: scriptingReady ? "ready" : "missing",
    commands: commandsReady ? "ready" : "missing",
    sidePanel: sidePanelReady ? "ready" : "missing",
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
      });
    }
  );
  return true;
});
