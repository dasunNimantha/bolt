const NATIVE_HOST = "com.bolt.nmh";
const IS_FIREFOX =
  typeof browser !== "undefined" && !!browser.runtime?.getBrowserInfo;

let interceptEnabled = true;
let videoDetectEnabled = true;
const interceptedIds = new Set();
const fallbackUrls = new Set();

// ─── Video detection state ──────────────────────────────────────────────

const tabVideos = new Map();

const VIDEO_MIME_TYPES = [
  "video/mp4",
  "video/webm",
  "video/ogg",
  "video/x-flv",
  "video/quicktime",
  "video/x-msvideo",
  "video/x-matroska",
  "video/mpeg",
  "video/3gpp",
  "video/mp2t",
];

const MANIFEST_MIME_TYPES = [
  "application/x-mpegurl",
  "application/vnd.apple.mpegurl",
  "audio/x-mpegurl",
  "audio/mpegurl",
];

const EXCLUDED_HOSTS = [
  "youtube.com",
  "www.youtube.com",
  "m.youtube.com",
  "music.youtube.com",
  "googlevideo.com",
];

function isExcludedHost(url) {
  try {
    const host = new URL(url).hostname;
    return EXCLUDED_HOSTS.some(
      (h) => host === h || host.endsWith("." + h),
    );
  } catch (_) {
    return false;
  }
}

function filenameFromUrl(url) {
  try {
    const path = new URL(url).pathname;
    const last = path.split("/").filter(Boolean).pop();
    if (last && /\.\w{2,5}$/.test(last)) return decodeURIComponent(last);
  } catch (_) {}
  return null;
}

// ─── Settings ───────────────────────────────────────────────────────────

chrome.storage.local.get({ enabled: true, videoDetect: true }, (data) => {
  interceptEnabled = data.enabled;
  videoDetectEnabled = data.videoDetect;
});

chrome.storage.onChanged.addListener((changes, area) => {
  if (area !== "local") return;
  if (changes.enabled) interceptEnabled = changes.enabled.newValue;
  if (changes.videoDetect) videoDetectEnabled = changes.videoDetect.newValue;
});

// ─── Download interception ──────────────────────────────────────────────

chrome.downloads.onCreated.addListener((downloadItem) => {
  if (!interceptEnabled) return;

  const url = downloadItem.finalUrl || downloadItem.url;
  if (!url.startsWith("http://") && !url.startsWith("https://")) return;

  if (fallbackUrls.delete(url)) return;

  const age = Date.now() - new Date(downloadItem.startTime).getTime();
  if (age > 5000) return;

  gatherAndSend(
    url,
    extractFilename(downloadItem.filename),
    downloadItem.referrer || null,
  );

  if (IS_FIREFOX) {
    const dlId = downloadItem.id;
    chrome.downloads.cancel(dlId, () => {
      const _err = chrome.runtime.lastError;
      chrome.downloads.erase({ id: dlId }, () => {
        const _err2 = chrome.runtime.lastError;
      });
    });
  } else {
    interceptedIds.add(downloadItem.id);
    setTimeout(() => interceptedIds.delete(downloadItem.id), 10000);
  }
});

// Chrome-only: onDeterminingFilename does not exist in Firefox
if (!IS_FIREFOX) {
  chrome.downloads.onDeterminingFilename.addListener(
    (downloadItem, suggest) => {
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
    },
  );
}

// ─── Video detection via webRequest ─────────────────────────────────────

