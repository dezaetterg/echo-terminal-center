use std::process::Command;

#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub filesystem: String,
    pub size: String,
    pub used: String,
    pub avail: String,
    pub use_percent: u8,
    pub mount: String,
}

#[derive(Debug, Clone)]
pub struct DisksState {
    pub partitions: Vec<PartitionInfo>,
    pub smart_status: String,
}

impl Default for DisksState {
    fn default() -> Self {
        Self::new()
    }
}

impl DisksState {
    pub fn new() -> Self {
        Self {
            partitions: Vec::new(),
            smart_status: String::new(),
        }
    }

    pub fn load(&mut self) {
        self.partitions = Self::collect_partitions();
        self.smart_status = Self::check_smart();
    }

    fn collect_partitions() -> Vec<PartitionInfo> {
        let mut list = Vec::new();
        let output = Command::new("df")
            .args([
                "-h", "-P", "-x", "tmpfs", "-x", "devtmpfs", "-x", "squashfs", "-x", "efivarfs",
            ])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut lines = text.lines();
                // Skip header line
                let _ = lines.next();

                for line in lines {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let filesystem = parts[0].to_string();
                        let size = parts[1].to_string();
                        let used = parts[2].to_string();
                        let avail = parts[3].to_string();
                        let pct_str = parts[4].trim_end_matches('%');
                        let use_percent = pct_str.parse::<u8>().unwrap_or(0);
                        let mount = parts[5..].join(" ");

                        list.push(PartitionInfo {
                            filesystem,
                            size,
                            used,
                            avail,
                            use_percent,
                            mount,
                        });
                    }
                }
            }
        }

        list
    }

    fn check_smart() -> String {
        // First check if smartctl binary exists
        let which = Command::new("which").arg("smartctl").output();
        let exists = match which {
            Ok(w) => w.status.success(),
            Err(_) => false,
        };

        if !exists {
            return "smartctl не установлен (установите пакет smartmontools)".to_string();
        }

        // Try sudo -n smartctl on root disk
        let res = Command::new("sudo")
            .args(["-n", "smartctl", "-H", "/dev/nvme0n1"])
            .output();

        match res {
            Ok(out) if out.status.success() => {
                let txt = String::from_utf8_lossy(&out.stdout);
                if txt.contains("PASSED") || txt.contains("OK") {
                    "SMART: Исправен (PASSED)".to_string()
                } else if txt.contains("FAILED") {
                    "SMART: Внимание! Обнаружены ошибки (FAILED)".to_string()
                } else {
                    "SMART: Доступен".to_string()
                }
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                if err.contains("password") || out.status.code() == Some(1) {
                    "SMART: Требуются права root (настройте sudoers)".to_string()
                } else {
                    "SMART: Не удалось прочитать статус".to_string()
                }
            }
            Err(_) => "SMART: Ошибка проверки".to_string(),
        }
    }
}
