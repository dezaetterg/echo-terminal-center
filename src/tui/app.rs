use crate::i18n::I18n;
use crate::report::SystemReport;
use crate::tui::screens::{
    apps::AppsState, audio::AudioState, bluetooth::BluetoothState, cache::CacheState,
    diagnostics::DiagnosticsState, disks::DisksState, logs::LogsState, network::NetworkState,
    packages::PackagesState, power::PowerState, services::ServicesState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    SystemInfo,
    PackageUpdates,
    CacheCleaning,
    ServiceManagement,
    DiskStatus,
    NetworkConnections,
    SystemLogs,
    Diagnostics,
    Bluetooth,
    Audio,
    AppManagement,
    PowerManagement,
}

impl Screen {
    pub fn title<'a>(&self, i18n: &'a I18n) -> &'a str {
        match self {
            Screen::MainMenu => i18n.app_title(),
            Screen::SystemInfo => i18n.menu_system_info(),
            Screen::PackageUpdates => i18n.menu_package_updates(),
            Screen::CacheCleaning => i18n.menu_cache_cleaning(),
            Screen::ServiceManagement => i18n.menu_service_management(),
            Screen::DiskStatus => i18n.menu_disk_status(),
            Screen::NetworkConnections => i18n.menu_network_connections(),
            Screen::SystemLogs => i18n.menu_system_logs(),
            Screen::Diagnostics => i18n.menu_diagnostics(),
            Screen::Bluetooth => i18n.menu_bluetooth(),
            Screen::Audio => i18n.menu_audio(),
            Screen::AppManagement => i18n.menu_apps(),
            Screen::PowerManagement => i18n.menu_power(),
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub selected_index: usize,
    pub running: bool,
    pub system_report: Option<SystemReport>,
    pub logs_state: LogsState,
    pub network_state: NetworkState,
    pub disks_state: DisksState,
    pub diagnostics_state: DiagnosticsState,
    pub packages_state: PackagesState,
    pub cache_state: CacheState,
    pub services_state: ServicesState,
    pub bluetooth_state: BluetoothState,
    pub audio_state: AudioState,
    pub apps_state: AppsState,
    pub power_state: PowerState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub const MENU_ITEM_COUNT: usize = 12;

    pub fn new() -> Self {
        Self {
            screen: Screen::MainMenu,
            selected_index: 0,
            running: true,
            system_report: None,
            logs_state: LogsState::new(),
            network_state: NetworkState::new(),
            disks_state: DisksState::new(),
            diagnostics_state: DiagnosticsState::new(),
            packages_state: PackagesState::new(),
            cache_state: CacheState::new(),
            services_state: ServicesState::new(),
            bluetooth_state: BluetoothState::new(),
            audio_state: AudioState::new(),
            apps_state: AppsState::new(),
            power_state: PowerState::new(),
        }
    }

    pub fn ensure_report(&mut self) {
        if self.system_report.is_none() {
            self.system_report = Some(SystemReport::collect());
        }
    }

    pub fn refresh_report(&mut self) {
        self.system_report = Some(SystemReport::collect());
    }

    pub fn refresh_current_screen(&mut self) {
        match self.screen {
            Screen::SystemInfo => self.refresh_report(),
            Screen::PackageUpdates => self.packages_state.load(),
            Screen::CacheCleaning => self.cache_state.load(),
            Screen::ServiceManagement => self.services_state.load(),
            Screen::DiskStatus => self.disks_state.load(),
            Screen::NetworkConnections => self.network_state.load(),
            Screen::SystemLogs => self.logs_state.load(),
            Screen::Diagnostics => self.diagnostics_state.load(),
            Screen::Bluetooth => self.bluetooth_state.load(),
            Screen::Audio => self.audio_state.load(),
            Screen::AppManagement => self.apps_state.load(),
            Screen::PowerManagement => self.power_state.load(),
            Screen::MainMenu => {}
        }
    }

    pub fn has_active_dialog(&self) -> bool {
        self.packages_state.confirm_upgrade
            || self.cache_state.confirm_clean
            || self.services_state.confirm_action.is_some()
            || self.apps_state.confirm_action.is_some()
    }

    pub fn close_dialog(&mut self) {
        self.packages_state.confirm_upgrade = false;
        self.cache_state.confirm_clean = false;
        self.services_state.confirm_action = None;
        self.apps_state.confirm_action = None;
    }

    pub fn active_dialog_command(&self) -> Option<String> {
        if self.packages_state.confirm_upgrade {
            let (cmd, args) = self.packages_state.upgrade_command();
            Some(format!("{cmd} {}", args.join(" ")))
        } else if self.cache_state.confirm_clean {
            let (cmd, args) = self.cache_state.clean_command();
            Some(format!("{cmd} {}", args.join(" ")))
        } else if let Some((ref srv, action)) = self.services_state.confirm_action {
            Some(format!("sudo systemctl {action} {srv}"))
        } else if let Some((ref cmd, ref args)) = self.apps_state.confirm_action {
            Some(format!("{cmd} {}", args.join(" ")))
        } else {
            None
        }
    }

    pub fn selected_screen(&self) -> Screen {
        match self.selected_index {
            0 => Screen::SystemInfo,
            1 => Screen::PackageUpdates,
            2 => Screen::CacheCleaning,
            3 => Screen::ServiceManagement,
            4 => Screen::DiskStatus,
            5 => Screen::NetworkConnections,
            6 => Screen::SystemLogs,
            7 => Screen::Diagnostics,
            8 => Screen::Bluetooth,
            9 => Screen::Audio,
            10 => Screen::AppManagement,
            11 => Screen::PowerManagement,
            _ => Screen::SystemInfo,
        }
    }

    pub fn next_item(&mut self) {
        if self.screen == Screen::MainMenu {
            if self.selected_index + 1 < Self::MENU_ITEM_COUNT {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn prev_item(&mut self) {
        if self.screen == Screen::MainMenu {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = Self::MENU_ITEM_COUNT - 1;
            }
        }
    }

    pub fn select(&mut self) {
        if self.screen == Screen::MainMenu {
            self.screen = self.selected_screen();
            match self.screen {
                Screen::SystemInfo => self.ensure_report(),
                Screen::PackageUpdates => self.packages_state.load(),
                Screen::CacheCleaning => self.cache_state.load(),
                Screen::ServiceManagement => self.services_state.load(),
                Screen::DiskStatus => self.disks_state.load(),
                Screen::NetworkConnections => self.network_state.load(),
                Screen::SystemLogs => self.logs_state.load(),
                Screen::Diagnostics => self.diagnostics_state.load(),
                Screen::Bluetooth => self.bluetooth_state.load(),
                Screen::Audio => self.audio_state.load(),
                Screen::AppManagement => self.apps_state.load(),
                Screen::PowerManagement => self.power_state.load(),
                Screen::MainMenu => {}
            }
        }
    }

    pub fn go_back(&mut self) {
        if self.has_active_dialog() {
            self.close_dialog();
            return;
        }

        match self.screen {
            Screen::MainMenu => {
                self.running = false;
            }
            _ => {
                self.screen = Screen::MainMenu;
            }
        }
    }
}
