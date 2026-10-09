use std::fs;

pub fn get_uptime_secs() -> u64 {
    if let Ok(content) = fs::read_to_string("/proc/uptime") {
        if let Some(first) = content.split_whitespace().next() {
            if let Ok(secs_f) = first.parse::<f64>() {
                return secs_f as u64;
            }
        }
    }
    0
}
