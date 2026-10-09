use std::process::Command;

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub mac: String,
    pub name: String,
    pub connected: bool,
    pub battery_percent: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct BluetoothState {
    pub adapter_powered: bool,
    pub adapter_name: String,
    pub devices: Vec<BluetoothDevice>,
    pub selected_index: usize,
    pub bluetooth_available: bool,
}

impl Default for BluetoothState {
    fn default() -> Self {
        Self::new()
    }
}

impl BluetoothState {
    pub fn new() -> Self {
        Self {
            adapter_powered: false,
            adapter_name: String::new(),
            devices: Vec::new(),
            selected_index: 0,
            bluetooth_available: false,
        }
    }

    pub fn load(&mut self) {
        let which = Command::new("which").arg("bluetoothctl").output();
        if !which.map(|o| o.status.success()).unwrap_or(false) {
            self.bluetooth_available = false;
            return;
        }

        let show_out = Command::new("bluetoothctl")
            .args(["--timeout", "5", "show"])
            .output();

        match show_out {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                if text.trim().is_empty() {
                    self.bluetooth_available = false;
                    return;
                }
                self.bluetooth_available = true;
                self.adapter_powered = text.lines().any(|l| {
                    let t = l.trim();
                    t == "Powered: yes" || t == "PowerState: on"
                });

                self.adapter_name = text
                    .lines()
                    .find(|l| l.trim().starts_with("Alias:") || l.trim().starts_with("Name:"))
                    .map(|l| {
                        l.trim()
                            .split_once(':')
                            .map(|(_, v)| v.trim().to_string())
                            .unwrap_or_default()
                    })
                    .unwrap_or_else(|| "Controller".to_string());
            }
            _ => {
                self.bluetooth_available = false;
                return;
            }
        }

        let mut connected_macs = Vec::new();
        if let Ok(out) = Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "Device" {
                        connected_macs.push(parts[1].to_uppercase());
                    }
                }
            }
        }

        let mut dev_list = Vec::new();
        if let Ok(out) = Command::new("bluetoothctl").args(["devices"]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[0] == "Device" {
                        let mac = parts[1].to_uppercase();
                        let name = parts[2..].join(" ");
                        let connected = connected_macs.contains(&mac);
                        let battery_percent = Self::get_device_battery(&mac);

                        dev_list.push(BluetoothDevice {
                            mac,
                            name,
                            connected,
                            battery_percent,
                        });
                    }
                }
            }
        }

        self.devices = dev_list;
        if self.selected_index >= self.devices.len() && !self.devices.is_empty() {
            self.selected_index = self.devices.len() - 1;
        }
    }

    fn get_device_battery(mac: &str) -> Option<u8> {
        let out = Command::new("bluetoothctl")
            .args(["info", mac])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(pos) = trimmed.find("Battery Percentage:") {
                let after = trimmed[pos + 19..].trim();
                // Format can be: "0x64 (100)" or "100"
                if let Some(open) = after.find('(') {
                    if let Some(close) = after.find(')') {
                        if let Ok(val) = after[open + 1..close].trim().parse::<u8>() {
                            return Some(val);
                        }
                    }
                }
                let first_word = after.split_whitespace().next().unwrap_or("");
                if let Ok(val) = first_word.parse::<u8>() {
                    return Some(val);
                }
            }
        }
        None
    }

    pub fn toggle_power_cmd(&self) -> (&'static str, Vec<&'static str>) {
        if self.adapter_powered {
            ("bluetoothctl", vec!["power", "off"])
        } else {
            ("bluetoothctl", vec!["power", "on"])
        }
    }

    pub fn connect_toggle_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        let dev = self.devices.get(self.selected_index)?;
        if dev.connected {
            Some((
                "bluetoothctl",
                vec!["disconnect".to_string(), dev.mac.clone()],
            ))
        } else {
            Some(("bluetoothctl", vec!["connect".to_string(), dev.mac.clone()]))
        }
    }

    pub fn next(&mut self) {
        if !self.devices.is_empty() {
            if self.selected_index + 1 < self.devices.len() {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn prev(&mut self) {
        if !self.devices.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.devices.len() - 1;
            }
        }
    }

    pub fn selected_device(&self) -> Option<&BluetoothDevice> {
        self.devices.get(self.selected_index)
    }
}
