use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

pub enum TrayAction {
    Show,
    Quit,
}

pub struct BoltTray {
    _icon: tray_icon::TrayIcon,
    show_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
}

impl BoltTray {
    pub fn new() -> Option<Self> {
        #[cfg(target_os = "linux")]
        {
            let _ = gtk::init();
        }

        let icon = Icon::from_rgba(create_icon_rgba(), 32, 32).ok()?;

        let show_item = MenuItem::new("Show Bolt", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        let menu = Menu::new();
        let _ = menu.append(&show_item);
        let _ = menu.append(&quit_item);

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Bolt Download Manager")
            .with_icon(icon)
            .build()
            .ok()?;

        // Flush GTK events immediately so the icon renders on startup
        #[cfg(target_os = "linux")]
        {
            for _ in 0..50 {
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        Some(Self {
            _icon: tray,
            show_id,
            quit_id,
        })
    }

    /// Process pending GTK events (required on Linux for the tray icon to
    /// appear and respond to clicks) and check for menu actions.
    pub fn poll(&self) -> Option<TrayAction> {
        #[cfg(target_os = "linux")]
        {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_id {
                return Some(TrayAction::Show);
            }
            if event.id == self.quit_id {
                return Some(TrayAction::Quit);
            }
        }
        None
    }

    pub fn set_tooltip(&self, tip: &str) {
        let _ = self._icon.set_tooltip(Some(tip));
    }
}

fn create_icon_rgba() -> Vec<u8> {
    let size = 32u32;
    let mut data = vec![0u8; (size * size * 4) as usize];
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let r = 15.5f32;

    // Lightning bolt polygon (from extension SVG scaled 48→32)
    let bolt: &[(f32, f32)] = &[
        (18.0, 4.0),
        (10.0, 18.0),
        (15.0, 18.0),
        (13.0, 28.0),
        (21.0, 14.0),
        (16.0, 14.0),
    ];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) as usize * 4;
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let dx = fx - cx;
            let dy = fy - cy;

            if dx * dx + dy * dy > r * r {
                continue;
            }

            if point_in_polygon(fx, fy, bolt) {
                // Black bolt: #1A1A1A
                data[idx] = 0x1A;
                data[idx + 1] = 0x1A;
                data[idx + 2] = 0x1A;
                data[idx + 3] = 0xFF;
            } else {
                // Gold background: #F2BF40
                data[idx] = 0xF2;
                data[idx + 1] = 0xBF;
                data[idx + 2] = 0x40;
                data[idx + 3] = 0xFF;
            }
        }
    }

    data
}

fn point_in_polygon(px: f32, py: f32, poly: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
