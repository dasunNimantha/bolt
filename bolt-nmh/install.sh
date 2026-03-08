#!/usr/bin/env bash
set -euo pipefail

MANIFEST_NAME="com.bolt.nmh.json"
FIREFOX_EXTENSION_ID="bolt@boltdm.site"

usage() {
    echo "Usage: $0 <chrome-extension-id> [bolt-nmh-path]"
    echo ""
    echo "  chrome-extension-id  Chrome extension ID (from chrome://extensions)"
    echo "  bolt-nmh-path        Path to bolt-nmh binary (default: auto-detect from cargo)"
    echo ""
    echo "Installs the native messaging host manifest for Chrome/Chromium and Firefox."
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

EXTENSION_ID="$1"

if [ $# -ge 2 ]; then
    NMH_PATH="$(realpath "$2")"
else
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    if [ -f "$SCRIPT_DIR/../target/release/bolt-nmh" ]; then
        NMH_PATH="$SCRIPT_DIR/../target/release/bolt-nmh"
    elif [ -f "$SCRIPT_DIR/../target/debug/bolt-nmh" ]; then
        NMH_PATH="$SCRIPT_DIR/../target/debug/bolt-nmh"
    else
        echo "Error: bolt-nmh binary not found. Build it first with:"
        echo "  cargo build -p bolt-nmh --release"
        exit 1
    fi
    NMH_PATH="$(realpath "$NMH_PATH")"
fi

if [ ! -x "$NMH_PATH" ]; then
    echo "Error: $NMH_PATH is not executable"
    exit 1
fi

case "$(uname -s)" in
    Linux)
        CHROME_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
        CHROMIUM_DIR="$HOME/.config/chromium/NativeMessagingHosts"
        FIREFOX_DIR="$HOME/.mozilla/native-messaging-hosts"
        ;;
    Darwin)
        CHROME_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
        CHROMIUM_DIR="$HOME/Library/Application Support/Chromium/NativeMessagingHosts"
        FIREFOX_DIR="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts"
        ;;
    *)
        echo "Error: Unsupported OS. On Windows, use install.ps1 instead."
        exit 1
        ;;
esac

CHROME_MANIFEST=$(cat <<EOF
{
  "name": "com.bolt.nmh",
  "description": "Bolt Download Manager Native Messaging Host",
  "path": "$NMH_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF
)

FIREFOX_MANIFEST=$(cat <<EOF
{
  "name": "com.bolt.nmh",
  "description": "Bolt Download Manager Native Messaging Host",
  "path": "$NMH_PATH",
  "type": "stdio",
  "allowed_extensions": [
    "$FIREFOX_EXTENSION_ID"
  ]
}
EOF
)

installed=0

# Install for Chrome/Chromium
for DIR in "$CHROME_DIR" "$CHROMIUM_DIR"; do
    BROWSER_NAME="$(basename "$(dirname "$DIR")")"
    if [ -d "$(dirname "$DIR")" ]; then
        mkdir -p "$DIR"
        echo "$CHROME_MANIFEST" > "$DIR/$MANIFEST_NAME"
        echo "Installed for $BROWSER_NAME: $DIR/$MANIFEST_NAME"
        installed=1
    fi
done

# Install for Firefox
if [ -d "$(dirname "$FIREFOX_DIR")" ] || [ -d "$FIREFOX_DIR" ]; then
    mkdir -p "$FIREFOX_DIR"
    echo "$FIREFOX_MANIFEST" > "$FIREFOX_DIR/$MANIFEST_NAME"
    echo "Installed for Firefox: $FIREFOX_DIR/$MANIFEST_NAME"
    installed=1
fi

if [ "$installed" -eq 0 ]; then
    mkdir -p "$CHROME_DIR"
    echo "$CHROME_MANIFEST" > "$CHROME_DIR/$MANIFEST_NAME"
    echo "Installed for Chrome: $CHROME_DIR/$MANIFEST_NAME"

    mkdir -p "$FIREFOX_DIR"
    echo "$FIREFOX_MANIFEST" > "$FIREFOX_DIR/$MANIFEST_NAME"
    echo "Installed for Firefox: $FIREFOX_DIR/$MANIFEST_NAME"
fi

echo ""
echo "Done. The native messaging host points to: $NMH_PATH"
echo "Restart your browser(s) for changes to take effect."
