use super::distro::{get_distro, DistroFamily};
use crate::report::PackageReport;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn get_packages_report() -> PackageReport {
    let distro = get_distro();
    let mut report = PackageReport::default();

    match distro.family {
        DistroFamily::Arch => {
            report.pacman = count_pacman();
        }
        DistroFamily::Debian => {
            report.dpkg = count_dpkg();
            report.snap = count_snaps();
        }
        DistroFamily::Fedora => {
            report.rpm = count_rpm();
        }
        DistroFamily::Other => {
            if let Some(c) = count_pacman() {
                report.pacman = Some(c);
            } else if let Some(c) = count_dpkg() {
                report.dpkg = Some(c);
            } else if let Some(c) = count_rpm() {
                report.rpm = Some(c);
            }
        }
    }

    report.flatpak = count_flatpaks();
    report
}

pub fn format_packages(report: &PackageReport) -> String {
    let mut parts = Vec::new();

    if let Some(c) = report.pacman {
        parts.push(format!("{c} (pacman)"));
    }
    if let Some(c) = report.dpkg {
        parts.push(format!("{c} (dpkg)"));
    }
    if let Some(c) = report.rpm {
        parts.push(format!("{c} (rpm)"));
    }
    if let Some(c) = report.flatpak {
        if c > 0 {
            parts.push(format!("{c} (flatpak)"));
        }
    }
    if let Some(c) = report.snap {
        if c > 0 {
            parts.push(format!("{c} (snap)"));
        }
    }

    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(", ")
    }
}

fn count_pacman() -> Option<usize> {
    let local_dir = Path::new("/var/lib/pacman/local");
    if local_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(local_dir) {
            let count = entries
                .filter_map(Result::ok)
                .filter(|e| {
                    if let Ok(ft) = e.file_type() {
                        if ft.is_dir() {
                            let name = e.file_name();
                            let s = name.to_string_lossy();
                            return s != "ALPM_DB_VERSION";
                        }
                    }
                    false
                })
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    if let Ok(output) = Command::new("pacman").arg("-Qq").output() {
        if output.status.success() {
            let count = output
                .stdout
                .split(|&b| b == b'\n')
                .filter(|line| !line.is_empty())
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

fn count_dpkg() -> Option<usize> {
    if let Ok(content) = fs::read_to_string("/var/lib/dpkg/status") {
        let count = content
            .lines()
            .filter(|l| l.starts_with("Status: install ok installed"))
            .count();
        if count > 0 {
            return Some(count);
        }
    }

    if let Ok(output) = Command::new("dpkg-query")
        .args(["-f", ".\\n", "-W"])
        .output()
    {
        if output.status.success() {
            let count = output
                .stdout
                .split(|&b| b == b'\n')
                .filter(|line| !line.is_empty())
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

fn count_rpm() -> Option<usize> {
    if let Ok(output) = Command::new("rpm")
        .args(["-qa", "--nodigest", "--nosignature"])
        .output()
    {
        if output.status.success() {
            let count = output
                .stdout
                .split(|&b| b == b'\n')
                .filter(|line| !line.is_empty())
                .count();
            if count > 0 {
                return Some(count);
            }
        }
    }

    None
}

fn count_snaps() -> Option<usize> {
    let snap_dir = Path::new("/var/lib/snapd/snaps");
    if snap_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(snap_dir) {
            let count = entries
                .filter_map(Result::ok)
                .filter(|e| {
                    let name = e.file_name();
                    name.to_string_lossy().ends_with(".snap")
                })
                .count();
            return Some(count);
        }
    }
    None
}

fn count_flatpaks() -> Option<usize> {
    let mut total = 0;

    let sys_app = Path::new("/var/lib/flatpak/app");
    if sys_app.is_dir() {
        if let Ok(entries) = fs::read_dir(sys_app) {
            total += entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                .count();
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_app = Path::new(&home).join(".local/share/flatpak/app");
        if user_app.is_dir() {
            if let Ok(entries) = fs::read_dir(user_app) {
                total += entries
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                    .count();
            }
        }
    }

    if total > 0 {
        Some(total)
    } else {
        None
    }
}
