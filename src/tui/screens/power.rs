use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String,
    pub percent: u8,
    pub status: String,
    pub health_percent: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct PowerState {
    pub has_battery: bool,
    pub battery: Option<BatteryInfo>,
    pub has_powerprofiles: bool,
    pub current_profile: String,
    pub profiles: Vec<String>,
    pub selected_profile_idx: usize,
    pub has_brightness: bool,
    pub brightness_percent: u8,
    pub is_ac_online: bool,
}

impl Default for PowerState {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerState {
    pub fn new() -> Self {
        Self {
            has_battery: false,
            battery: None,
            has_powerprofiles: false,
            current_profile: String::new(),
            profiles: Vec::new(),
            selected_profile_idx: 0,
            has_brightness: false,
            brightness_percent: 0,
            is_ac_online: true,
        }
    }

    pub fn load(&mut self) {
        self.load_battery();
        self.load_power_profiles();
        self.load_brightness();
    }

    fn load_battery(&mut self) {
        let ps_path = Path::new("/sys/class/power_supply");
        self.has_battery = false;
        self.battery = None;
        self.is_ac_online = true;

        if let Ok(entries) = fs::read_dir(ps_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("BAT") {
                    let path = entry.path();
                    let cap_str = fs::read_to_string(path.join("capacity")).unwrap_or_default();
                    let status = fs::read_to_string(path.join("status"))
                        .unwrap_or_else(|_| "Unknown".to_string())
                        .trim()
                        .to_string();
                    let percent = cap_str.trim().parse::<u8>().unwrap_or(0);

                    // Health calculation (energy_full / energy_full_design or charge_full / charge_full_design)
                    let full = fs::read_to_string(path.join("energy_full"))
                        .or_else(|_| fs::read_to_string(path.join("charge_full")))
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok());
                    let design = fs::read_to_string(path.join("energy_full_design"))
                        .or_else(|_| fs::read_to_string(path.join("charge_full_design")))
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok());

                    let health_percent = match (full, design) {
                        (Some(f), Some(d)) if d > 0 => Some(((f * 100) / d).min(100) as u8),
                        _ => None,
                    };

                    self.has_battery = true;
                    self.battery = Some(BatteryInfo {
                        name: fname,
                        percent,
                        status,
                        health_percent,
                    });
                } else if fname.starts_with("AC") || fname.starts_with("ADP") {
                    let online_str =
                        fs::read_to_string(entry.path().join("online")).unwrap_or_default();
                    self.is_ac_online = online_str.trim() == "1";
                }
            }
        }
    }

    fn load_power_profiles(&mut self) {
        let which = Command::new("which").arg("powerprofilesctl").output();
        if !which.map(|o| o.status.success()).unwrap_or(false) {
            self.has_powerprofiles = false;
            self.profiles.clear();
            return;
        }

        // Get current profile
        let get_out = Command::new("powerprofilesctl").arg("get").output();
        if let Ok(out) = get_out {
            if out.status.success() {
                self.current_profile = String::from_utf8_lossy(&out.stdout).trim().to_string();
                self.has_powerprofiles = true;
            } else {
                self.has_powerprofiles = false;
                return;
            }
        } else {
            self.has_powerprofiles = false;
            return;
        }

        // List available profiles
        let mut profiles = Vec::new();
        if let Ok(out) = Command::new("powerprofilesctl").arg("list").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('*')
                        || trimmed.starts_with('-')
                        || trimmed.ends_with(':')
                    {
                        let clean = trimmed
                            .trim_start_matches('*')
                            .trim_start_matches('-')
                            .trim_end_matches(':')
                            .trim();
                        if (clean == "performance" || clean == "balanced" || clean == "power-saver")
                            && !profiles.contains(&clean.to_string())
                        {
                            profiles.push(clean.to_string());
                        }
                    }
                }
            }
        }

        if profiles.is_empty() {
            profiles = vec![
                "performance".to_string(),
                "balanced".to_string(),
                "power-saver".to_string(),
            ];
        }

        self.selected_profile_idx = profiles
            .iter()
            .position(|p| p == &self.current_profile)
            .unwrap_or(0);
        self.profiles = profiles;
    }

    fn load_brightness(&mut self) {
        let which = Command::new("which").arg("brightnessctl").output();
        if !which.map(|o| o.status.success()).unwrap_or(false) {
            self.has_brightness = false;
            return;
        }

        // Verify backlight devices actually exist
        let bl_path = Path::new("/sys/class/backlight");
        let has_devices = fs::read_dir(bl_path)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);

        if !has_devices {
            self.has_brightness = false;
            return;
        }

        // Run brightnessctl info
        if let Ok(out) = Command::new("brightnessctl").args(["info"]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                // Search for pattern like "(75%)"
                for line in text.lines() {
                    if let Some(open) = line.find('(') {
                        if let Some(close) = line[open..].find("%)") {
                            let pct_str = &line[open + 1..open + close];
                            if let Ok(val) = pct_str.parse::<u8>() {
                                self.brightness_percent = val;
                                self.has_brightness = true;
                                return;
                            }
                        }
                    }
                }
            }
        }

        self.has_brightness = false;
    }

    pub fn next_profile(&mut self) {
        if !self.profiles.is_empty() {
            if self.selected_profile_idx + 1 < self.profiles.len() {
                self.selected_profile_idx += 1;
            } else {
                self.selected_profile_idx = 0;
            }
        }
    }

    pub fn prev_profile(&mut self) {
        if !self.profiles.is_empty() {
            if self.selected_profile_idx > 0 {
                self.selected_profile_idx -= 1;
            } else {
                self.selected_profile_idx = self.profiles.len() - 1;
            }
        }
    }

    pub fn set_profile_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        if !self.has_powerprofiles {
            return None;
        }
        let profile = self.profiles.get(self.selected_profile_idx)?;
        Some(("powerprofilesctl", vec!["set".to_string(), profile.clone()]))
    }

    pub fn brightness_up_cmd(&self) -> Option<(&'static str, Vec<&'static str>)> {
        if !self.has_brightness {
            return None;
        }
        Some(("brightnessctl", vec!["set", "10%+"]))
    }

    pub fn brightness_down_cmd(&self) -> Option<(&'static str, Vec<&'static str>)> {
        if !self.has_brightness {
            return None;
        }
        Some(("brightnessctl", vec!["set", "10%-"]))
    }
}
