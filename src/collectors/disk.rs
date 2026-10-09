use crate::i18n::I18n;
use crate::report::DiskReport;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_disk_report() -> DiskReport {
    let root_dev = get_root_device_name().unwrap_or_else(|| "unknown".to_string());
    let base_dev = get_base_disk_name(&root_dev);

    let size_path = format!("/sys/block/{base_dev}/size");
    let sectors: u64 = fs::read_to_string(&size_path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    let total_bytes = sectors * 512;

    let rot_path = format!("/sys/block/{base_dev}/queue/rotational");
    let is_ssd = fs::read_to_string(&rot_path)
        .map(|s| s.trim() == "0")
        .unwrap_or(true);

    let is_nvme = base_dev.starts_with("nvme");

    DiskReport {
        device: base_dev,
        total_bytes,
        is_ssd,
        is_nvme,
    }
}

pub fn format_disk(report: &DiskReport, i18n: &I18n) -> String {
    if report.total_bytes == 0 {
        return "—".to_string();
    }

    let size_gb = (report.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u64;

    let type_str = if report.is_ssd {
        if report.is_nvme {
            "SSD (NVMe)"
        } else {
            "SSD"
        }
    } else {
        "HDD"
    };

    let unit = i18n.unit_gb();
    format!("{type_str} {size_gb} {unit}")
}

fn get_root_device_name() -> Option<String> {
    let mounts = fs::read_to_string("/proc/mounts").ok()?;
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "/" {
            let dev_path = parts[0];
            let canonical = fs::canonicalize(dev_path).unwrap_or_else(|_| PathBuf::from(dev_path));
            if let Some(file_name) = canonical.file_name() {
                return Some(file_name.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn get_base_disk_name(dev: &str) -> String {
    if Path::new(&format!("/sys/block/{dev}")).exists() {
        return dev.to_string();
    }

    if dev.starts_with("nvme") {
        if let Some(p_idx) = dev.rfind('p') {
            let prefix = &dev[..p_idx];
            if Path::new(&format!("/sys/block/{prefix}")).exists() {
                return prefix.to_string();
            }
        }
    }

    if dev.starts_with("mmcblk") {
        if let Some(p_idx) = dev.rfind('p') {
            let prefix = &dev[..p_idx];
            if Path::new(&format!("/sys/block/{prefix}")).exists() {
                return prefix.to_string();
            }
        }
    }

    let stripped: String = dev.chars().take_while(|c| !c.is_ascii_digit()).collect();
    if Path::new(&format!("/sys/block/{stripped}")).exists() {
        return stripped;
    }

    dev.to_string()
}
