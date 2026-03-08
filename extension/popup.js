const NATIVE_HOST = "com.bolt.nmh";
const toggle = document.getElementById("enableToggle");
const cookieToggle = document.getElementById("cookieToggle");
const statusEl = document.getElementById("status");

const COOKIE_PERMS = {
  permissions: ["cookies"],
  origins: ["<all_urls>"],
};

chrome.storage.local.get({ enabled: true }, (data) => {
  toggle.checked = data.enabled;
});

chrome.permissions.contains(COOKIE_PERMS, (granted) => {
  cookieToggle.checked = granted;
});

toggle.addEventListener("change", () => {
  chrome.storage.local.set({ enabled: toggle.checked });
});

cookieToggle.addEventListener("change", () => {
  if (cookieToggle.checked) {
    chrome.permissions.request(COOKIE_PERMS, (granted) => {
      cookieToggle.checked = granted;
    });
  } else {
    chrome.permissions.remove(COOKIE_PERMS, () => {
      cookieToggle.checked = false;
    });
  }
});

chrome.runtime.sendNativeMessage(NATIVE_HOST, { ping: true }, (response) => {
  if (chrome.runtime.lastError) {
    const msg = chrome.runtime.lastError.message || "";
    if (msg.includes("not found") || msg.includes("not installed")) {
      statusEl.textContent = "Native host not installed";
    } else {
      statusEl.textContent = "Cannot connect to native host";
    }
    statusEl.className = "status disconnected";
    return;
  }

  if (response && response.status === "ok") {
    statusEl.textContent = "Connected to Bolt";
    statusEl.className = "status connected";
  } else if (response && response.message === "Bolt is not running") {
    statusEl.textContent = "Bolt is not running";
    statusEl.className = "status disconnected";
  } else {
    statusEl.textContent = "Connected (host ready)";
    statusEl.className = "status connected";
  }
});
