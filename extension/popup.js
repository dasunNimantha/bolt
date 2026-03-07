const NATIVE_HOST = "com.bolt.nmh";
const toggle = document.getElementById("enableToggle");
const statusEl = document.getElementById("status");

chrome.storage.local.get({ enabled: true }, (data) => {
  toggle.checked = data.enabled;
});

toggle.addEventListener("change", () => {
  chrome.storage.local.set({ enabled: toggle.checked });
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
