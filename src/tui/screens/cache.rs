use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct CacheState {
    pub pkg_cache_str: String,
    pub journal_cache_str: String,
    pub user_cache_str: String,
    pub confirm_clean: bool,
    #[allow(dead_code)]
    pub clean_status: Option<String>,
}

impl Default for CacheState {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            pkg_cache_str: "—".to_string(),
            journal_cache_str: "—".to_string(),
            user_cache_str: "—".to_string(),
            confirm_clean: false,
            clean_status: None,
        }
    }

    pub fn load(&mut self) {
        self.pkg_cache_str = Self::measure_pkg_cache();
        self.journal_cache_str = Self::measure_journal();
        self.user_cache_str = Self::measure_user_cache();
        self.confirm_clean = false;
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.0} KB", bytes as f64 / 1024.0)
        }
    }

    fn dir_size<P: AsRef<Path>>(path: P) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total += meta.len();
                    } else if meta.is_dir() {
                        total += Self::dir_size(entry.path());
                    }
                }
            }
        }
        total
    }

    fn measure_pkg_cache() -> String {
        let paths = [
            "/var/cache/pacman/pkg",
            "/var/cache/apt/archives",
            "/var/cache/dnf",
        ];

        let mut found_bytes = 0;
        let mut any_found = false;

        for p in &paths {
            if Path::new(p).exists() {
                any_found = true;
                found_bytes += Self::dir_size(p);
            }
        }

        if any_found {
            Self::format_bytes(found_bytes)
        } else {
            "0 MB".to_string()
        }
    }

    fn measure_journal() -> String {
        if let Ok(out) = Command::new("journalctl").arg("--disk-usage").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                // "Archived and active journals take up 3.9G in the file system."
                if let Some(pos) = text.find("take up ") {
                    let after = &text[pos + 8..];
                    if let Some(end) = after.find(" in") {
                        return after[..end].trim().to_string();
                    }
                }
                return text.trim().to_string();
            }
        }
        "—".to_string()
    }

    fn measure_user_cache() -> String {
        if let Ok(home) = std::env::var("HOME") {
            let thumb = format!("{home}/.cache/thumbnails");
            let size = Self::dir_size(&thumb);
            return Self::format_bytes(size);
        }
        "0 MB".to_string()
    }

    pub fn clean_command(&self) -> (&'static str, Vec<&'static str>) {
        if Command::new("which")
            .arg("pacman")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            ("sudo", vec!["pacman", "-Sc", "--noconfirm"])
        } else if Command::new("which")
            .arg("apt-get")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            ("sudo", vec!["apt-get", "clean"])
        } else if Command::new("which")
            .arg("dnf")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            ("sudo", vec!["dnf", "clean", "all"])
        } else {
            ("echo", vec!["No package manager cache cleaner"])
        }
    }
}
