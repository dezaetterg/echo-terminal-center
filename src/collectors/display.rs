use std::fs;
use std::path::Path;

pub fn get_display_resolution() -> Option<String> {
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
                        let modes_file = path.join("modes");
                        if let Ok(modes) = fs::read_to_string(&modes_file) {
                            if let Some(first_mode) = modes.lines().next() {
                                let trimmed = first_mode.trim();
                                if !trimmed.is_empty() {
                                    return Some(trimmed.replace('x', " × "));
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
