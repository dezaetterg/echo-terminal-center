use crate::i18n::{I18n, Language};
use crate::report::CpuReport;
use std::collections::HashSet;
use std::fs;

pub fn get_cpu_report() -> CpuReport {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    let mut raw_model = String::new();
    let mut cpu_cores: Option<usize> = None;
    let mut core_ids = HashSet::new();
    let mut processor_count = 0;
    let mut mhz_fallback: Option<f64> = None;

    for line in content.lines() {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim();
            let val = v.trim();

            match key {
                "model name" => {
                    if raw_model.is_empty() && !val.is_empty() {
                        raw_model = val.to_string();
                    }
                }
                "cpu cores" => {
                    if cpu_cores.is_none() {
                        if let Ok(num) = val.parse::<usize>() {
                            cpu_cores = Some(num);
                        }
                    }
                }
                "core id" => {
                    if let Ok(id) = val.parse::<usize>() {
                        core_ids.insert(id);
                    }
                }
                "processor" => {
                    processor_count += 1;
                }
                "cpu MHz" if mhz_fallback.is_none() => {
                    if let Ok(m) = val.parse::<f64>() {
                        mhz_fallback = Some(m);
                    }
                }
                _ => {}
            }
        }
    }

    if raw_model.is_empty() {
        return CpuReport {
            model: "—".to_string(),
            cores: 0,
            freq_ghz: None,
        };
    }

    let cores = cpu_cores.unwrap_or(if !core_ids.is_empty() {
        core_ids.len()
    } else if processor_count > 0 {
        processor_count
    } else {
        1
    });

    let freq_ghz = get_cpu_freq_ghz(&raw_model, mhz_fallback);
    let clean_model = sanitize_cpu_model(&raw_model);

    CpuReport {
        model: clean_model,
        cores,
        freq_ghz,
    }
}

pub fn format_cpu(report: &CpuReport, i18n: &I18n) -> String {
    if report.model == "—" && report.cores == 0 {
        return "—".to_string();
    }

    let cores_str = format!("{}{}", report.cores, i18n.cores_suffix());
    let unit_ghz = i18n.unit_ghz();

    if let Some(ghz) = report.freq_ghz {
        let freq_val = if ghz.fract() == 0.0 {
            format!("{ghz:.1}")
        } else {
            let s = format!("{ghz:.2}");
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        };

        let freq_formatted = match i18n.lang {
            Language::Russian => format!("{} {unit_ghz}", freq_val.replace('.', ",")),
            Language::English => format!("{freq_val} {unit_ghz}"),
        };

        format!("{freq_formatted} {cores_str} {}", report.model)
    } else {
        format!("{cores_str} {}", report.model)
    }
}

fn get_cpu_freq_ghz(raw_model: &str, mhz_fallback: Option<f64>) -> Option<f64> {
    if let Ok(val) = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        if let Ok(khz) = val.trim().parse::<f64>() {
            if khz > 100_000.0 {
                return Some(khz / 1_000_000.0);
            }
        }
    }

    if let Some(pos) = raw_model.find('@') {
        let after = &raw_model[pos + 1..].trim();
        if let Some(num_part) = after.strip_suffix("GHz") {
            if let Ok(val) = num_part.trim().parse::<f64>() {
                return Some(val);
            }
        }
    }

    if let Some(mhz) = mhz_fallback {
        if mhz > 100.0 {
            return Some(mhz / 1000.0);
        }
    }

    None
}

fn sanitize_cpu_model(raw: &str) -> String {
    let mut s = raw.to_string();

    if let Some(pos) = s.find('@') {
        s = s[..pos].trim().to_string();
    }

    s = s
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("CPU", "")
        .replace("Processor", "")
        .replace("Dual-Core", "")
        .replace("Quad-Core", "")
        .replace("Six-Core", "")
        .replace("Eight-Core", "");

    let words: Vec<&str> = s.split_whitespace().collect();
    let filtered: Vec<&str> = words
        .into_iter()
        .filter(|w| !w.to_lowercase().ends_with("-core"))
        .collect();
    filtered.join(" ")
}
