use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SystemReport {
    pub host: Option<String>,
    pub display_diagonal_inches: Option<u32>,
    pub os: String,
    pub kernel: String,
    pub de: Option<String>,
    pub shell: String,
    pub packages: PackageReport,
    pub uptime_seconds: u64,
    pub cpu: CpuReport,
    pub gpu: GpuReport,
    pub memory: MemoryReport,
    pub disk: DiskReport,
    pub display_resolution: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PackageReport {
    pub pacman: Option<usize>,
    pub dpkg: Option<usize>,
    pub rpm: Option<usize>,
    pub flatpak: Option<usize>,
    pub snap: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuReport {
    pub model: String,
    pub cores: usize,
    pub freq_ghz: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GpuReport {
    pub vendor: String,
    pub model: Option<String>,
    pub vram_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryReport {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskReport {
    pub device: String,
    pub total_bytes: u64,
    pub is_ssd: bool,
    pub is_nvme: bool,
}

impl SystemReport {
    pub fn collect() -> Self {
        let (host, display_diagonal_inches) = crate::collectors::host::get_host_info();
        let os = crate::collectors::os::get_os();
        let kernel = crate::collectors::kernel::get_kernel();
        let de = crate::collectors::de::get_de_info();
        let shell = crate::collectors::shell::get_shell();
        let packages = crate::collectors::packages::get_packages_report();
        let uptime_seconds = crate::collectors::uptime::get_uptime_secs();
        let cpu = crate::collectors::cpu::get_cpu_report();
        let gpu = crate::collectors::gpu::get_gpu_report();
        let memory = crate::collectors::memory::get_memory_report();
        let disk = crate::collectors::disk::get_disk_report();
        let display_resolution = crate::collectors::display::get_display_resolution();

        Self {
            host,
            display_diagonal_inches,
            os,
            kernel,
            de,
            shell,
            packages,
            uptime_seconds,
            cpu,
            gpu,
            memory,
            disk,
            display_resolution,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn to_short(&self) -> String {
        let cpu_str = if self.cpu.cores > 0 {
            format!("{} ({})", self.cpu.model, self.cpu.cores)
        } else {
            self.cpu.model.clone()
        };

        let gpu_str = match (&self.gpu.model, self.gpu.vram_mb) {
            (Some(m), Some(v)) => {
                let gb = (v as f64 / 1024.0).round() as u64;
                format!("{m} {gb}GB")
            }
            (Some(m), None) => m.clone(),
            (None, _) => self.gpu.vendor.clone(),
        };

        let ram_used_gb = self.memory.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let ram_total_gb = self.memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let ram_str = format!("RAM: {ram_used_gb:.1}/{ram_total_gb:.1} GB");

        let disk_gb = (self.disk.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).round() as u64;
        let disk_type = if self.disk.is_nvme {
            "NVMe"
        } else if self.disk.is_ssd {
            "SSD"
        } else {
            "HDD"
        };
        let disk_str = format!("{disk_type} {disk_gb} GB");

        let hours = self.uptime_seconds / 3600;
        let mins = (self.uptime_seconds % 3600) / 60;
        let uptime_str = if hours > 24 {
            let days = hours / 24;
            format!("Up: {}d {}h", days, hours % 24)
        } else {
            format!("Up: {hours}h {mins}m")
        };

        format!(
            "{} | {} | {} | {} | {} | {} | {}",
            self.os, self.kernel, cpu_str, gpu_str, ram_str, disk_str, uptime_str
        )
    }
}
