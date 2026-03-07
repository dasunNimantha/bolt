const NATIVE_HOST = "com.bolt.nmh";

let interceptEnabled = true;

chrome.storage.local.get({ enabled: true }, (data) => {
  interceptEnabled = data.enabled;
});

chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "local" && changes.enabled) {
    interceptEnabled = changes.enabled.newValue;
  }
});

// Fires before the save-as dialog — cancel here to suppress the dialog.
// Does NOT send to Bolt (onCreated handles that).
chrome.downloads.onDeterminingFilename.addListener((downloadItem, suggest) => {
  if (!interceptEnabled) {
    suggest();
    return;
  }

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    suggest();
    return;
  }

  chrome.downloads.cancel(downloadItem.id);
  chrome.downloads.erase({ id: downloadItem.id });
  // deliberately not calling suggest() — pauses Chrome's UI
});

// Fires first in the download lifecycle — this is where we send to Bolt.
chrome.downloads.onCreated.addListener((downloadItem) => {
  if (!interceptEnabled) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url.startsWith("http://") && !url.startsWith("https://")) return;

  chrome.downloads.cancel(downloadItem.id);
  chrome.downloads.erase({ id: downloadItem.id });

  const filename = extractFilename(downloadItem.filename);
  const referrer = downloadItem.referrer || null;

  gatherAndSend(url, filename, referrer);
});

async function gatherAndSend(url, filename, referrer) {
  let cookies = null;
  try {
    const urlObj = new URL(url);
    const cookieList = await chrome.cookies.getAll({ domain: urlObj.hostname });
    if (cookieList.length > 0) {
      cookies = cookieList.map((c) => `${c.name}=${c.value}`).join("; ");
    }
  } catch (_) {
    // Ignore cookie errors
  }

  const message = { url };
  if (filename) message.filename = filename;
  if (referrer) message.referrer = referrer;
  if (cookies) message.cookies = cookies;

  chrome.runtime.sendNativeMessage(NATIVE_HOST, message, (response) => {
    if (chrome.runtime.lastError) {
      console.error("Bolt native messaging error:", chrome.runtime.lastError.message);
      setBadge("!", "#f44336");
      return;
    }

    if (response && response.status === "ok") {
      setBadge("\u2713", "#4CAF50");
    } else {
      const errMsg = response?.message || "Unknown error";
      console.error("Bolt error:", errMsg);
      setBadge("!", "#f44336");
    }
  });
}

function extractFilename(path) {
  if (!path) return null;
  const parts = path.replace(/\\/g, "/").split("/");
  const name = parts[parts.length - 1];
  return name || null;
}

function setBadge(text, color) {
  chrome.action.setBadgeText({ text });
  chrome.action.setBadgeBackgroundColor({ color });
  setTimeout(() => {
    chrome.action.setBadgeText({ text: "" });
  }, 3000);
}
