use crate::i18n::I18n;
use std::fs;
use std::path::Path;

pub fn get_host_info() -> (Option<String>, Option<u32>) {
    (get_dmi_model(), get_display_diagonal())
}

pub fn format_host(model: Option<&str>, diagonal: Option<u32>, i18n: &I18n) -> String {
    let prefix = i18n.host_prefix();
    let inch_suf = i18n.inch_suffix();

    match (model, diagonal) {
        (Some(m), Some(d)) => format!("{prefix} {m} ({d}{inch_suf})"),
        (Some(m), None) => format!("{prefix} {m}"),
        (None, Some(d)) => format!("{prefix} ({d}{inch_suf})"),
        (None, None) => "—".to_string(),
    }
}

fn get_dmi_model() -> Option<String> {
    let clean_val = |s: String| -> Option<String> {
        let trimmed = s.trim();
        if trimmed.is_empty()
            || trimmed == "To be filled by O.E.M."
            || trimmed == "System Product Name"
            || trimmed == "Default string"
            || trimmed == "None"
        {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    if let Ok(prod) = fs::read_to_string("/sys/class/dmi/id/product_name") {
        if let Some(val) = clean_val(prod) {
            return Some(val);
        }
    }

    if let Ok(board) = fs::read_to_string("/sys/class/dmi/id/board_name") {
        if let Some(val) = clean_val(board) {
            return Some(val);
        }
    }

    None
}

fn get_display_diagonal() -> Option<u32> {
    let drm_dir = Path::new("/sys/class/drm");
    if !drm_dir.is_dir() {
        return None;
    }

    if let Ok(entries) = fs::read_dir(drm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let status_file = path.join("status");
            if status_file.is_file() {
                if let Ok(status) = fs::read_to_string(&status_file) {
                    if status.trim() == "connected" {
                        let edid_file = path.join("edid");
                        if let Ok(data) = fs::read(&edid_file) {
                            if data.len() >= 128 {
                                let w_cm = data[21] as f64;
                                let h_cm = data[22] as f64;
                                if w_cm > 0.0 && h_cm > 0.0 {
                                    let diag_in = (w_cm * w_cm + h_cm * h_cm).sqrt() / 2.54;
                                    let rounded = diag_in.round() as u32;
                                    if rounded > 0 && rounded < 150 {
                                        return Some(rounded);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
