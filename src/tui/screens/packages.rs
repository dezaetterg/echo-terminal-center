use std::process::Command;

#[derive(Debug, Clone)]
pub struct PackagesState {
    pub updates: Vec<String>,
    pub pkg_manager: &'static str,
    pub confirm_upgrade: bool,
    pub scroll_offset: usize,
}

impl Default for PackagesState {
    fn default() -> Self {
        Self::new()
    }
}

impl PackagesState {
    pub fn new() -> Self {
        let pkg_manager = Self::detect_manager();
        Self {
            updates: Vec::new(),
            pkg_manager,
            confirm_upgrade: false,
            scroll_offset: 0,
        }
    }

    fn detect_manager() -> &'static str {
        if Command::new("which")
            .arg("pacman")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "pacman"
        } else if Command::new("which")
            .arg("apt")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "apt"
        } else if Command::new("which")
            .arg("dnf")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            "dnf"
        } else {
            "unknown"
        }
    }

    pub fn load(&mut self) {
        let mut list = Vec::new();
        self.pkg_manager = Self::detect_manager();

        match self.pkg_manager {
            "pacman" => {
                // Try checkupdates first, fall back to pacman -Qu
                let res = Command::new("checkupdates").output();
                let output = match res {
                    Ok(out) if out.status.success() => Some(out.stdout),
                    _ => Command::new("pacman")
                        .args(["-Qu"])
                        .output()
                        .ok()
                        .map(|o| o.stdout),
                };

                if let Some(stdout) = output {
                    let text = String::from_utf8_lossy(&stdout);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            list.push(trimmed.to_string());
                        }
                    }
                }
            }
            "apt" => {
                if let Ok(out) = Command::new("apt").args(["list", "--upgradable"]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        // Ignore header line ("Listing...", "Вывод списка...")
                        if !trimmed.is_empty() && trimmed.contains('[') {
                            list.push(trimmed.to_string());
                        }
                    }
                }
            }
            "dnf" => {
                if let Ok(out) = Command::new("dnf").args(["check-update"]).output() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !trimmed.starts_with("Last metadata") {
                            list.push(trimmed.to_string());
                        }
                    }
                }
            }
            _ => {
                list.push("Пакетный менеджер не определен".to_string());
            }
        }

        // Also check flatpak if available
        if let Ok(out) = Command::new("flatpak")
            .args(["remote-ls", "--updates"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        list.push(format!("[flatpak] {trimmed}"));
                    }
                }
            }
        }

        self.updates = list;
        self.scroll_offset = 0;
        self.confirm_upgrade = false;
    }

    pub fn upgrade_command(&self) -> (&'static str, Vec<&'static str>) {
        match self.pkg_manager {
            "pacman" => ("sudo", vec!["pacman", "-Syu"]),
            "apt" => ("sudo", vec!["apt", "upgrade"]),
            "dnf" => ("sudo", vec!["dnf", "upgrade"]),
            _ => ("echo", vec!["No package manager found"]),
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, visible_lines: usize) {
        if self.updates.len() > visible_lines {
            let max_offset = self.updates.len().saturating_sub(visible_lines);
            if self.scroll_offset < max_offset {
                self.scroll_offset += 1;
            }
        }
    }
}
