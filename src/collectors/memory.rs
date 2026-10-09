use crate::i18n::I18n;
use crate::report::MemoryReport;
use std::fs;

pub fn get_memory_report() -> MemoryReport {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();

    let mut total_kb: Option<u64> = None;
    let mut avail_kb: Option<u64> = None;
    let mut free_kb: Option<u64> = None;
    let mut buffers_kb: Option<u64> = None;
    let mut cached_kb: Option<u64> = None;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
            if let Ok(val) = val_str.parse::<u64>() {
                match key {
                    "MemTotal:" => total_kb = Some(val),
                    "MemAvailable:" => avail_kb = Some(val),
                    "MemFree:" => free_kb = Some(val),
                    "Buffers:" => buffers_kb = Some(val),
                    "Cached:" => cached_kb = Some(val),
                    _ => {}
                }
            }
        }
    }

    let total = total_kb.unwrap_or(0);
    let avail = avail_kb.unwrap_or_else(|| {
        let free = free_kb.unwrap_or(0);
        let buffers = buffers_kb.unwrap_or(0);
        let cached = cached_kb.unwrap_or(0);
        free + buffers + cached
    });

    let used = total.saturating_sub(avail);
    let percent = if total > 0 {
        ((used as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    MemoryReport {
        total_bytes: total * 1024,
        used_bytes: used * 1024,
        available_bytes: avail * 1024,
        percent,
    }
}

pub fn format_memory(report: &MemoryReport, i18n: &I18n) -> String {
    if report.total_bytes == 0 {
        return "—".to_string();
    }

    let total_gb = report.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = report.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let total_slots = 10;
    let filled = ((report.percent / 100.0) * total_slots as f64).round() as usize;
    let filled = filled.min(total_slots);
    let empty = total_slots - filled;

    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
    let unit = i18n.unit_gb();

    format!(
        "{used_gb:.1} {unit} / {total_gb:.1} {unit} ({:.0}%) {bar}",
        report.percent
    )
}
