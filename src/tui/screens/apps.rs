use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppSource {
    Native,
    Flatpak,
}

#[derive(Debug, Clone)]
pub struct AppItem {
    pub name: String,
    pub id_or_pkg: String,
    pub version: String,
    pub description: String,
    pub source: AppSource,
    pub is_installed: bool,
}

#[derive(Debug, Clone)]
pub struct AppsState {
    pub active_tab: usize, // 0 = Search & Install, 1 = Installed & Remove
    pub search_query: String,
    pub search_input_active: bool,
    pub search_results: Vec<AppItem>,
    pub installed_apps: Vec<AppItem>,
    pub search_idx: usize,
    pub installed_idx: usize,
    pub pkg_manager: &'static str,
    pub flatpak_available: bool,
    pub confirm_action: Option<(String, Vec<String>)>, // (Command, Args) for confirmation dialog
}

impl Default for AppsState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppsState {
    pub fn new() -> Self {
        let pkg_manager = Self::detect_manager();
        let flatpak_available = Command::new("which")
            .arg("flatpak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Self {
            active_tab: 0,
            search_query: String::new(),
            search_input_active: false,
            search_results: Vec::new(),
            installed_apps: Vec::new(),
            search_idx: 0,
            installed_idx: 0,
            pkg_manager,
            flatpak_available,
            confirm_action: None,
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
        self.pkg_manager = Self::detect_manager();
        self.flatpak_available = Command::new("which")
            .arg("flatpak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if self.active_tab == 1 {
            self.load_installed();
        } else if !self.search_query.trim().is_empty() {
            self.perform_search();
        }
    }

    pub fn switch_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 { 1 } else { 0 };
        if self.active_tab == 1 && self.installed_apps.is_empty() {
            self.load_installed();
        }
    }

