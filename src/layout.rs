use crate::collectors;
use crate::i18n::I18n;
use crate::logo_data::{LOGO_LINES, LOGO_WIDTH};
use crate::report::SystemReport;

pub fn render(report: &SystemReport, i18n: &I18n) {
    let host =
        collectors::host::format_host(report.host.as_deref(), report.display_diagonal_inches, i18n);
    let os = report.os.clone();
    let kernel = report.kernel.clone();
    let de = report.de.clone().unwrap_or_else(|| "—".to_string());
    let shell = report.shell.clone();
    let packages = collectors::packages::format_packages(&report.packages);
    let uptime = if report.uptime_seconds > 0 {
        i18n.format_uptime(report.uptime_seconds)
    } else {
        "—".to_string()
    };
    let cpu = collectors::cpu::format_cpu(&report.cpu, i18n);
    let gpu = collectors::gpu::format_gpu(&report.gpu, i18n);
    let memory = collectors::memory::format_memory(&report.memory, i18n);
    let disk = collectors::disk::format_disk(&report.disk, i18n);
    let display = report
        .display_resolution
        .clone()
        .unwrap_or_else(|| "—".to_string());

    let items = vec![
        (i18n.label_os(), os),
        (i18n.label_kernel(), kernel),
        (i18n.label_de(), de),
        (i18n.label_shell(), shell),
        (i18n.label_packages(), packages),
        (i18n.label_uptime(), uptime),
        (i18n.label_cpu(), cpu),
        (i18n.label_gpu(), gpu),
        (i18n.label_memory(), memory),
        (i18n.label_disk(), disk),
        (i18n.label_display(), display),
    ];

    let max_key_width = items
        .iter()
        .map(|(k, _)| unicode_width(k))
        .max()
        .unwrap_or(12);

    let key_pad = max_key_width + 2;

    let mut right_lines: Vec<String> = Vec::new();

    // Line 0: Header (Bold White)
    right_lines.push(format!("\x1b[1;97m{host}\x1b[0m"));

    // Line 1: Separator line
    let sep_len = 54;
    right_lines.push(format!("\x1b[90m{}\x1b[0m", "─".repeat(sep_len)));

    // Lines 2+: Key-value pairs
    for (k, v) in items {
        let k_len = unicode_width(k);
        let spaces = " ".repeat(key_pad.saturating_sub(k_len));
        right_lines.push(format!("\x1b[96m{k}\x1b[0m{spaces}\x1b[97m{v}\x1b[0m"));
    }

    // Line after data: Blank line
    right_lines.push(String::new());

    // Color palette bar
    let palette =
        "   \x1b[41m   \x1b[42m   \x1b[43m   \x1b[44m   \x1b[45m   \x1b[46m   \x1b[47m   \x1b[0m";
    right_lines.push(palette.to_string());

    // Combine columns
    let total_rows = LOGO_LINES.len().max(right_lines.len());
    let empty_logo_padding = " ".repeat(LOGO_WIDTH);
    let column_gap = "   ";

    for row in 0..total_rows {
        let left = if row < LOGO_LINES.len() {
            LOGO_LINES[row]
        } else {
            &empty_logo_padding
        };

        let right = if row < right_lines.len() {
            &right_lines[row]
        } else {
            ""
        };

        println!("{left}{column_gap}{right}");
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().count()
}
