use std::fs;

#[derive(Debug, Clone)]
pub struct TempSensor {
    #[allow(dead_code)]
    pub name: String,
    pub label: String,
    pub temp_c: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub capacity_percent: u8,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsState {
    pub temperatures: Vec<TempSensor>,
    pub battery: Option<BatteryInfo>,
    pub kernel_version: String,
    pub kernel_tainted: bool,
    pub swap_total_mb: u64,
    pub swap_used_mb: u64,
    pub swap_percent: f32,
}

impl Default for DiagnosticsState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            temperatures: Vec::new(),
            battery: None,
            kernel_version: String::new(),
            kernel_tainted: false,
            swap_total_mb: 0,
            swap_used_mb: 0,
            swap_percent: 0.0,
        }
    }

    pub fn load(&mut self) {
        self.temperatures = Self::collect_temperatures();
        self.battery = Self::collect_battery();
        self.kernel_version = fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        let tainted = fs::read_to_string("/proc/sys/kernel/tainted")
            .unwrap_or_default()
            .trim()
            .parse::<u64>()
            .unwrap_or(0);
        self.kernel_tainted = tainted != 0;

        self.collect_swap();
    }

    fn collect_temperatures() -> Vec<TempSensor> {
        let mut sensors = Vec::new();

        // hwmon sensors
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let dir_path = entry.path();
                let name = fs::read_to_string(dir_path.join("name"))
                    .unwrap_or_else(|_| "hwmon".to_string())
                    .trim()
                    .to_string();

                if let Ok(sub_entries) = fs::read_dir(&dir_path) {
                    for sub in sub_entries.flatten() {
                        let file_name = sub.file_name().to_string_lossy().to_string();
                        if file_name.starts_with("temp") && file_name.ends_with("_input") {
                            let prefix = file_name.trim_end_matches("_input");
                            let label_file = dir_path.join(format!("{prefix}_label"));
                            let label = fs::read_to_string(label_file)
                                .unwrap_or_else(|_| prefix.to_string())
                                .trim()
                                .to_string();

                            if let Ok(content) = fs::read_to_string(sub.path()) {
                                if let Ok(val) = content.trim().parse::<f32>() {
                                    sensors.push(TempSensor {
                                        name: name.clone(),
                                        label,
                                        temp_c: val / 1000.0,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // thermal_zone sensors
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            for entry in entries.flatten() {
                let dir_path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("thermal_zone") {
                    let zone_type = fs::read_to_string(dir_path.join("type"))
                        .unwrap_or_else(|_| file_name.clone())
                        .trim()
                        .to_string();

                    if let Ok(content) = fs::read_to_string(dir_path.join("temp")) {
                        if let Ok(val) = content.trim().parse::<f32>() {
                            sensors.push(TempSensor {
                                name: "thermal".to_string(),
                                label: zone_type,
                                temp_c: val / 1000.0,
                            });
                        }
                    }
                }
            }
        }

        sensors
    }

    fn collect_battery() -> Option<BatteryInfo> {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let p = entry.path();
                let supply_type = fs::read_to_string(p.join("type"))
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase();

                if supply_type == "battery" {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let cap = fs::read_to_string(p.join("capacity"))
                        .unwrap_or_default()
                        .trim()
                        .parse::<u8>()
                        .unwrap_or(0);
                    let status = fs::read_to_string(p.join("status"))
                        .unwrap_or_else(|_| "Unknown".to_string())
                        .trim()
                        .to_string();

                    return Some(BatteryInfo {
                        name,
                        capacity_percent: cap,
                        status,
                    });
                }
            }
        }
        None
    }

    fn collect_swap(&mut self) {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb: u64 = 0;
            let mut free_kb: u64 = 0;

            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                    if let Ok(val) = v.parse::<u64>() {
                        match k {
                            "SwapTotal:" => total_kb = val,
                            "SwapFree:" => free_kb = val,
                            _ => {}
                        }
                    }
                }
            }

            self.swap_total_mb = total_kb / 1024;
            let used_kb = total_kb.saturating_sub(free_kb);
            self.swap_used_mb = used_kb / 1024;
            self.swap_percent = if total_kb > 0 {
                ((used_kb as f32 / total_kb as f32) * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            };
        }
    }
}
