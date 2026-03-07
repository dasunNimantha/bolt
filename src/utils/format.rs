pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec <= 0.0 {
        return "—".to_string();
    }
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

pub fn format_eta(seconds: u64) -> String {
    if seconds >= 3600 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    } else if seconds >= 60 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", seconds)
    }
}

pub fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        let half = (max_len - 3) / 2;
        format!("{}...{}", &url[..half], &url[url.len() - half..])
    }
}

pub fn truncate_filename(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        return name.to_string();
    }
    if let Some(dot_pos) = name.rfind('.') {
        let ext = &name[dot_pos..];
        let stem_budget = max_len.saturating_sub(ext.len() + 1);
        if stem_budget > 0 {
            return format!("{}…{}", &name[..stem_budget], ext);
        }
    }
    format!("{}…", &name[..max_len.saturating_sub(1)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn format_bytes_bytes() {
        assert_eq!(format_bytes(512), "512 B");
    }

    #[test]
    fn format_bytes_kilobytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
    }

    #[test]
    fn format_bytes_megabytes() {
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(10 * 1024 * 1024), "10.0 MB");
    }

    #[test]
    fn format_bytes_gigabytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn format_bytes_terabytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.00 TB");
    }

    #[test]
    fn format_speed_zero() {
        assert_eq!(format_speed(0.0), "—");
    }

    #[test]
    fn format_speed_negative() {
        assert_eq!(format_speed(-100.0), "—");
    }

    #[test]
    fn format_speed_normal() {
        assert_eq!(format_speed(1024.0), "1.0 KB/s");
        assert_eq!(format_speed(5.0 * 1024.0 * 1024.0), "5.0 MB/s");
    }

    #[test]
    fn format_eta_seconds() {
        assert_eq!(format_eta(45), "45s");
    }

    #[test]
    fn format_eta_minutes() {
        assert_eq!(format_eta(125), "2m 5s");
    }

    #[test]
    fn format_eta_hours() {
        assert_eq!(format_eta(3661), "1h 1m");
    }

    #[test]
    fn truncate_url_short() {
        let url = "https://example.com";
        assert_eq!(truncate_url(url, 100), url);
    }

    #[test]
    fn truncate_url_long() {
        let url = "https://example.com/very/long/path/to/some/resource/file.zip";
        let result = truncate_url(url, 30);
        assert!(result.len() <= 30);
        assert!(result.contains("..."));
    }

    #[test]
    fn truncate_filename_short() {
        assert_eq!(truncate_filename("short.txt", 20), "short.txt");
    }

    #[test]
    fn truncate_filename_long_with_ext() {
        let name = "a_very_long_filename_that_exceeds_the_limit.mp4";
        let result = truncate_filename(name, 20);
        assert!(result.chars().count() <= 20);
        assert!(result.ends_with(".mp4"));
        assert!(result.contains('…'));
    }

    #[test]
    fn truncate_filename_no_extension() {
        let name = "a_very_long_filename_without_any_extension_at_all";
        let result = truncate_filename(name, 15);
        assert!(result.contains('…'));
    }
}
