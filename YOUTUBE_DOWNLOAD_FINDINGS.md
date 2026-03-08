# YouTube Video Download — Technical Findings

Investigation conducted March 2026 while attempting to add YouTube video
download support to the Bolt browser extension.

---

## Summary

Downloading YouTube videos from a browser extension is **not feasible** using
standard extension APIs alone. Every approach attempted resulted in either
403 errors, empty captures, or security policy violations. The only reliable
method is `yt-dlp` (an external tool), which reimplements YouTube's
proprietary URL signing and n-parameter transformation in ~20k lines of
Python.

---

## Approaches Tried & Why They Failed

### 1. `chrome.downloads.download()` with streamingData URLs
- **What**: Extract video URLs from `movie_player.getPlayerResponse().streamingData`,
  then pass them to Chrome's download API.
- **Result**: **403 Forbidden** on every request.
- **Why**: YouTube's streaming URLs contain an `n` parameter (throttle token).
  The raw URL from `streamingData` has the *untransformed* `n` value. YouTube's
  player JavaScript transforms `n` before making requests. Without this
  transform, the CDN rejects the request with 403.

### 2. `fetch()` in MAIN world with streamingData URLs
- **What**: Inject a script into the page's main JS context, use `fetch()` to
  download the video (hoping the page's cookies/session would help).
- **Result**: **403 Forbidden**.
- **Why**: Same `n` parameter issue. The cookies/session don't help because the
  403 is caused by the untransformed throttle token, not missing auth.
- **Additional issue**: YouTube's Content Security Policy blocks `innerHTML`
  assignments via Trusted Types, preventing UI injection in MAIN world.

### 3. Monkey-patching `fetch()` and `XMLHttpRequest` in MAIN world (`yt-hook.js`)
- **What**: Override `fetch` and `XHR.open` at `document_start` in the MAIN
  world to intercept YouTube's video requests and capture the *transformed*
  URLs (with correct `n` parameter).
- **Result**: **Zero captures** (`captured: 0`).
- **Why**: YouTube's video player fetches media data through **Web Workers**
  or the browser's internal **Media Source Extensions (MSE)** pipeline. These
  bypass the main thread's `fetch`/`XHR` globals entirely, so monkey-patching
  them sees nothing.

### 4. `PerformanceObserver` / `performance.getEntriesByType("resource")`
- **What**: Monitor all network resource loads via the Performance API. Filter
  for `googlevideo.com` URLs with successful transfers.
- **Result**: Captured some URLs but they were **stale/initial** requests that
  had already received 403. The successfully-playing URLs either didn't
  appear or had `transferSize: 0` due to CORS restrictions on the
  Performance API (cross-origin resources don't expose timing details).

### 5. `chrome.webRequest.onResponseStarted` for `*.googlevideo.com`
- **What**: Use the extension's webRequest API to monitor all HTTP responses
  from YouTube's video CDN. Filter for 200/206 status codes with `itag` in
  the URL.
- **Result**: Listener fired, saw 39 requests total:
  - 6 × 403 (blocked video requests — these had `itag` in URL)
  - 15 × 200/206 (successful, but **none** had `itag` in URL — these were
    utility/stats/logging endpoints, not video data)
  - 18 × other status codes (204, 302, etc.)
- **Why**: The actual video data requests either:
  - Use a different URL structure (path-based itag: `/itag/243/` instead of
    `?itag=243`)
  - Are delivered through YouTube's **SABR (Server Adaptive Bitrate)** protocol,
    which multiplexes video data through a single long-lived connection that
    doesn't follow the traditional `videoplayback?itag=X` format
  - Come from a different domain or through a mechanism not visible to
    `webRequest`

### 6. `yt-dlp` integration (external tool)
- **What**: Call `yt-dlp -g -f ITAG URL` from Bolt's Rust backend to resolve
  the real download URL (yt-dlp handles n-parameter transformation internally).
- **Result**: **URL resolution worked** — yt-dlp returned valid CDN URLs.
  However, the full integration had UX issues:
  - Takes 10–15 seconds to resolve (yt-dlp must fetch the page, extract
    player JS, find the n-transform function, and apply it)
  - Bolt's Rust HTTP client may still get 403 due to **TLS fingerprinting** —
    YouTube's CDN can distinguish requests from Chrome vs. a Rust HTTP
    client (different TLS ClientHello)
  - Progressive formats (360p/720p with audio) work, but adaptive formats
    (1080p+) are video-only and need separate audio muxing

---

## Key Technical Barriers

### N-Parameter Transformation
YouTube adds a `n` parameter to every video URL. The raw value from the API
is a "challenge" that must be transformed by a JavaScript function embedded
in YouTube's player code (`base.js`). The function changes frequently
(~weekly) and is obfuscated. Without the correct transformation, the CDN
returns 403 or throttles to ~100 KB/s. This is YouTube's primary
anti-download measure.

### TLS Fingerprinting
YouTube's CDN (`googlevideo.com`) may inspect the TLS ClientHello to verify
the request comes from a real browser. Rust's `reqwest` (with `rustls`) has
a different TLS fingerprint than Chrome, which can cause 403 even with a
valid URL. Chrome's own download API (`chrome.downloads.download()`) uses
the browser's TLS stack and *would* work, but can't because of the
n-parameter issue.

### SABR Protocol
YouTube has been migrating from traditional DASH (one HTTP request per video
segment) to SABR (Server Adaptive Bitrate), where the server controls
bitrate selection and delivers video through a different mechanism. This
makes intercepting individual segment URLs via webRequest unreliable.

### Web Worker Isolation
YouTube's video player fetches media data from Web Workers, which have their
own `fetch`/`XHR` globals that can't be monkey-patched from the main thread.
This prevents content scripts from intercepting the actual video requests.

### Content Security Policy (Trusted Types)
YouTube enforces Trusted Types, which blocks `innerHTML` assignments from
injected scripts in the MAIN world. All DOM manipulation must use
`createElement`/`textContent`/`appendChild` — no HTML string insertion.

---

## What Would Actually Work

1. **Full `yt-dlp` subprocess** — Have Bolt spawn `yt-dlp` to handle the
   entire download (not just URL resolution). This avoids TLS fingerprinting
   since yt-dlp uses its own HTTP client with browser-like fingerprints.
   Downside: requires yt-dlp installed, 10-15s startup time, and tracking an
   external process.

2. **Chrome DevTools Protocol (`chrome.debugger`)** — Attach to the tab via
   the debugger API and monitor `Network.responseReceived` events. This sees
   ALL requests including those from Workers. Downside: shows "Extension is
   debugging this browser" banner — terrible UX.

3. **Re-implementing YouTube's n-transform** — Extract and execute YouTube's
   obfuscated JavaScript transform function on every page load. This is what
   yt-dlp does. Downside: fragile, breaks frequently, requires maintaining a
   parser for YouTube's obfuscated code.

---

## Files That Were Modified/Created

- `extension/yt-hook.js` — MAIN world content script for YouTube URL capture
- `extension/content.js` — Video overlay UI, YouTube streaming data parsing
- `extension/background.js` — webRequest listeners, YouTube URL capture, ytdl handling
- `extension/manifest.json` — yt-hook.js injection, googlevideo.com permissions
- `src/ipc.rs` — yt-dlp URL resolution in Bolt's IPC handler

---

## Conclusion

YouTube has invested heavily in preventing video downloads. Their protections
(n-parameter, TLS fingerprinting, SABR, Web Workers) form a layered defense
that cannot be bypassed with standard browser extension APIs. The only viable
approach requires an external tool like yt-dlp that reverse-engineers
YouTube's player code on each request.
