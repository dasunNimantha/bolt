use std::path::{Path, PathBuf};

const MANIFEST_NAME: &str = "com.bolt.nmh.json";

/// Chrome extension IDs allowed to connect (allowed_origins format).
/// Add the Chrome Web Store extension ID here once published.
const ALLOWED_CHROME_IDS: &[&str] = &[
    // "chrome-extension://YOUR_STORE_ID_HERE/",
];

/// Firefox extension ID (allowed_extensions format).
const FIREFOX_EXTENSION_ID: &str = "bolt@boltdm.site";

/// Automatically install the native messaging host manifest for all
/// detected browsers. Called once on startup -- skips browsers that
/// already have the manifest installed.
pub fn auto_install() {
    let nmh_binary = match find_nmh_binary() {
        Some(p) => p,
        None => return,
    };

    if ALLOWED_CHROME_IDS.is_empty() {
        install_for_dev_mode(&nmh_binary);
        return;
    }

    let chrome_origins: Vec<String> = ALLOWED_CHROME_IDS.iter().map(|id| id.to_string()).collect();
    let firefox_extensions = vec![FIREFOX_EXTENSION_ID.to_string()];

    for dir in nmh_dirs() {
        if is_firefox_dir(&dir) {
            install_manifest_firefox(&dir, &nmh_binary, &firefox_extensions);
        } else {
            install_manifest_chrome(&dir, &nmh_binary, &chrome_origins);
        }
    }
}

/// In dev mode (no store IDs configured), look for existing manifests
/// and refresh the binary path without overwriting the allowed_origins/allowed_extensions.
fn install_for_dev_mode(nmh_binary: &Path) {
    for dir in nmh_dirs() {
        let manifest_path = dir.join(MANIFEST_NAME);
        if manifest_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&manifest_path) {
                if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&data) {
                    if let Some(obj) = val.as_object_mut() {
                        obj.insert(
                            "path".to_string(),
                            serde_json::Value::String(nmh_binary.to_string_lossy().to_string()),
                        );
                        if let Ok(updated) = serde_json::to_string_pretty(&val) {
                            let _ = std::fs::write(&manifest_path, updated);
                        }
                    }
                }
            }
        }
    }
}

fn install_manifest_chrome(host_dir: &Path, nmh_binary: &Path, origins: &[String]) {
    let manifest_path = host_dir.join(MANIFEST_NAME);
    if manifest_path.exists() {
        return;
    }

    let manifest = serde_json::json!({
        "name": "com.bolt.nmh",
        "description": "Bolt Download Manager Native Messaging Host",
        "path": nmh_binary.to_string_lossy(),
        "type": "stdio",
        "allowed_origins": origins
    });

    if let Some(parent) = manifest_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(&manifest) {
        let _ = std::fs::write(&manifest_path, data);
    }
}

fn install_manifest_firefox(host_dir: &Path, nmh_binary: &Path, extensions: &[String]) {
    let manifest_path = host_dir.join(MANIFEST_NAME);
    if manifest_path.exists() {
        return;
    }

    let manifest = serde_json::json!({
        "name": "com.bolt.nmh",
        "description": "Bolt Download Manager Native Messaging Host",
        "path": nmh_binary.to_string_lossy(),
        "type": "stdio",
        "allowed_extensions": extensions
    });

    if let Some(parent) = manifest_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(&manifest) {
        let _ = std::fs::write(&manifest_path, data);
    }
}

/// Returns true if the NMH directory belongs to a Mozilla/Firefox browser.
fn is_firefox_dir(dir: &Path) -> bool {
    let path_str = dir.to_string_lossy().to_lowercase();
    path_str.contains("mozilla") || path_str.contains("firefox")
}

