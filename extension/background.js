const NATIVE_HOST = "com.bolt.nmh";

let interceptEnabled = true;
const interceptedIds = new Set();
const fallbackUrls = new Set();

chrome.storage.local.get({ enabled: true }, (data) => {
  interceptEnabled = data.enabled;
});

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.enabled) {
    interceptEnabled = changes.enabled.newValue;
  }
});

chrome.downloads.onCreated.addListener((downloadItem) => {
  if (!interceptEnabled) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url.startsWith("http://") && !url.startsWith("https://")) return;

  if (fallbackUrls.delete(url)) return;

  const age = Date.now() - new Date(downloadItem.startTime).getTime();
  if (age > 5000) return;

  interceptedIds.add(downloadItem.id);
  setTimeout(() => interceptedIds.delete(downloadItem.id), 10000);

  gatherAndSend(
    url,
    extractFilename(downloadItem.filename),
    downloadItem.referrer || null,
  );
});

chrome.downloads.onDeterminingFilename.addListener((downloadItem, suggest) => {
  if (interceptedIds.delete(downloadItem.id)) {
    const dlId = downloadItem.id;
    suggest({ filename: downloadItem.filename });
    chrome.downloads.cancel(dlId, () => {
      const _err = chrome.runtime.lastError;
      chrome.downloads.erase({ id: dlId }, () => {
        const _err2 = chrome.runtime.lastError;
      });
    });
    return;
  }
  suggest();
});

async function gatherAndSend(url, filename, referrer) {
  let cookies = null;
  try {
    if (chrome.cookies) {
      const cookieList = await chrome.cookies.getAll({
        domain: new URL(url).hostname,
      });
      if (cookieList.length) {
        cookies = cookieList.map((c) => `${c.name}=${c.value}`).join("; ");
      }
    }
  } catch (_) {}

  const message = { url };
  if (filename) message.filename = filename;
  if (referrer) message.referrer = referrer;
  if (cookies) message.cookies = cookies;

  chrome.runtime.sendNativeMessage(NATIVE_HOST, message, (response) => {
    if (chrome.runtime.lastError || (response && response.status === "error")) {
      fallbackToChrome(url);
    } else if (response && response.status === "ok") {
      setBadge("\u2713", "#4CAF50");
    } else {
      setBadge("!", "#f44336");
    }
  });
}

function fallbackToChrome(url) {
  fallbackUrls.add(url);
  setTimeout(() => fallbackUrls.delete(url), 30000);
  chrome.downloads.download({ url }, () => {
    const _err = chrome.runtime.lastError;
  });
}

function extractFilename(path) {
  if (!path) return null;
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || null;
}

function setBadge(text, color) {
  chrome.action.setBadgeText({ text });
  chrome.action.setBadgeBackgroundColor({ color });
  setTimeout(() => chrome.action.setBadgeText({ text: "" }), 3000);
}
