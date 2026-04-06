(() => {
  const EXCLUDED_HOSTS = [
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "music.youtube.com",
  ];

  const host = location.hostname;
  if (EXCLUDED_HOSTS.some((h) => host === h || host.endsWith("." + h))) return;

  const BOLT_ATTR = "data-bolt-video";
  const MIN_VIDEO_SIZE = window === window.top ? 200 : 100;
  let detectEnabled = true;
  const processedVideos = new WeakSet();
  const overlays = new Map();

  chrome.runtime.sendMessage({ type: "getVideoDetectEnabled" }, (resp) => {
    if (chrome.runtime.lastError) return;
    if (resp) detectEnabled = resp.enabled;
    if (detectEnabled) init();
  });

  chrome.storage.onChanged.addListener((changes, area) => {
    if (area !== "local") return;
    if (changes.videoDetect) {
      detectEnabled = changes.videoDetect.newValue;
      if (detectEnabled) {
        init();
      } else {
        removeAllOverlays();
      }
    }
  });

  function init() {
    scanForVideos();
    observeDOM();
  }

  // ─── DOM scanning ──────────────────────────────────────────────────────

  function scanForVideos() {
    if (!detectEnabled) return;
    const videos = document.querySelectorAll("video");
    videos.forEach(processVideo);
  }

  function processVideo(video) {
    if (processedVideos.has(video)) return;
    if (video.hasAttribute(BOLT_ATTR)) return;
    if (video.offsetWidth < MIN_VIDEO_SIZE && video.offsetHeight < MIN_VIDEO_SIZE) return;

    processedVideos.add(video);
    video.setAttribute(BOLT_ATTR, "1");

    const urls = collectVideoUrls(video);
    if (urls.length > 0) reportUrlsToBackground(urls);

    attachOverlay(video);

    video.addEventListener("loadeddata", () => {
      const newUrls = collectVideoUrls(video);
      if (newUrls.length > 0) reportUrlsToBackground(newUrls);
    }, { once: true });
  }

  function collectVideoUrls(video) {
    const urls = [];
    const seen = new Set();

    function addUrl(url) {
      if (!url) return;
      if (url.startsWith("blob:") || url.startsWith("data:")) return;
      try {
        const absolute = new URL(url, location.href).href;
        if (!absolute.startsWith("http")) return;
        if (seen.has(absolute)) return;
        seen.add(absolute);
        urls.push({
          url: absolute,
          filename: filenameFromUrl(absolute),
          contentType: guessType(absolute),
          size: null,
        });
      } catch (_) {}
    }

    addUrl(video.src);
    addUrl(video.currentSrc);
    video.querySelectorAll("source").forEach((s) => addUrl(s.src));

    return urls;
  }

  function filenameFromUrl(url) {
    try {
      const path = new URL(url).pathname;
      const last = path.split("/").filter(Boolean).pop();
      if (last && /\.\w{2,5}$/.test(last)) return decodeURIComponent(last);
    } catch (_) {}
    return null;
  }

  function guessType(url) {
    const ext = url.split("?")[0].split(".").pop().toLowerCase();
    const map = {
      mp4: "video/mp4", webm: "video/webm", ogv: "video/ogg", ogg: "video/ogg",
      mkv: "video/x-matroska", avi: "video/x-msvideo", mov: "video/quicktime",
      m3u8: "application/x-mpegurl", ts: "video/mp2t",
    };
    return map[ext] || "video/mp4";
  }

  function reportUrlsToBackground(urls) {
    try {
      chrome.runtime.sendMessage({ type: "reportVideos", videos: urls }, () => {
        const _err = chrome.runtime.lastError;
      });
    } catch (_) {}
  }

  // ─── MutationObserver ─────────────────────────────────────────────────

  let observerActive = false;

  function observeDOM() {
    if (observerActive) return;
    observerActive = true;

    const observer = new MutationObserver((mutations) => {
      if (!detectEnabled) return;
      for (const m of mutations) {
        for (const node of m.addedNodes) {
          if (node.nodeType !== 1) continue;
          if (node.tagName === "VIDEO") processVideo(node);
          node.querySelectorAll?.("video").forEach(processVideo);
        }
      }
    });

    observer.observe(document.body || document.documentElement, {
      childList: true,
      subtree: true,
    });
  }

  // ─── Overlay UI (Shadow DOM, no wrapper) ──────────────────────────────

  function attachOverlay(video) {
    const shadowHost = document.createElement("div");
    shadowHost.className = "bolt-overlay-host";
    shadowHost.style.cssText =
      "position:fixed;z-index:2147483647;display:none;";

    const shadow = shadowHost.attachShadow({ mode: "closed" });

    const style = document.createElement("style");
    style.textContent = OVERLAY_CSS;

    const btn = document.createElement("button");
    btn.className = "bolt-btn";
    btn.title = "Download video with Bolt";
    btn.innerHTML = `<svg viewBox="0 0 48 48" fill="none"><rect width="48" height="48" rx="8" fill="#F2BF40"/><path d="M27 6L15 27h7.5l-3 15 12-21H24l3-15z" fill="#1A1A1A"/></svg><span>Download with Bolt</span>`;

    const dropdown = document.createElement("div");
    dropdown.className = "bolt-dropdown";

    let dropdownOpen = false;

    btn.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (dropdownOpen) {
        dropdown.classList.remove("open");
        dropdownOpen = false;
        return;
      }
      dropdown.classList.add("open");
      dropdownOpen = true;
      loadVideos(dropdown, video);
    });

    document.addEventListener("click", () => {
      if (dropdownOpen) {
        dropdown.classList.remove("open");
        dropdownOpen = false;
      }
    });

    shadow.appendChild(style);
    shadow.appendChild(btn);
    shadow.appendChild(dropdown);
    document.documentElement.appendChild(shadowHost);

    overlays.set(video, shadowHost);

    function updatePosition() {
      const rect = video.getBoundingClientRect();
      if (rect.width < 50 || rect.height < 50) {
        shadowHost.style.display = "none";
        return;
      }
      shadowHost.style.top = (rect.top + 8) + "px";
      shadowHost.style.left = (rect.left + 8) + "px";
      shadowHost.style.display = "block";
    }

    updatePosition();

    const posInterval = setInterval(() => {
      if (!document.contains(video)) {
        clearInterval(posInterval);
        shadowHost.remove();
        overlays.delete(video);
        return;
      }
      updatePosition();
    }, 1000);

    window.addEventListener("scroll", updatePosition, { passive: true });
    window.addEventListener("resize", updatePosition, { passive: true });

    chrome.runtime.onMessage.addListener((msg) => {
      if (msg.type === "videosUpdated" && dropdownOpen) {
        loadVideos(dropdown, video);
      }
    });
  }

  const OVERLAY_CSS = `
    :host {
      all: initial;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }
    .bolt-btn {
      height: 36px;
      padding: 0 16px;
      border-radius: 8px;
      background: rgba(242, 191, 64, 0.95);
      border: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
      transition: transform 0.15s, background 0.15s;
      box-shadow: 0 2px 12px rgba(0,0,0,0.4);
    }
    .bolt-btn:hover { transform: scale(1.03); background: rgba(242, 191, 64, 1); }
    .bolt-btn svg { width: 20px; height: 20px; flex-shrink: 0; border-radius: 3px; }
    .bolt-btn span {
      font-size: 14px;
      font-weight: 700;
      color: #1a1a1a;
      white-space: nowrap;
      letter-spacing: -0.2px;
    }
    .bolt-dropdown {
      position: absolute;
      top: 40px;
      left: 0;
      min-width: 260px;
      max-width: 380px;
      max-height: 340px;
      overflow-y: auto;
      background: #1a2744;
      border: 1px solid rgba(255,255,255,0.1);
      border-radius: 10px;
      box-shadow: 0 8px 32px rgba(0,0,0,0.5);
      display: none;
    }
    .bolt-dropdown.open { display: block; }
    .bolt-dropdown-header {
      padding: 10px 14px;
      font-size: 12px;
      font-weight: 600;
      color: #F2BF40;
      border-bottom: 1px solid rgba(255,255,255,0.06);
      display: flex;
      align-items: center;
      gap: 6px;
      position: sticky;
      top: 0;
      background: #1a2744;
    }
    .bolt-dropdown-header svg { width: 14px; height: 14px; }
    .bolt-item {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 10px 14px;
      cursor: pointer;
      transition: background 0.15s;
      border: none;
      background: none;
      width: 100%;
      text-align: left;
      color: #e0e0e0;
      font-size: 13px;
    }
    .bolt-item:hover { background: rgba(255,255,255,0.06); }
    .bolt-item-info { flex: 1; min-width: 0; }
    .bolt-item-name {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: 13px;
      color: #fff;
    }
    .bolt-item-meta { font-size: 11px; color: #7a889e; margin-top: 2px; }
    .bolt-item-dl {
      width: 28px; height: 28px;
      border-radius: 6px;
      background: rgba(242,191,64,0.15);
      border: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      transition: background 0.15s;
    }
    .bolt-item-dl:hover { background: rgba(242,191,64,0.3); }
    .bolt-item-dl svg { width: 14px; height: 14px; }
    .bolt-empty { padding: 16px 14px; text-align: center; color: #7a889e; font-size: 12px; }
    .bolt-spinner { padding: 16px 14px; text-align: center; color: #7a889e; font-size: 12px; }
  `;

  function loadVideos(dropdown, video) {
    dropdown.textContent = "";

    const header = document.createElement("div");
    header.className = "bolt-dropdown-header";
    header.innerHTML = `<svg viewBox="0 0 48 48" fill="none"><rect width="48" height="48" rx="8" fill="#F2BF40"/><path d="M27 6L15 27h7.5l-3 15 12-21H24l3-15z" fill="#1A1A1A"/></svg>`;
    const headerText = document.createElement("span");
    headerText.textContent = "Download with Bolt";
    header.appendChild(headerText);
    dropdown.appendChild(header);

    const spinner = document.createElement("div");
    spinner.className = "bolt-spinner";
    spinner.textContent = "Finding videos...";
    dropdown.appendChild(spinner);

    const domUrls = collectVideoUrls(video);

    chrome.runtime.sendMessage({ type: "getVideos" }, (resp) => {
      if (chrome.runtime.lastError) {
        spinner.textContent = "Error loading videos";
        return;
      }

      const bgVideos = resp?.videos || [];
      const allVideos = mergeVideos(domUrls, bgVideos);

      dropdown.removeChild(spinner);

      if (allVideos.length === 0) {
        const empty = document.createElement("div");
        empty.className = "bolt-empty";
        empty.textContent = "No downloadable videos found. Try playing the video first.";
        dropdown.appendChild(empty);
        return;
      }

      allVideos.forEach((v) => {
        const item = document.createElement("button");
        item.className = "bolt-item";

        const info = document.createElement("div");
        info.className = "bolt-item-info";

        const name = document.createElement("div");
        name.className = "bolt-item-name";
        name.textContent = v.filename || urlBasename(v.url);

        const meta = document.createElement("div");
        meta.className = "bolt-item-meta";
        const parts = [];
        if (v.isManifest) {
          parts.push("HLS Stream");
        } else if (v.contentType) {
          parts.push(v.contentType.replace("video/", "").replace("application/", "").toUpperCase());
        }
        if (v.size) parts.push(formatBytes(v.size));
        meta.textContent = parts.join(" · ") || v.url.substring(0, 60);

        info.appendChild(name);
        info.appendChild(meta);

        const dlBtn = document.createElement("div");
        dlBtn.className = "bolt-item-dl";
        dlBtn.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="#F2BF40" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`;

        item.appendChild(info);
        item.appendChild(dlBtn);

        item.addEventListener("click", (e) => {
          e.preventDefault();
          e.stopPropagation();
          chrome.runtime.sendMessage({
            type: "downloadVideo",
            url: v.url,
            filename: v.filename,
          }, () => { const _err = chrome.runtime.lastError; });
          name.textContent = "Sent to Bolt!";
          name.style.color = "#4ade80";
          setTimeout(() => { dropdown.classList.remove("open"); }, 1000);
        });

        dropdown.appendChild(item);
      });
    });
  }

  function mergeVideos(domList, bgList) {
    const seen = new Set();
    const result = [];
    for (const list of [bgList, domList]) {
      for (const v of list) {
        if (seen.has(v.url)) continue;
        seen.add(v.url);
        result.push(v);
      }
    }
    return result;
  }

  function urlBasename(url) {
    try {
      const path = new URL(url).pathname;
      const last = path.split("/").filter(Boolean).pop();
      return last ? decodeURIComponent(last).substring(0, 50) : "video";
    } catch (_) { return "video"; }
  }

  function formatBytes(bytes) {
    if (!bytes || bytes <= 0) return "";
    const units = ["B", "KB", "MB", "GB"];
    let i = 0;
    let size = bytes;
    while (size >= 1024 && i < units.length - 1) { size /= 1024; i++; }
    return size.toFixed(i > 0 ? 1 : 0) + " " + units[i];
  }

  // ─── Cleanup ──────────────────────────────────────────────────────────

  function removeAllOverlays() {
    overlays.forEach((host, video) => {
      host.remove();
      video.removeAttribute(BOLT_ATTR);
      processedVideos.delete(video);
    });
    overlays.clear();
  }
})();
