use crate::i18n::I18n;
use crate::report::GpuReport;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}

pub fn get_gpu_report() -> GpuReport {
    let pci_dir = Path::new("/sys/bus/pci/devices");
    if !pci_dir.is_dir() {
        return GpuReport {
            vendor: "—".to_string(),
            model: None,
            vram_mb: None,
        };
    }

    let mut detected_vendors = Vec::new();

    if let Ok(entries) = fs::read_dir(pci_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let class_file = path.join("class");
            let vendor_file = path.join("vendor");

            if let (Ok(class_str), Ok(vendor_str)) = (
                fs::read_to_string(&class_file),
                fs::read_to_string(&vendor_file),
            ) {
                let class = class_str.trim().to_lowercase();
                if class.starts_with("0x0300") || class.starts_with("0x0302") {
                    let vendor = vendor_str.trim().to_lowercase();
                    if vendor == "0x10de" {
                        detected_vendors.push(GpuVendor::Nvidia);
                    } else if vendor == "0x1002" {
                        detected_vendors.push(GpuVendor::Amd);
                    } else if vendor == "0x8086" {
                        detected_vendors.push(GpuVendor::Intel);
                    } else {
                        detected_vendors.push(GpuVendor::Other);
                    }
                }
            }
        }
    }

    if detected_vendors.is_empty() {
        return GpuReport {
            vendor: "—".to_string(),
            model: None,
            vram_mb: None,
        };
    }

    let primary_vendor = if detected_vendors.contains(&GpuVendor::Nvidia) {
        GpuVendor::Nvidia
    } else if detected_vendors.contains(&GpuVendor::Amd) {
        GpuVendor::Amd
    } else if detected_vendors.contains(&GpuVendor::Intel) {
        GpuVendor::Intel
    } else {
        GpuVendor::Other
    };

    match primary_vendor {
        GpuVendor::Nvidia => {
            let (model, vram_mb) = query_nvidia_smi();
            GpuReport {
                vendor: "NVIDIA".to_string(),
                model,
                vram_mb,
            }
        }
        GpuVendor::Amd => GpuReport {
            vendor: "AMD".to_string(),
            model: Some("AMD Radeon".to_string()),
            vram_mb: None,
        },
        GpuVendor::Intel => GpuReport {
            vendor: "Intel".to_string(),
            model: Some("Intel Graphics".to_string()),
            vram_mb: None,
        },
        GpuVendor::Other => GpuReport {
            vendor: "—".to_string(),
            model: None,
            vram_mb: None,
        },
    }
}

pub fn format_gpu(report: &GpuReport, i18n: &I18n) -> String {
    match (&report.model, report.vram_mb) {
        (Some(model), Some(mb)) => {
            let gb = (mb as f64 / 1024.0).round() as u64;
            let unit = i18n.unit_gb();
            format!("{model} {gb} {unit}")
        }
        (Some(model), None) => model.clone(),
        (None, _) => report.vendor.clone(),
    }
}

fn query_nvidia_smi() -> (Option<String>, Option<u64>) {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(first_line) = text.lines().next() {
                let parts: Vec<&str> = first_line.split(',').map(|s| s.trim()).collect();
                if !parts.is_empty() {
                    let name = parts[0].to_string();
                    let vram = parts
                        .get(1)
                        .and_then(|v| v.parse::<f64>().ok())
                        .map(|m| m as u64);
                    return (Some(name), vram);
                }
            }
        }
    }

    (Some("NVIDIA Graphics".to_string()), None)
}
