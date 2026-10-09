use std::process::Command;

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub volume_percent: u8,
    pub is_muted: bool,
}

#[derive(Debug, Clone)]
pub struct AudioState {
    pub sinks: Vec<AudioDevice>,
    pub sources: Vec<AudioDevice>,
    pub active_tab: usize, // 0 = Sinks (Output), 1 = Sources (Input)
    pub selected_sink_idx: usize,
    pub selected_source_idx: usize,
    pub audio_available: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            sinks: Vec::new(),
            sources: Vec::new(),
            active_tab: 0,
            selected_sink_idx: 0,
            selected_source_idx: 0,
            audio_available: false,
        }
    }

    pub fn load(&mut self) {
        let which = Command::new("which").arg("pactl").output();
        if !which.map(|o| o.status.success()).unwrap_or(false) {
            self.audio_available = false;
            return;
        }

        self.audio_available = true;

        let def_sink = Command::new("pactl")
            .arg("get-default-sink")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let def_source = Command::new("pactl")
            .arg("get-default-source")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let mut sinks = Vec::new();
        if let Ok(out) = Command::new("pactl")
            .args(["list", "sinks", "short"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 2 {
                        let id = parts[0].trim().to_string();
                        let name = parts[1].trim().to_string();
                        let is_default = name == def_sink;
                        let (volume, muted) = Self::get_device_volume_and_mute(&name, true);

                        sinks.push(AudioDevice {
                            id,
                            name,
                            is_default,
                            volume_percent: volume,
                            is_muted: muted,
                        });
                    }
                }
            }
        }

        // Filter out monitor streams (e.g. *.monitor)
        let mut sources = Vec::new();
        if let Ok(out) = Command::new("pactl")
            .args(["list", "sources", "short"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 2 {
                        let id = parts[0].trim().to_string();
                        let name = parts[1].trim().to_string();
                        if name.ends_with(".monitor") || name.contains(".monitor") {
                            continue;
                        }
                        let is_default = name == def_source;
                        let (volume, muted) = Self::get_device_volume_and_mute(&name, false);

                        sources.push(AudioDevice {
                            id,
                            name,
                            is_default,
                            volume_percent: volume,
                            is_muted: muted,
                        });
                    }
                }
            }
        }

        self.sinks = sinks;
        self.sources = sources;

        if self.selected_sink_idx >= self.sinks.len() && !self.sinks.is_empty() {
            self.selected_sink_idx = self.sinks.len() - 1;
        }
        if self.selected_source_idx >= self.sources.len() && !self.sources.is_empty() {
            self.selected_source_idx = self.sources.len() - 1;
        }
    }

    fn get_device_volume_and_mute(name: &str, is_sink: bool) -> (u8, bool) {
        let (cmd_vol, cmd_mute) = if is_sink {
            ("get-sink-volume", "get-sink-mute")
        } else {
            ("get-source-volume", "get-source-mute")
        };

        let mut volume = 0u8;
        if let Ok(out) = Command::new("pactl").args([cmd_vol, name]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                volume = Self::parse_volume_percentage(&text);
            }
        }

        let mut muted = false;
        if let Ok(out) = Command::new("pactl").args([cmd_mute, name]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                muted = text.to_lowercase().contains("mute: yes");
            }
        }

        (volume, muted)
    }

    pub fn parse_volume_percentage(output: &str) -> u8 {
        // Look for pattern like `/  99% /` or `99%`
        let mut percentages = Vec::new();
        for part in output.split('/') {
            let trimmed = part.trim();
            if let Some(pct_idx) = trimmed.find('%') {
                let num_str = trimmed[..pct_idx].trim();
                // Take only numeric suffix if whitespace or chars precede
                let digits: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
                if let Ok(val) = digits.parse::<u16>() {
                    percentages.push(val.min(100) as u8);
                }
            }
        }

        if !percentages.is_empty() {
            let sum: usize = percentages.iter().map(|&x| x as usize).sum();
            (sum / percentages.len()) as u8
        } else {
            0
        }
    }

    pub fn switch_tab(&mut self) {
        self.active_tab = if self.active_tab == 0 { 1 } else { 0 };
    }

    pub fn next(&mut self) {
        if self.active_tab == 0 {
            if !self.sinks.is_empty() {
                if self.selected_sink_idx + 1 < self.sinks.len() {
                    self.selected_sink_idx += 1;
                } else {
                    self.selected_sink_idx = 0;
                }
            }
        } else if !self.sources.is_empty() {
            if self.selected_source_idx + 1 < self.sources.len() {
                self.selected_source_idx += 1;
            } else {
                self.selected_source_idx = 0;
            }
        }
    }

    pub fn prev(&mut self) {
        if self.active_tab == 0 {
            if !self.sinks.is_empty() {
                if self.selected_sink_idx > 0 {
                    self.selected_sink_idx -= 1;
                } else {
                    self.selected_sink_idx = self.sinks.len() - 1;
                }
            }
        } else if !self.sources.is_empty() {
            if self.selected_source_idx > 0 {
                self.selected_source_idx -= 1;
            } else {
                self.selected_source_idx = self.sources.len() - 1;
            }
        }
    }

    pub fn selected_device(&self) -> Option<&AudioDevice> {
        if self.active_tab == 0 {
            self.sinks.get(self.selected_sink_idx)
        } else {
            self.sources.get(self.selected_source_idx)
        }
    }

    pub fn set_default_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        let dev = self.selected_device()?;
        if self.active_tab == 0 {
            Some((
                "pactl",
                vec!["set-default-sink".to_string(), dev.name.clone()],
            ))
        } else {
            Some((
                "pactl",
                vec!["set-default-source".to_string(), dev.name.clone()],
            ))
        }
    }

    pub fn toggle_mute_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        let dev = self.selected_device()?;
        if self.active_tab == 0 {
            Some((
                "pactl",
                vec![
                    "set-sink-mute".to_string(),
                    dev.name.clone(),
                    "toggle".to_string(),
                ],
            ))
        } else {
            Some((
                "pactl",
                vec![
                    "set-source-mute".to_string(),
                    dev.name.clone(),
                    "toggle".to_string(),
                ],
            ))
        }
    }

    pub fn volume_up_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        let dev = self.selected_device()?;
        if self.active_tab == 0 {
            Some((
                "pactl",
                vec![
                    "set-sink-volume".to_string(),
                    dev.name.clone(),
                    "+5%".to_string(),
                ],
            ))
        } else {
            Some((
                "pactl",
                vec![
                    "set-source-volume".to_string(),
                    dev.name.clone(),
                    "+5%".to_string(),
                ],
            ))
        }
    }

    pub fn volume_down_cmd(&self) -> Option<(&'static str, Vec<String>)> {
        let dev = self.selected_device()?;
        if self.active_tab == 0 {
            Some((
                "pactl",
                vec![
                    "set-sink-volume".to_string(),
                    dev.name.clone(),
                    "-5%".to_string(),
                ],
            ))
        } else {
            Some((
                "pactl",
                vec![
                    "set-source-volume".to_string(),
                    dev.name.clone(),
                    "-5%".to_string(),
                ],
            ))
        }
    }
}
