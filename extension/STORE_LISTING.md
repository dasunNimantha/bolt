# Chrome Web Store Listing — Bolt Download Manager

> Copy-paste the sections below into the Chrome Developer Dashboard when submitting.

---

## Short Description (132 chars max)

Supercharge your downloads — Bolt intercepts browser downloads and accelerates them with multi-segment parallel downloading.

---

## Detailed Description

Bolt Download Manager is a companion extension for the Bolt desktop application — a fast, multi-threaded download manager built with Rust.

When installed alongside the Bolt desktop app, this extension automatically intercepts downloads from your browser and sends them to Bolt, where they are accelerated using multi-segment parallel downloading with up to 8 simultaneous connections per file.

HOW IT WORKS

1. A download starts in your browser.
2. The extension intercepts it and sends the URL (along with cookies for authenticated downloads) to the Bolt desktop app.
3. A confirmation popup appears in Bolt where you can choose to start the download immediately, add it to the queue, or cancel.
4. Bolt downloads the file at maximum speed using parallel segments.

FEATURES

- One-click interception — all browser downloads are automatically captured.
- Toggle on/off — disable interception any time from the extension popup.
- Cookie forwarding — authenticated downloads (behind logins) work seamlessly.
- Referrer forwarding — ensures downloads that check the referring page succeed.
- Connection status — the popup shows whether the Bolt desktop app is running.

REQUIREMENTS

This extension requires the Bolt desktop application to be installed on your computer. Bolt is free and open source, available for Linux, Windows, and macOS.

Download Bolt: https://github.com/dasunNimantha/bolt/releases

PRIVACY

This extension does NOT collect any data. All information (URLs, cookies) is sent only to the locally installed Bolt app on your computer via Chrome's Native Messaging API. Nothing leaves your machine. No analytics, no tracking, no telemetry.

The full privacy policy and source code are available at: https://github.com/dasunNimantha/bolt

PERMISSIONS EXPLAINED

- "Manage your downloads" — needed to detect and intercept downloads.
- "Read and change all your data on all websites" — needed to read cookies for authenticated downloads from any site. No browsing data is collected or stored.
- "Communicate with cooperating native applications" — needed to talk to the Bolt desktop app.

---

## Category

Productivity

---

## Tags (up to 5)

download manager, download accelerator, multi-threaded, file downloader, browser integration