    pub fn perform_search(&mut self) {
        let query = self.search_query.trim();
        if query.is_empty() {
            self.search_results.clear();
            return;
        }

        let mut results = Vec::new();

        match self.pkg_manager {
            "pacman" => {
                // pacman -Ss <query>: 2 lines per package (header + description)
                if let Ok(out) = Command::new("pacman").args(["-Ss", query]).output() {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        let mut current_header: Option<String> = None;

                        for line in text.lines() {
                            if line.starts_with(' ') || line.starts_with('\t') {
                                if let Some(header) = current_header.take() {
                                    let desc = line.trim().to_string();
                                    let parts: Vec<&str> = header.split_whitespace().collect();
                                    if !parts.is_empty() {
                                        let repo_pkg = parts[0];
                                        let pkg_name =
                                            repo_pkg.split('/').nth(1).unwrap_or(repo_pkg);
                                        let version = parts.get(1).unwrap_or(&"").to_string();
                                        let is_installed = header.contains("[installed");

                                        results.push(AppItem {
                                            name: pkg_name.to_string(),
                                            id_or_pkg: pkg_name.to_string(),
                                            version,
                                            description: desc,
                                            source: AppSource::Native,
                                            is_installed,
                                        });
                                    }
                                }
                            } else if !line.trim().is_empty() {
                                current_header = Some(line.to_string());
                            }
                        }
                    }
                }
            }
            "apt" => {
                // apt search <query> or apt-cache search
                if let Ok(out) = Command::new("apt-cache").args(["search", query]).output() {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        for line in text.lines().take(30) {
                            if let Some((pkg, desc)) = line.split_once(" - ") {
                                results.push(AppItem {
                                    name: pkg.trim().to_string(),
                                    id_or_pkg: pkg.trim().to_string(),
                                    version: String::new(),
                                    description: desc.trim().to_string(),
                                    source: AppSource::Native,
                                    is_installed: false,
                                });
                            }
                        }
                    }
                }
            }
            "dnf" => {
                if let Ok(out) = Command::new("dnf").args(["search", query]).output() {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        for line in text.lines().take(30) {
                            if let Some((pkg, desc)) = line.split_once(" : ") {
                                results.push(AppItem {
                                    name: pkg.trim().to_string(),
                                    id_or_pkg: pkg.trim().to_string(),
                                    version: String::new(),
                                    description: desc.trim().to_string(),
                                    source: AppSource::Native,
                                    is_installed: false,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        if self.flatpak_available {
            if let Ok(out) = Command::new("flatpak").args(["search", query]).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines().take(20) {
                        let cols: Vec<&str> = line.split('\t').collect();
                        if cols.len() >= 3 {
                            let name = cols[0].trim().to_string();
                            let desc = cols[1].trim().to_string();
                            let app_id = cols[2].trim().to_string();
                            let ver = cols.get(3).map(|v| v.trim()).unwrap_or("").to_string();

                            // Filter out headers
                            if app_id != "Application ID"
                                && app_id != "ID Приложения"
                                && !app_id.is_empty()
                            {
                                results.push(AppItem {
                                    name,
                                    id_or_pkg: app_id,
                                    version: ver,
                                    description: desc,
                                    source: AppSource::Flatpak,
                                    is_installed: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        self.search_results = results;
        self.search_idx = 0;
    }

    pub fn load_installed(&mut self) {
        let mut list = Vec::new();

        if self.flatpak_available {
            if let Ok(out) = Command::new("flatpak")
                .args(["list", "--app", "--columns=name,application,version"])
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for (i, line) in text.lines().enumerate() {
                        if i == 0 || line.trim().is_empty() {
                            continue; // Skip table header
                        }
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Flatpak columns might have multi-word names; app id is reverse-DNS (contains '.')
                            let mut app_id = String::new();
                            let mut app_id_idx = 0;
                            for (idx, p) in parts.iter().enumerate() {
                                if p.contains('.') && p.chars().any(|c| c.is_ascii_alphabetic()) {
                                    app_id = p.to_string();
                                    app_id_idx = idx;
                                    break;
                                }
                            }

                            if !app_id.is_empty() {
                                let name = parts[..app_id_idx].join(" ");
                                let version = parts.get(app_id_idx + 1).unwrap_or(&"").to_string();

                                list.push(AppItem {
                                    name: if name.is_empty() {
                                        app_id.clone()
                                    } else {
                                        name
                                    },
                                    id_or_pkg: app_id,
                                    version,
                                    description: "Flatpak Application".to_string(),
                                    source: AppSource::Flatpak,
                                    is_installed: true,
                                });
                            }
                        }
                    }
                }
            }
        }

        match self.pkg_manager {
            "pacman" => {
                // pacman -Qe: only explicitly installed packages
                if let Ok(out) = Command::new("pacman").args(["-Qe"]).output() {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        for line in text.lines() {
                            if let Some((pkg, ver)) = line.split_once(' ') {
                                list.push(AppItem {
                                    name: pkg.trim().to_string(),
                                    id_or_pkg: pkg.trim().to_string(),
                                    version: ver.trim().to_string(),
                                    description: "Pacman explicit package".to_string(),
                                    source: AppSource::Native,
                                    is_installed: true,
                                });
                            }
                        }
                    }
                }
            }
            "apt" => {
                if let Ok(out) = Command::new("dpkg-query")
                    .args(["-W", "-f=${Package}\t${Version}\t${binary:Summary}\n"])
                    .output()
                {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        for line in text.lines().take(150) {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                let pkg = parts[0].trim().to_string();
                                let ver = parts[1].trim().to_string();
                                let desc = parts.get(2).unwrap_or(&"").trim().to_string();
                                list.push(AppItem {
                                    name: pkg.clone(),
                                    id_or_pkg: pkg,
                                    version: ver,
                                    description: desc,
                                    source: AppSource::Native,
                                    is_installed: true,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        self.installed_apps = list;
        if self.installed_idx >= self.installed_apps.len() && !self.installed_apps.is_empty() {
            self.installed_idx = self.installed_apps.len() - 1;
        }
    }

    pub fn next(&mut self) {
        if self.active_tab == 0 {
            if !self.search_results.is_empty() {
                if self.search_idx + 1 < self.search_results.len() {
                    self.search_idx += 1;
                } else {
                    self.search_idx = 0;
                }
            }
        } else if !self.installed_apps.is_empty() {
            if self.installed_idx + 1 < self.installed_apps.len() {
                self.installed_idx += 1;
            } else {
                self.installed_idx = 0;
            }
        }
    }

    pub fn prev(&mut self) {
        if self.active_tab == 0 {
            if !self.search_results.is_empty() {
                if self.search_idx > 0 {
                    self.search_idx -= 1;
                } else {
                    self.search_idx = self.search_results.len() - 1;
                }
            }
        } else if !self.installed_apps.is_empty() {
            if self.installed_idx > 0 {
                self.installed_idx -= 1;
            } else {
                self.installed_idx = self.installed_apps.len() - 1;
            }
        }
    }

    #[allow(dead_code)]
    pub fn selected_item(&self) -> Option<&AppItem> {
        if self.active_tab == 0 {
            self.search_results.get(self.search_idx)
        } else {
            self.installed_apps.get(self.installed_idx)
        }
    }

    pub fn prepare_install_command(&self) -> Option<(String, Vec<String>)> {
        let item = self.search_results.get(self.search_idx)?;
        match item.source {
            AppSource::Flatpak => {
                // Policy: install flatpak apps in --user mode by default for safety without root password
                Some((
                    "flatpak".to_string(),
                    vec![
                        "install".to_string(),
                        "--user".to_string(),
                        "-y".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                ))
            }
            AppSource::Native => match self.pkg_manager {
                "pacman" => Some((
                    "sudo".to_string(),
                    vec![
                        "pacman".to_string(),
                        "-S".to_string(),
                        "-y".to_string(),
                        "--noconfirm".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                "apt" => Some((
                    "sudo".to_string(),
                    vec![
                        "apt".to_string(),
                        "install".to_string(),
                        "-y".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                "dnf" => Some((
                    "sudo".to_string(),
                    vec![
                        "dnf".to_string(),
                        "install".to_string(),
                        "-y".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                _ => None,
            },
        }
    }

    pub fn prepare_remove_command(&self) -> Option<(String, Vec<String>)> {
        let item = self.installed_apps.get(self.installed_idx)?;
        match item.source {
            AppSource::Flatpak => Some((
                "flatpak".to_string(),
                vec![
                    "uninstall".to_string(),
                    "-y".to_string(),
                    item.id_or_pkg.clone(),
                ],
            )),
            AppSource::Native => match self.pkg_manager {
                "pacman" => Some((
                    "sudo".to_string(),
                    vec![
                        "pacman".to_string(),
                        "-R".to_string(),
                        "--noconfirm".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                "apt" => Some((
                    "sudo".to_string(),
                    vec![
                        "apt".to_string(),
                        "remove".to_string(),
                        "-y".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                "dnf" => Some((
                    "sudo".to_string(),
                    vec![
                        "dnf".to_string(),
                        "remove".to_string(),
                        "-y".to_string(),
                        item.id_or_pkg.clone(),
                    ],
                )),
                _ => None,
            },
        }
    }
}