const VIDEO_URL_EXTENSIONS = /\.(mp4|webm|mkv|avi|mov|flv|wmv|m4v|3gp|ogv)(\?|$)/i;
const MANIFEST_URL_EXTENSIONS = /\.(m3u8|mpd)(\?|$)/i;
const SEGMENT_EXTENSIONS = /\.(m4s|ts|m4f|m4a|m4v)(\?|#|$)/i;

chrome.webRequest.onResponseStarted.addListener(
  (details) => {
    if (!videoDetectEnabled) return;
    if (details.tabId < 0) return;
    if (isExcludedHost(details.url)) return;

    const status = details.statusCode;
    if (status !== 200 && status !== 206) return;

    const ctHeader = (details.responseHeaders || []).find(
      (h) => h.name.toLowerCase() === "content-type",
    );
    const ct = ctHeader
      ? ctHeader.value.toLowerCase().split(";")[0].trim()
      : "";

    const isVideoMime = VIDEO_MIME_TYPES.includes(ct);
    const isManifestMime = MANIFEST_MIME_TYPES.includes(ct);
    const isVideoUrl = VIDEO_URL_EXTENSIONS.test(details.url);
    const isManifestUrl = MANIFEST_URL_EXTENSIONS.test(details.url);

    if (!isVideoMime && !isManifestMime && !isVideoUrl && !isManifestUrl) return;

    try {
      const urlPath = new URL(details.url).pathname;
      if (SEGMENT_EXTENSIONS.test(urlPath)) return;
    } catch (_) {}

    const clHeader = (details.responseHeaders || []).find(
      (h) => h.name.toLowerCase() === "content-length",
    );
    const size = clHeader ? parseInt(clHeader.value, 10) : null;

    if (!isManifestMime && !isManifestUrl) {
      if (size !== null && size < 500000) return;
    }

    if (!tabVideos.has(details.tabId)) {
      tabVideos.set(details.tabId, []);
    }

    const list = tabVideos.get(details.tabId);
    if (list.some((v) => v.url === details.url)) return;

    list.push({
      url: details.url,
      contentType: ct || (isManifestUrl ? "application/x-mpegurl" : "video/mp4"),
      size: size,
      filename: filenameFromUrl(details.url),
      isManifest: isManifestMime || isManifestUrl,
    });

    chrome.tabs.sendMessage(
      details.tabId,
      { type: "videosUpdated", count: list.length },
      () => { const _err = chrome.runtime.lastError; },
    );
  },
  { urls: ["<all_urls>"] },
  ["responseHeaders"],
);

chrome.tabs.onRemoved.addListener((tabId) => {
  tabVideos.delete(tabId);
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo) => {
  if (changeInfo.status === "loading") {
    tabVideos.delete(tabId);
  }
});

// ─── Message handler from content script ────────────────────────────────

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (msg.type === "getVideos") {
    const tabId = sender.tab?.id;
    const videos = tabId != null ? tabVideos.get(tabId) || [] : [];
    sendResponse({ videos });
    return false;
  }

  if (msg.type === "reportVideos") {
    const tabId = sender.tab?.id;
    if (tabId == null) return false;
    const domVideos = msg.videos || [];
    if (!tabVideos.has(tabId)) {
      tabVideos.set(tabId, []);
    }
    const list = tabVideos.get(tabId);
    for (const v of domVideos) {
      if (!list.some((existing) => existing.url === v.url)) {
        list.push(v);
      }
    }
    sendResponse({ ok: true });
    return false;
  }

  if (msg.type === "downloadVideo") {
    const referrer = sender.tab?.url || null;
    gatherAndSend(msg.url, msg.filename || null, referrer);
    sendResponse({ ok: true });
    return false;
  }

  if (msg.type === "getVideoDetectEnabled") {
    sendResponse({ enabled: videoDetectEnabled });
    return false;
  }

  return false;
});

// ─── Shared helpers ─────────────────────────────────────────────────────

async function gatherAndSend(url, filename, referrer, cookieDomain) {
  let cookies = null;
  try {
    if (chrome.cookies) {
      const domain = cookieDomain || cookieDomainFor(url);
      const cookieList = await chrome.cookies.getAll({ domain });
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
    if (
      chrome.runtime.lastError ||
      (response && response.status === "error")
    ) {
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

function cookieDomainFor(url) {
  try {
    const host = new URL(url).hostname;
    if (host.endsWith(".fbcdn.net")) return ".facebook.com";
    if (host.endsWith(".twimg.com")) return ".twitter.com";
    if (host.endsWith(".cdninstagram.com")) return ".instagram.com";
    return host;
  } catch (_) {
    return "";
  }
}
