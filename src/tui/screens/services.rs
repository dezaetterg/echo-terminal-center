use std::process::Command;

#[derive(Debug, Clone)]
pub struct ServiceItem {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ServicesState {
    pub services: Vec<ServiceItem>,
    pub selected_index: usize,
    pub confirm_action: Option<(String, &'static str)>, // (service_name, action e.g. "restart" or "stop")
}

impl Default for ServicesState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServicesState {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            selected_index: 0,
            confirm_action: None,
        }
    }

    pub fn load(&mut self) {
        let mut list = Vec::new();

        if let Ok(out) = Command::new("systemctl")
            .args(["--failed", "--no-legend", "--no-pager"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        let name = parts[0].trim_end_matches(".service").to_string();
                        list.push(ServiceItem {
                            name,
                            status: "failed".to_string(),
                        });
                    }
                }
            }
        }

        let key_services = [
            "NetworkManager",
            "bluetooth",
            "docker",
            "podman",
            "sshd",
            "cups",
            "systemd-timesyncd",
            "systemd-resolved",
            "sddm",
            "gdm",
            "lightdm",
            "firewalld",
            "ufw",
            "cronie",
            "cron",
            "pipewire",
        ];

        for &srv in &key_services {
            if list.iter().any(|s| s.name == srv) {
                continue;
            }

            if let Ok(out) = Command::new("systemctl").args(["is-active", srv]).output() {
                let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if status == "active" || status == "inactive" || status == "failed" {
                    list.push(ServiceItem {
                        name: srv.to_string(),
                        status,
                    });
                }
            }
        }

        self.services = list;
        self.selected_index = 0;
        self.confirm_action = None;
    }

    pub fn next(&mut self) {
        if !self.services.is_empty() {
            if self.selected_index + 1 < self.services.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn prev(&mut self) {
        if !self.services.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.services.len() - 1;
            }
        }
    }

    pub fn selected_service(&self) -> Option<&ServiceItem> {
        self.services.get(self.selected_index)
    }
}
