# Privacy Policy — Bolt Download Manager Browser Extension

**Last updated:** March 7, 2026

## Overview

Bolt Download Manager ("the Extension") is a browser extension that intercepts downloads initiated in your browser and hands them off to the Bolt desktop application installed on your computer. The Extension is designed with privacy as a core principle: all data stays on your machine and is never transmitted to any external server.

## Data Accessed

When a download is initiated in your browser, the Extension accesses the following information:

- **Download URL** — the address of the file being downloaded.
- **Filename** — the name suggested by the browser or server for the downloaded file.
- **Referrer** — the page URL from which the download was triggered.
- **Cookies** — cookies associated with the download domain, used to authenticate the download request so Bolt can resume or accelerate it.

## How Data Is Used

All accessed data is sent exclusively to the **locally installed Bolt desktop application** on your computer via Chrome's Native Messaging API. This communication happens entirely on your local machine through an OS-level IPC channel. The data is used solely to:

- Add the download to Bolt's download queue.
- Authenticate with the server hosting the file (using the forwarded cookies).

## Data Storage

The Extension stores a single preference (`enabled`: true/false) in `chrome.storage.local` to remember whether download interception is turned on or off. No download URLs, cookies, filenames, or any other personal data is stored by the Extension.

## Data Sharing

The Extension does **not**:

- Collect analytics or telemetry.
- Send any data to external servers, third parties, or cloud services.
- Track browsing history or user behavior.
- Use any data for advertising purposes.

All data remains on your local machine and is only shared between the Extension and the Bolt desktop application.

## Permissions Justification

| Permission | Reason |
|---|---|
| `downloads` | Required to detect new downloads and cancel them so Bolt can handle them instead. |
| `cookies` | Required to forward authentication cookies to Bolt so it can download files that require login. |
| `nativeMessaging` | Required to communicate with the Bolt desktop application via the native messaging host. |
| `storage` | Required to persist the on/off toggle preference. |
| `<all_urls>` (host permission) | Required to access cookies for any domain from which a download may originate. Without this, authenticated downloads from arbitrary sites would fail. |

## Open Source

The Extension and the Bolt desktop application are open source. You can review the complete source code at: https://github.com/dasunNimantha/bolt

## Contact

If you have questions about this privacy policy, please open an issue on the GitHub repository: https://github.com/dasunNimantha/bolt/issues

## Changes

Any changes to this privacy policy will be reflected in this document and in the Extension's Chrome Web Store listing.
