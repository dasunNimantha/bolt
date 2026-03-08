use std::path::PathBuf;

fn exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::fs;

    fn autostart_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.config_dir().join("autostart").join("bolt.desktop"))
    }

    pub fn set_enabled(enabled: bool) {
        let Some(path) = autostart_path() else {
            return;
        };
        if enabled {
            let Some(exe) = exe_path() else { return };
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let desktop_entry = format!(
                "\
[Desktop Entry]
Type=Application
Name=Bolt
Comment=Multi-segment download manager
Exec={exe}
Terminal=false
StartupNotify=false
X-GNOME-Autostart-enabled=true
",
                exe = exe.display()
            );
            let _ = fs::write(&path, desktop_entry);
        } else {
            let _ = fs::remove_file(&path);
        }
    }

    pub fn is_enabled() -> bool {
        autostart_path().is_some_and(|p| p.exists())
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::fs;

    const PLIST_LABEL: &str = "com.bolt.app";

    fn plist_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| {
            d.home_dir()
                .join("Library")
                .join("LaunchAgents")
                .join(format!("{PLIST_LABEL}.plist"))
        })
    }

    pub fn set_enabled(enabled: bool) {
        let Some(path) = plist_path() else { return };
        if enabled {
            let Some(exe) = exe_path() else { return };
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let plist = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
                label = PLIST_LABEL,
                exe = exe.display()
            );
            let _ = fs::write(&path, plist);
        } else {
            let _ = fs::remove_file(&path);
        }
    }

    pub fn is_enabled() -> bool {
        plist_path().is_some_and(|p| p.exists())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;

    const REG_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APP_NAME: &str = "bolt";

    pub fn set_enabled(enabled: bool) {
        use std::process::Command;

        if enabled {
            let Some(exe) = exe_path() else { return };
            let exe_str = exe.to_string_lossy().replace('/', "\\");
            let _ = Command::new("reg")
                .args([
                    "add",
                    &format!("HKCU\\{REG_KEY}"),
                    "/v",
                    APP_NAME,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_str),
                    "/f",
                ])
                .output();
        } else {
            let _ = Command::new("reg")
                .args(["delete", &format!("HKCU\\{REG_KEY}"), "/v", APP_NAME, "/f"])
                .output();
        }
    }

    pub fn is_enabled() -> bool {
        use std::process::Command;

        Command::new("reg")
            .args(["query", &format!("HKCU\\{REG_KEY}"), "/v", APP_NAME])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod platform {
    pub fn set_enabled(_enabled: bool) {}
    pub fn is_enabled() -> bool {
        false
    }
}

pub fn set_enabled(enabled: bool) {
    platform::set_enabled(enabled);
}

pub fn is_enabled() -> bool {
    platform::is_enabled()
}
