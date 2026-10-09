#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Russian,
    English,
}

pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn detect() -> Self {
        let env_lang = std::env::var("LC_ALL")
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default();

        let lang = if env_lang.starts_with("ru") {
            Language::Russian
        } else {
            Language::English
        };

        Self { lang }
    }

    pub fn app_title(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Echo Terminal Center",
            Language::English => "Echo Terminal Center",
        }
    }

    pub fn host_prefix(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Компьютер",
            Language::English => "Computer",
        }
    }

    pub fn inch_suffix(&self) -> &'static str {
        match self.lang {
            Language::Russian => "-дюймовый",
            Language::English => "-inch",
        }
    }

    pub fn label_os(&self) -> &'static str {
        match self.lang {
            Language::Russian => "ОС:",
            Language::English => "OS:",
        }
    }

    pub fn label_kernel(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Ядро:",
            Language::English => "Kernel:",
        }
    }

    pub fn label_de(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Окружение:",
            Language::English => "DE:",
        }
    }

    pub fn label_shell(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Оболочка:",
            Language::English => "Shell:",
        }
    }

    pub fn label_packages(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Пакеты:",
            Language::English => "Packages:",
        }
    }

    pub fn label_uptime(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Время работы:",
            Language::English => "Uptime:",
        }
    }

    pub fn label_cpu(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Процессор:",
            Language::English => "CPU:",
        }
    }

    pub fn label_gpu(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Графика:",
            Language::English => "GPU:",
        }
    }

    pub fn label_memory(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Память:",
            Language::English => "Memory:",
        }
    }

    pub fn label_disk(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Системный диск:",
            Language::English => "System Disk:",
        }
    }

    pub fn label_display(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Дисплей:",
            Language::English => "Display:",
        }
    }

    pub fn cores_suffix(&self) -> &'static str {
        match self.lang {
            Language::Russian => "-ядерный",
            Language::English => "-core",
        }
    }

    pub fn unit_gb(&self) -> &'static str {
        match self.lang {
            Language::Russian => "ГБ",
            Language::English => "GB",
        }
    }

    pub fn unit_ghz(&self) -> &'static str {
        match self.lang {
            Language::Russian => "ГГц",
            Language::English => "GHz",
        }
    }

    pub fn format_uptime(&self, total_seconds: u64) -> String {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let mins = (total_seconds % 3600) / 60;

        match self.lang {
            Language::Russian => {
                if days > 0 {
                    format!("{days} дн. {hours} ч. {mins} мин.")
                } else if hours > 0 {
                    format!("{hours} ч. {mins} мин.")
                } else {
                    format!("{mins} мин.")
                }
            }
            Language::English => {
                if days > 0 {
                    format!("{days} d {hours} h {mins} m")
                } else if hours > 0 {
                    format!("{hours} h {mins} m")
                } else {
                    format!("{mins} m")
                }
            }
        }
    }

    // TUI Menu items
    pub fn menu_system_info(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Системная информация",
            Language::English => "System Information",
        }
    }

    pub fn menu_package_updates(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Обновление пакетов",
            Language::English => "Package Updates",
        }
    }

    pub fn menu_cache_cleaning(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Очистка кэша",
            Language::English => "Cache Cleaning",
        }
    }

    pub fn menu_service_management(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Управление сервисами",
            Language::English => "Service Management",
        }
    }

    pub fn menu_disk_status(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Состояние дисков",
            Language::English => "Disk Status",
        }
    }

    pub fn menu_network_connections(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Сетевые подключения",
            Language::English => "Network Connections",
        }
    }

    pub fn menu_system_logs(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Логи системы",
            Language::English => "System Logs",
        }
    }

    pub fn menu_diagnostics(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Диагностика",
            Language::English => "Diagnostics",
        }
    }

    pub fn menu_bluetooth(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Bluetooth",
            Language::English => "Bluetooth",
        }
    }

    pub fn menu_audio(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Аудио",
            Language::English => "Audio",
        }
    }

    pub fn menu_apps(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Установка приложений",
            Language::English => "App Management",
        }
    }

    pub fn menu_power(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Питание",
            Language::English => "Power",
        }
    }

    // TUI Hints
    pub fn hint_main_menu(&self) -> &'static str {
        match self.lang {
            Language::Russian => "↑/↓: Навигация | Enter: Выбрать | q/Esc: Выход",
            Language::English => "↑/↓: Navigate | Enter: Select | q/Esc: Quit",
        }
    }

    #[allow(dead_code)]
    pub fn hint_screen(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню",
            Language::English => "Esc: Back to menu",
        }
    }

    pub fn hint_system_info(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню | r: Обновить данные",
            Language::English => "Esc: Back to menu | r: Refresh data",
        }
    }

    pub fn hint_logs(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад | ↑/↓: Скролл | Home/End: Начало/Конец | r: Обновить",
            Language::English => "Esc: Back | ↑/↓: Scroll | Home/End: Top/Bottom | r: Refresh",
        }
    }

    pub fn hint_network(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню | r: Обновить список",
            Language::English => "Esc: Back to menu | r: Refresh list",
        }
    }

    pub fn hint_disks(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню | r: Обновить информацию",
            Language::English => "Esc: Back to menu | r: Refresh information",
        }
    }

    pub fn hint_diagnostics(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню | r: Обновить сенсоры",
            Language::English => "Esc: Back to menu | r: Refresh sensors",
        }
    }

    pub fn hint_packages(&self) -> &'static str {
        match self.lang {
            Language::Russian => {
                "Esc: Назад | Enter/u: Обновить всё | ↑/↓: Скролл | r: Перепроверить"
            }
            Language::English => "Esc: Back | Enter/u: Upgrade all | ↑/↓: Scroll | r: Re-check",
        }
    }

    pub fn hint_cache(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад к меню | Enter/c: Очистить кэш | r: Пересчитать",
            Language::English => "Esc: Back to menu | Enter/c: Clean cache | r: Recalculate",
        }
    }

    pub fn hint_services(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад | ↑/↓: Выбор | r: Перезапустить | s: Остановить",
            Language::English => "Esc: Back | ↑/↓: Select | r: Restart | s: Stop",
        }
    }

    pub fn hint_bluetooth(&self) -> &'static str {
        match self.lang {
            Language::Russian => {
                "Esc: Назад | ↑/↓: Выбор | c/Enter: Подкл/Откл | p: Питание | r: Обновить"
            }
            Language::English => {
                "Esc: Back | ↑/↓: Select | c/Enter: Connect/Disconnect | p: Power | r: Refresh"
            }
        }
    }

    pub fn hint_audio(&self) -> &'static str {
        match self.lang {
            Language::Russian => "Esc: Назад | Tab: Выходы/Входы | ↑/↓: Выбор | Enter: По умолч. | +/-: Громкость | m: Mute | r: Обновить",
            Language::English => "Esc: Back | Tab: Sinks/Sources | ↑/↓: Select | Enter: Set Default | +/-: Volume | m: Mute | r: Refresh",
        }
    }

    pub fn hint_apps(&self) -> &'static str {
        match self.lang {
            Language::Russian => {
                "Esc: Назад | Tab: Режим | /: Поиск | ↑/↓: Выбор | Enter: Действие | r: Обновить"
            }
            Language::English => {
                "Esc: Back | Tab: Mode | /: Search | ↑/↓: Select | Enter: Action | r: Refresh"
            }
        }
    }

    pub fn hint_power(&self) -> &'static str {
        match self.lang {
            Language::Russian => {
                "Esc: Назад | ↑/↓: Выбор профиля | Enter: Применить | +/-: Яркость | r: Обновить"
            }
            Language::English => {
                "Esc: Back | ↑/↓: Select Profile | Enter: Apply | +/-: Brightness | r: Refresh"
            }
        }
    }

    pub fn hint_confirm(&self) -> &'static str {
        match self.lang {
            Language::Russian => "y: Подтвердить выполнение | n/Esc: Отмена",
            Language::English => "y: Confirm execution | n/Esc: Cancel",
        }
    }

    pub fn confirm_title(&self) -> &'static str {
        match self.lang {
            Language::Russian => " Подтверждение действия ",
            Language::English => " Action Confirmation ",
        }
    }

    #[allow(dead_code)]
    pub fn confirm_prompt(&self, cmd: &str) -> String {
        match self.lang {
            Language::Russian => format!("Будет выполнено: {cmd}\nПродолжить? [y/N]"),
            Language::English => format!("Will execute: {cmd}\nContinue? [y/N]"),
        }
    }

    pub fn terminal_too_small(&self, w: u16, h: u16) -> String {
        match self.lang {
            Language::Russian => format!(
                "Размер окна: {}×{}. Минимальный требуемый размер: 80×20. Увеличьте терминал.",
                w, h
            ),
            Language::English => format!(
                "Window size: {}x{}. Minimum required size: 80x20. Please enlarge terminal.",
                w, h
            ),
        }
    }
}