/// Locate the bolt-nmh binary next to the running bolt executable.
fn find_nmh_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;

    #[cfg(target_os = "windows")]
    let name = "bolt-nmh.exe";
    #[cfg(not(target_os = "windows"))]
    let name = "bolt-nmh";

    let candidate = dir.join(name);
    if candidate.exists() {
        return Some(candidate);
    }

    // Development fallback: check target/release and target/debug
    if let Some(project_root) = dir.parent().and_then(|p| p.parent()) {
        for profile in &["release", "debug"] {
            let path = project_root.join("target").join(profile).join(name);
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

/// Returns all NativeMessagingHosts directories for supported browsers.
fn nmh_dirs() -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut dirs = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            let config = base.config_dir();
            // Chromium-based browsers
            dirs.push(config.join("google-chrome/NativeMessagingHosts"));
            dirs.push(config.join("chromium/NativeMessagingHosts"));
            dirs.push(config.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"));
            dirs.push(config.join("microsoft-edge/NativeMessagingHosts"));
            dirs.push(config.join("vivaldi/NativeMessagingHosts"));
        }
        if let Some(home) = directories::BaseDirs::new().map(|b| b.home_dir().to_path_buf()) {
            // Firefox
            dirs.push(home.join(".mozilla/native-messaging-hosts"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            let home = base.home_dir();
            let lib = home.join("Library/Application Support");
            // Chromium-based browsers
            dirs.push(lib.join("Google/Chrome/NativeMessagingHosts"));
            dirs.push(lib.join("Chromium/NativeMessagingHosts"));
            dirs.push(lib.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"));
            dirs.push(lib.join("Microsoft Edge/NativeMessagingHosts"));
            dirs.push(lib.join("Vivaldi/NativeMessagingHosts"));
            // Firefox
            dirs.push(lib.join("Mozilla/NativeMessagingHosts"));
        }
    }

    #[cfg(target_os = "windows")]
    {
        install_windows_registry();
    }

    dirs
}

/// On Windows, native messaging hosts are registered via the Windows Registry.
#[cfg(target_os = "windows")]
fn install_windows_registry() {
    use std::process::Command;

    let nmh_binary = match find_nmh_binary() {
        Some(p) => p,
        None => return,
    };

    let manifest_dir = nmh_binary.parent().unwrap_or(Path::new("."));
    let manifest_path = manifest_dir.join(MANIFEST_NAME);

    // Chrome manifest (allowed_origins)
    if !manifest_path.exists() && !ALLOWED_CHROME_IDS.is_empty() {
        let origins: Vec<String> = ALLOWED_CHROME_IDS.iter().map(|id| id.to_string()).collect();

        let manifest = serde_json::json!({
            "name": "com.bolt.nmh",
            "description": "Bolt Download Manager Native Messaging Host",
            "path": nmh_binary.to_string_lossy().replace('/', "\\"),
            "type": "stdio",
            "allowed_origins": origins
        });

        if let Ok(data) = serde_json::to_string_pretty(&manifest) {
            let _ = std::fs::write(&manifest_path, data);
        }
    }

    // Firefox manifest (allowed_extensions) — separate file next to chrome manifest
    let firefox_manifest_path = manifest_dir.join("com.bolt.nmh.firefox.json");
    if !firefox_manifest_path.exists() {
        let manifest = serde_json::json!({
            "name": "com.bolt.nmh",
            "description": "Bolt Download Manager Native Messaging Host",
            "path": nmh_binary.to_string_lossy().replace('/', "\\"),
            "type": "stdio",
            "allowed_extensions": [FIREFOX_EXTENSION_ID]
        });

        if let Ok(data) = serde_json::to_string_pretty(&manifest) {
            let _ = std::fs::write(&firefox_manifest_path, data);
        }
    }

    let manifest_str = manifest_path.to_string_lossy().replace('/', "\\");
    let firefox_manifest_str = firefox_manifest_path.to_string_lossy().replace('/', "\\");

    // Register in HKCU for Chrome and Edge
    for key in &[
        r"Software\Google\Chrome\NativeMessagingHosts\com.bolt.nmh",
        r"Software\Microsoft\Edge\NativeMessagingHosts\com.bolt.nmh",
    ] {
        let _ = Command::new("reg")
            .args([
                "add",
                &format!("HKCU\\{}", key),
                "/ve",
                "/t",
                "REG_SZ",
                "/d",
                &manifest_str,
                "/f",
            ])
            .output();
    }

    // Register in HKCU for Firefox
    let _ = Command::new("reg")
        .args([
            "add",
            r"HKCU\Software\Mozilla\NativeMessagingHosts\com.bolt.nmh",
            "/ve",
            "/t",
            "REG_SZ",
            "/d",
            &firefox_manifest_str,
            "/f",
        ])
        .output();
}
