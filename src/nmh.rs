use std::path::{Path, PathBuf};

const MANIFEST_NAME: &str = "com.bolt.nmh.json";

/// Extension IDs allowed to connect to the native messaging host.
/// Add the Chrome Web Store extension ID here once published.
const ALLOWED_EXTENSION_IDS: &[&str] = &[
    // Chrome Web Store ID (update after publishing)
    // "chrome-extension://YOUR_STORE_ID_HERE/",
];

/// Automatically install the native messaging host manifest for all
/// detected browsers. Called once on startup — skips browsers that
/// already have the manifest installed.
pub fn auto_install() {
    let nmh_binary = match find_nmh_binary() {
        Some(p) => p,
        None => return,
    };

    if ALLOWED_EXTENSION_IDS.is_empty() {
        install_for_dev_mode(&nmh_binary);
        return;
    }

    let origins: Vec<String> = ALLOWED_EXTENSION_IDS
        .iter()
        .map(|id| id.to_string())
        .collect();

    for dir in nmh_dirs() {
        install_manifest(&dir, &nmh_binary, &origins);
    }
}

/// In dev mode (no store IDs configured), look for existing manifests
/// and refresh the binary path without overwriting the allowed_origins.
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

fn install_manifest(host_dir: &Path, nmh_binary: &Path, origins: &[String]) {
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

    // Cargo workspace: target/release/bolt → target/release/bolt-nmh
    // Already covered above since both land in the same dir.

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
    let mut dirs = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            let config = base.config_dir();
            dirs.push(config.join("google-chrome/NativeMessagingHosts"));
            dirs.push(config.join("chromium/NativeMessagingHosts"));
            dirs.push(config.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"));
            dirs.push(config.join("microsoft-edge/NativeMessagingHosts"));
            dirs.push(config.join("vivaldi/NativeMessagingHosts"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(base) = directories::BaseDirs::new() {
            let home = base.home_dir();
            let lib = home.join("Library/Application Support");
            dirs.push(lib.join("Google/Chrome/NativeMessagingHosts"));
            dirs.push(lib.join("Chromium/NativeMessagingHosts"));
            dirs.push(lib.join("BraveSoftware/Brave-Browser/NativeMessagingHosts"));
            dirs.push(lib.join("Microsoft Edge/NativeMessagingHosts"));
            dirs.push(lib.join("Vivaldi/NativeMessagingHosts"));
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

    // Write manifest JSON next to the binary
    let manifest_dir = nmh_binary.parent().unwrap_or(Path::new("."));
    let manifest_path = manifest_dir.join(MANIFEST_NAME);

    if !manifest_path.exists() && !ALLOWED_EXTENSION_IDS.is_empty() {
        let origins: Vec<String> = ALLOWED_EXTENSION_IDS
            .iter()
            .map(|id| id.to_string())
            .collect();

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

    // Register in HKCU for Chrome and Edge
    let manifest_str = manifest_path.to_string_lossy().replace('/', "\\");
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
}
