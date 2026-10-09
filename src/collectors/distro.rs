use std::fs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistroFamily {
    Arch,
    Debian,
    Fedora,
    Other,
}

pub struct DistroInfo {
    pub pretty_name: String,
    pub family: DistroFamily,
}

pub fn get_distro() -> DistroInfo {
    let content = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
        .unwrap_or_default();

    let mut pretty_name = None;
    let mut id = String::new();
    let mut id_like = String::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
            pretty_name = Some(val.trim_matches('"').trim_matches('\'').to_string());
        } else if let Some(val) = line.strip_prefix("ID=") {
            id = val.trim_matches('"').trim_matches('\'').to_lowercase();
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            id_like = val.trim_matches('"').trim_matches('\'').to_lowercase();
        }
    }

    let family = if id == "arch" || id_like.split_whitespace().any(|s| s == "arch") {
        DistroFamily::Arch
    } else if id == "debian"
        || id == "ubuntu"
        || id == "pika"
        || id_like
            .split_whitespace()
            .any(|s| s == "debian" || s == "ubuntu")
    {
        DistroFamily::Debian
    } else if id == "fedora"
        || id == "rhel"
        || id_like
            .split_whitespace()
            .any(|s| s == "fedora" || s == "rhel")
    {
        DistroFamily::Fedora
    } else {
        DistroFamily::Other
    };

    DistroInfo {
        pretty_name: pretty_name.unwrap_or_else(|| "—".to_string()),
        family,
    }
}
