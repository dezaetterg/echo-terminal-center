use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub state: String,
    pub ips: Vec<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkState {
    pub interfaces: Vec<NetworkInterface>,
    pub default_gateway: Option<String>,
    pub dns_servers: Vec<String>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            interfaces: Vec::new(),
            default_gateway: None,
            dns_servers: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        self.interfaces = Self::collect_interfaces();
        self.default_gateway = Self::collect_gateway();
        self.dns_servers = Self::collect_dns();
    }

    fn collect_interfaces() -> Vec<NetworkInterface> {
        let mut ifaces = Vec::new();

        if let Ok(out) = Command::new("ip").args(["-br", "addr"]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[0].to_string();
                        let state = parts[1].to_string();
                        let ips: Vec<String> = parts[2..].iter().map(|s| s.to_string()).collect();

                        let mac_path = format!("/sys/class/net/{name}/address");
                        let mac = fs::read_to_string(mac_path)
                            .ok()
                            .map(|s| s.trim().to_string());

                        ifaces.push(NetworkInterface {
                            name,
                            state,
                            ips,
                            mac,
                        });
                    }
                }
                return ifaces;
            }
        }

        // Fallback: sysfs
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let operstate = fs::read_to_string(entry.path().join("operstate"))
                    .unwrap_or_else(|_| "UNKNOWN".to_string())
                    .trim()
                    .to_uppercase();
                let mac = fs::read_to_string(entry.path().join("address"))
                    .ok()
                    .map(|s| s.trim().to_string());

                ifaces.push(NetworkInterface {
                    name,
                    state: operstate,
                    ips: Vec::new(),
                    mac,
                });
            }
        }

        ifaces
    }

    fn collect_gateway() -> Option<String> {
        if let Ok(out) = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let first_line = text.lines().next()?;
                // "default via 192.168.1.1 dev wlx502b73d0df54 proto dhcp src 192.168.1.226 metric 600"
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 3 && parts[1] == "via" {
                    let gw = parts[2];
                    let dev = parts.get(4).unwrap_or(&"");
                    return Some(format!("{gw} (dev {dev})"));
                }
            }
        }
        None
    }

    fn collect_dns() -> Vec<String> {
        let mut servers = Vec::new();
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("nameserver ") {
                    if let Some(ip) = trimmed.split_whitespace().nth(1) {
                        servers.push(ip.to_string());
                    }
                }
            }
        }
        servers
    }
}
