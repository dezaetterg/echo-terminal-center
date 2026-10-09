use crate::collectors;
use crate::i18n::{I18n, Language};
use crate::logo_data::LOGO_LINES;
use crate::report::SystemReport;
use crate::tui::app::{App, Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, i18n: &I18n) {
    let area = f.area();

    // Check minimum size (80x20)
    if area.width < 80 || area.height < 20 {
        let warning_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Внимание ",
                Language::English => " Warning ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow).bold());

        let warning_text = Paragraph::new(i18n.terminal_too_small(area.width, area.height))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(warning_block);

        f.render_widget(warning_text, area);
        return;
    }

    // Main 3-part layout
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Bottom hints footer
        ])
        .split(area);

    // Header
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let header_title = match app.screen {
        Screen::MainMenu => Line::from(vec![
            Span::styled(
                " Echo Terminal Center ",
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Центр управления системой",
                    Language::English => "System Management Center",
                },
                Style::default().fg(Color::Gray),
            ),
        ]),
        screen => Line::from(vec![
            Span::styled(
                " Echo Terminal Center ",
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("› ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(screen.title(i18n), Style::default().fg(Color::White).bold()),
        ]),
    };

    let header_p = Paragraph::new(header_title).block(header_block);
    f.render_widget(header_p, main_chunks[0]);

    // Active screen
    match app.screen {
        Screen::MainMenu => render_main_menu(f, main_chunks[1], app, i18n),
        Screen::SystemInfo => render_system_info(f, main_chunks[1], app, i18n),
        Screen::PackageUpdates => render_packages(f, main_chunks[1], app, i18n),
        Screen::CacheCleaning => render_cache(f, main_chunks[1], app, i18n),
        Screen::ServiceManagement => render_services(f, main_chunks[1], app, i18n),
        Screen::DiskStatus => render_disks(f, main_chunks[1], app, i18n),
        Screen::NetworkConnections => render_network(f, main_chunks[1], app, i18n),
        Screen::SystemLogs => render_logs(f, main_chunks[1], app, i18n),
        Screen::Diagnostics => render_diagnostics(f, main_chunks[1], app, i18n),
        Screen::Bluetooth => render_bluetooth(f, main_chunks[1], app, i18n),
        Screen::Audio => render_audio(f, main_chunks[1], app, i18n),
        Screen::AppManagement => render_apps(f, main_chunks[1], app, i18n),
        Screen::PowerManagement => render_power(f, main_chunks[1], app, i18n),
    }

    // Confirmation dialog
    if app.has_active_dialog() {
        render_confirm_dialog(f, area, app, i18n);
    }

    // Footer hints
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer_hint = if app.has_active_dialog() {
        i18n.hint_confirm()
    } else {
        match app.screen {
            Screen::MainMenu => i18n.hint_main_menu(),
            Screen::SystemInfo => i18n.hint_system_info(),
            Screen::PackageUpdates => i18n.hint_packages(),
            Screen::CacheCleaning => i18n.hint_cache(),
            Screen::ServiceManagement => i18n.hint_services(),
            Screen::DiskStatus => i18n.hint_disks(),
            Screen::NetworkConnections => i18n.hint_network(),
            Screen::SystemLogs => i18n.hint_logs(),
            Screen::Diagnostics => i18n.hint_diagnostics(),
            Screen::Bluetooth => i18n.hint_bluetooth(),
            Screen::Audio => i18n.hint_audio(),
            Screen::AppManagement => i18n.hint_apps(),
            Screen::PowerManagement => i18n.hint_power(),
        }
    };

    let footer_p = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled(footer_hint, Style::default().fg(Color::Yellow)),
    ]))
    .block(footer_block);

    f.render_widget(footer_p, main_chunks[2]);
}

fn render_main_menu(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(38), // Left: Navigation menu
            Constraint::Min(0),     // Right: Module description & details
        ])
        .split(area);

    // Left: Menu List
    let menu_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => " Меню ",
            Language::English => " Menu ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let items = [
        (i18n.menu_system_info(), " (fetch)"),
        (i18n.menu_package_updates(), ""),
        (i18n.menu_cache_cleaning(), ""),
        (i18n.menu_service_management(), ""),
        (i18n.menu_disk_status(), ""),
        (i18n.menu_network_connections(), ""),
        (i18n.menu_system_logs(), ""),
        (i18n.menu_diagnostics(), ""),
        (i18n.menu_bluetooth(), ""),
        (i18n.menu_audio(), ""),
        (i18n.menu_apps(), ""),
        (i18n.menu_power(), ""),
    ];

    let inner_h = content_chunks[0].height.saturating_sub(2) as usize;
    let visible_count = if inner_h == 0 { items.len() } else { inner_h };
    let offset = if app.selected_index >= visible_count {
        app.selected_index - visible_count + 1
    } else {
        0
    };

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible_count)
        .map(|(idx, (name, tag))| {
            let is_selected = idx == app.selected_index;
            let prefix = if is_selected { "> " } else { "  " };
            let name_style = if is_selected {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let tag_style = if is_selected {
                Style::default().fg(Color::LightCyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan).bold()),
                Span::styled(*name, name_style),
                Span::styled(*tag, tag_style),
            ]))
        })
        .collect();

    let list = List::new(list_items).block(menu_block);
    f.render_widget(list, content_chunks[0]);

    // Right: Selected item description & details
    let details_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => " Сведения о модуле ",
            Language::English => " Module Details ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let selected_screen = app.selected_screen();
    let (desc, tool, priv_level) = match selected_screen {
        Screen::SystemInfo => (
            match i18n.lang {
                Language::Russian => "Сводка о конфигурации системы, процессоре, графике, памяти, дисках и времени работы.",
                Language::English => "Summary of system configuration, CPU, graphics, memory, storage, and uptime.",
            },
            "procfs, sysfs, nvidia-smi",
            match i18n.lang {
                Language::Russian => "Пользователь (чтение)",
                Language::English => "User (read-only)",
            },
        ),
        Screen::PackageUpdates => (
            match i18n.lang {
                Language::Russian => "Проверка наличия обновлений пакетов и их установка с запросом подтверждения.",
                Language::English => "Check for package updates and install them with confirmation.",
            },
            "pacman, apt, dnf",
            match i18n.lang {
                Language::Russian => "Root / sudo (для установки)",
                Language::English => "Root / sudo (for install)",
            },
        ),
        Screen::CacheCleaning => (
            match i18n.lang {
                Language::Russian => "Очистка неиспользуемого кэша пакетов и журналов для освобождения места на диске.",
                Language::English => "Clean up unused package cache and journals to free disk space.",
            },
            "paccache, apt-get, journalctl",
            match i18n.lang {
                Language::Russian => "Root / sudo",
                Language::English => "Root / sudo",
            },
        ),
        Screen::ServiceManagement => (
            match i18n.lang {
                Language::Russian => "Просмотр статуса системных служб, перезапуск и проверка сбойных юнитов.",
                Language::English => "View system service status, restart, and inspect failed units.",
            },
            "systemctl",
            match i18n.lang {
                Language::Russian => "Root / sudo (для изменения)",
                Language::English => "Root / sudo (for restart/stop)",
            },
        ),
        Screen::DiskStatus => (
            match i18n.lang {
                Language::Russian => "Информация о точках монтирования, свободном месте и состоянии дисков.",
                Language::English => "Mount points information, free space, and disk health.",
            },
            "df, lsblk, smartctl",
            match i18n.lang {
                Language::Russian => "Пользователь / smartctl (sudo -n)",
                Language::English => "User / smartctl (sudo -n)",
            },
        ),
        Screen::NetworkConnections => (
            match i18n.lang {
                Language::Russian => "Список активных интерфейсов, IP-адресов, шлюзов по умолчанию и статуса сети.",
                Language::English => "Active network interfaces, IP addresses, default gateways, and link status.",
            },
            "ip, nmcli",
            match i18n.lang {
                Language::Russian => "Пользователь (чтение)",
                Language::English => "User (read-only)",
            },
        ),
        Screen::SystemLogs => (
            match i18n.lang {
                Language::Russian => "Просмотр последних системных ошибок и предупреждений журнала systemd.",
                Language::English => "View recent system errors and warnings from the systemd journal.",
            },
            "journalctl -p err -n 100",
            match i18n.lang {
                Language::Russian => "Пользователь / группа systemd-journal",
                Language::English => "User / systemd-journal group",
            },
        ),
        Screen::Diagnostics => (
            match i18n.lang {
                Language::Russian => "Комплексная диагностика: температура CPU, батарея, состояние ядра и памяти.",
                Language::English => "System diagnostics: CPU temperature, battery, kernel status, and memory.",
            },
            "sysfs, /sys/class/thermal, /sys/class/power_supply",
            match i18n.lang {
                Language::Russian => "Пользователь (чтение)",
                Language::English => "User (read-only)",
            },
        ),
        Screen::Bluetooth => (
            match i18n.lang {
                Language::Russian => "Управление Bluetooth: статус адаптера, список сопряженных и подключенных устройств, заряд батареи.",
                Language::English => "Bluetooth management: adapter status, paired/connected devices, battery levels.",
            },
            "bluetoothctl, rfkill",
            match i18n.lang {
                Language::Russian => "Пользователь (чтение/управление)",
                Language::English => "User (read/write)",
            },
        ),
        Screen::Audio => (
            match i18n.lang {
                Language::Russian => "Управление аудио: переключение выходов и микрофонов по умолчанию, громкость и Mute.",
                Language::English => "Audio management: default sinks and sources, volume level, and mute.",
            },
            "pactl",
            match i18n.lang {
                Language::Russian => "Пользователь (управление)",
                Language::English => "User (read/write)",
            },
        ),
        Screen::AppManagement => (
            match i18n.lang {
                Language::Russian => "Поиск и установка приложений из репозиториев и Flatpak, а также удаление программ.",
                Language::English => "Search and install apps from repos and Flatpak, and uninstall software.",
            },
            "pacman, apt, dnf, flatpak",
            match i18n.lang {
                Language::Russian => "Root / sudo (пакеты) / Пользователь (Flatpak)",
                Language::English => "Root / sudo (packages) / User (Flatpak)",
            },
        ),
        Screen::PowerManagement => (
            match i18n.lang {
                Language::Russian => "Управление питанием: статус батареи ноутбука, выбор профиля производительности и подсветка экрана.",
                Language::English => "Power management: laptop battery stats, performance profiles, and screen brightness.",
            },
            "powerprofilesctl, brightnessctl, sysfs",
            match i18n.lang {
                Language::Russian => "Пользователь / D-Bus",
                Language::English => "User / D-Bus",
            },
        ),
        _ => ("", "", ""),
    };

    let (label_tool, label_priv, label_action) = match i18n.lang {
        Language::Russian => ("Утилиты: ", "Привилегии: ", "Enter: Открыть модуль"),
        Language::English => ("Utilities: ", "Privileges: ", "Enter: Open module"),
    };

    let details_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                selected_screen.title(i18n),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(desc, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(label_tool, Style::default().fg(Color::DarkGray)),
            Span::styled(tool, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(label_priv, Style::default().fg(Color::DarkGray)),
            Span::styled(priv_level, Style::default().fg(Color::LightBlue)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("▶ {label_action}"),
                Style::default().fg(Color::Green).bold(),
            ),
        ]),
    ];

    let details_p = Paragraph::new(details_text)
        .block(details_block)
        .wrap(Wrap { trim: true });

    f.render_widget(details_p, content_chunks[1]);
}

fn render_system_info(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let card_block = Block::default()
        .title(format!(" {} ", i18n.menu_system_info()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = card_block.inner(area);
    f.render_widget(card_block, area);

    let report = match &app.system_report {
        Some(r) => r,
        None => {
            let loading_text = Paragraph::new(match i18n.lang {
                Language::Russian => "Сбор системной информации...",
                Language::English => "Collecting system information...",
            })
            .alignment(Alignment::Center);
            f.render_widget(loading_text, inner);
            return;
        }
    };

    let top_pad = if inner.height >= 17 { 1 } else { 0 };
    let content_area = Rect {
        x: inner.x,
        y: inner.y + top_pad,
        width: inner.width,
        height: inner.height.saturating_sub(top_pad),
    };

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Left: Echo Logo
            Constraint::Min(0),     // Right: System Information
        ])
        .split(content_area);

    // Render parsed Logo
    let logo_lines: Vec<Line<'static>> = LOGO_LINES.iter().map(|l| parse_ansi_line(l)).collect();
    let logo_p = Paragraph::new(logo_lines);
    f.render_widget(logo_p, columns[0]);

    // Render System Info lines
    let info_lines = build_system_info_lines(report, i18n);
    let info_p = Paragraph::new(info_lines);
    f.render_widget(info_p, columns[1]);
}

fn build_system_info_lines(report: &SystemReport, i18n: &I18n) -> Vec<Line<'static>> {
    let host_str =
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

    let items = [
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

    let mut lines = Vec::new();

    // Line 0: Host header
    lines.push(Line::from(vec![Span::styled(
        host_str,
        Style::default().fg(Color::White).bold(),
    )]));

    // Line 1: Separator line
    lines.push(Line::from(vec![Span::styled(
        "─".repeat(48),
        Style::default().fg(Color::DarkGray),
    )]));

    // Lines 2..12: Data rows
    for (k, v) in items {
        let k_len = unicode_width(k);
        let pad = " ".repeat(key_pad.saturating_sub(k_len));

        if let Some(bracket_idx) = v.find('[') {
            let before = &v[..bracket_idx];
            let bar_str = &v[bracket_idx..];
            let fill = bar_str.chars().filter(|&c| c == '█').count();
            let empty = bar_str.chars().filter(|&c| c == '░').count();

            lines.push(Line::from(vec![
                Span::styled(k, Style::default().fg(Color::Cyan).bold()),
                Span::raw(pad),
                Span::styled(before.to_string(), Style::default().fg(Color::White)),
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled("█".repeat(fill), Style::default().fg(Color::Cyan).bold()),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::styled("]", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(k, Style::default().fg(Color::Cyan).bold()),
                Span::raw(pad),
                Span::styled(v, Style::default().fg(Color::White)),
            ]));
        }
    }

    // Line 13: Blank line
    lines.push(Line::from(""));

    // Line 14: Color palette bar
    lines.push(Line::from(vec![
        Span::styled("   ", Style::default().bg(Color::Red)),
        Span::styled("   ", Style::default().bg(Color::Green)),
        Span::styled("   ", Style::default().bg(Color::Yellow)),
        Span::styled("   ", Style::default().bg(Color::Blue)),
        Span::styled("   ", Style::default().bg(Color::Magenta)),
        Span::styled("   ", Style::default().bg(Color::Cyan)),
        Span::styled("   ", Style::default().bg(Color::White)),
        Span::styled("   ", Style::default().bg(Color::DarkGray)),
    ]));

    lines
}

fn render_packages(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_package_updates()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top status bar
            Constraint::Min(0),    // Updates list
        ])
        .split(inner);

    // Top status
    let total = app.packages_state.updates.len();
    let (label_mgr, label_count, label_act) = match i18n.lang {
        Language::Russian => ("Менеджер: ", "Обновлений: ", "▶ Enter / u: Обновить всё"),
        Language::English => ("Manager: ", "Updates: ", "▶ Enter / u: Upgrade all"),
    };

    let count_color = if total == 0 {
        Color::Green
    } else {
        Color::Yellow
    };
    let status_line = Line::from(vec![
        Span::styled(label_mgr, Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("[{}]", app.packages_state.pkg_manager),
            Style::default().fg(Color::Cyan).bold(),
        ),
        Span::raw("   "),
        Span::styled(label_count, Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{total}"), Style::default().fg(count_color).bold()),
        Span::raw("   "),
        Span::styled(label_act, Style::default().fg(Color::Green).bold()),
    ]);
    let status_p = Paragraph::new(status_line).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(status_p, chunks[0]);

    // Updates list
    if app.packages_state.updates.is_empty() {
        let msg = match i18n.lang {
            Language::Russian => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "✓ Все пакеты обновлены.",
                    Style::default().fg(Color::Green).bold(),
                )),
                Line::from(Span::styled(
                    "В системе нет пакетов, ожидающих обновления.",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
            Language::English => vec![
                Line::from(""),
                Line::from(Span::styled(
                    "✓ System is up to date.",
                    Style::default().fg(Color::Green).bold(),
                )),
                Line::from(Span::styled(
                    "No pending package updates available.",
                    Style::default().fg(Color::DarkGray),
                )),
            ],
        };
        let empty_p = Paragraph::new(msg).alignment(Alignment::Center);
        f.render_widget(empty_p, chunks[1]);
    } else {
        let visible_height = chunks[1].height as usize;
        let offset = app.packages_state.scroll_offset;
        let items: Vec<ListItem> = app
            .packages_state
            .updates
            .iter()
            .skip(offset)
            .take(visible_height)
            .map(|u| {
                ListItem::new(Line::from(vec![
                    Span::styled(" ● ", Style::default().fg(Color::Cyan)),
                    Span::styled(u.clone(), Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, chunks[1]);
    }
}

fn render_cache(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_cache_cleaning()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Package Cache
            Constraint::Length(4), // Journal Cache
            Constraint::Length(4), // Thumbnail Cache
            Constraint::Length(4), // Action bar
            Constraint::Min(0),
        ])
        .split(inner);

    let (title_pkg, title_jrn, title_usr, title_act) = match i18n.lang {
        Language::Russian => (
            " Кэш пакетов (/var/cache) ",
            " Журналы systemd (journalctl) ",
            " Кэш миниатюр (~/.cache/thumbnails) ",
            " Очистка кэша ",
        ),
        Language::English => (
            " Package Cache (/var/cache) ",
            " Systemd Journal (journalctl) ",
            " Thumbnail Cache (~/.cache/thumbnails) ",
            " Cache Cleaning ",
        ),
    };

    let pkg_block = Block::default()
        .title(title_pkg)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let pkg_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "Объем кэша: ",
                Language::English => "Cache size: ",
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            &app.cache_state.pkg_cache_str,
            Style::default().fg(Color::Yellow).bold(),
        ),
    ]);
    f.render_widget(Paragraph::new(pkg_line).block(pkg_block), chunks[0]);

    let jrn_block = Block::default()
        .title(title_jrn)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let jrn_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "Занято журналами: ",
                Language::English => "Journal disk usage: ",
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            &app.cache_state.journal_cache_str,
            Style::default().fg(Color::Yellow).bold(),
        ),
    ]);
    f.render_widget(Paragraph::new(jrn_line).block(jrn_block), chunks[1]);

    let usr_block = Block::default()
        .title(title_usr)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    let usr_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "Кэш миниатюр: ",
                Language::English => "Thumbnails size: ",
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            &app.cache_state.user_cache_str,
            Style::default().fg(Color::Yellow).bold(),
        ),
    ]);
    f.render_widget(Paragraph::new(usr_line).block(usr_block), chunks[2]);

    let act_block = Block::default()
        .title(title_act)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));
    let (cmd, args) = app.cache_state.clean_command();
    let act_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("▶ {cmd} {}", args.join(" ")),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  │  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "Enter / c: Очистить кэш пакетов",
                Language::English => "Enter / c: Clean package cache",
            },
            Style::default().fg(Color::Green).bold(),
        ),
    ]);
    f.render_widget(Paragraph::new(act_line).block(act_block), chunks[3]);
}

fn render_services(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_service_management()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(44), // Left: List of services
            Constraint::Min(0),     // Right: Selected service control
        ])
        .split(inner);

    // Left: Services List
    let (title_list, title_ctrl) = match i18n.lang {
        Language::Russian => (" Службы systemd ", " Управление службой "),
        Language::English => (" Systemd Services ", " Service Control "),
    };

    let list_block = Block::default()
        .title(title_list)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let max_visible = chunks[0].height.saturating_sub(2) as usize;
    let sel_idx = app.services_state.selected_index;
    let start_idx = if sel_idx >= max_visible {
        sel_idx + 1 - max_visible
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .services_state
        .services
        .iter()
        .enumerate()
        .skip(start_idx)
        .take(max_visible)
        .map(|(idx, s)| {
            let is_sel = idx == sel_idx;
            let cursor = if is_sel { "> " } else { "  " };

            let (badge_style, badge_text) = match s.status.as_str() {
                "active" => (Style::default().fg(Color::Green), "[ active ]"),
                "failed" => (Style::default().fg(Color::Red).bold(), "[ failed ]"),
                _ => (Style::default().fg(Color::DarkGray), "[inactive]"),
            };

            let name_style = if is_sel {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };

            let name_padded = format!("{:<22}", s.name);

            ListItem::new(Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                Span::styled(name_padded, name_style),
                Span::styled(badge_text, badge_style),
            ]))
        })
        .collect();

    let list = List::new(items).block(list_block);
    f.render_widget(list, chunks[0]);

    // Right: Selected Service details & actions
    let ctrl_block = Block::default()
        .title(title_ctrl)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(s) = app.services_state.selected_service() {
        let status_color = match s.status.as_str() {
            "active" => Color::Green,
            "failed" => Color::Red,
            _ => Color::DarkGray,
        };

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Имя службы: ",
                        Language::English => "Service name: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&s.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Текущий статус: ",
                        Language::English => "Current status: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&s.status, Style::default().fg(status_color).bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("─".repeat(34), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "▶ r: Перезапустить службу (systemctl restart)",
                        Language::English => "▶ r: Restart service (systemctl restart)",
                    },
                    Style::default().fg(Color::Yellow).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "▶ s: Остановить службу (systemctl stop)",
                        Language::English => "▶ s: Stop service (systemctl stop)",
                    },
                    Style::default().fg(Color::Red).bold(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => {
                            "Примечание: действия выполняются через sudo с запросом подтверждения."
                        }
                        Language::English => {
                            "Note: actions are executed via sudo with confirmation prompt."
                        }
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];

        let p = Paragraph::new(lines)
            .block(ctrl_block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    } else {
        let empty_p = Paragraph::new(match i18n.lang {
            Language::Russian => "Служба не выбрана",
            Language::English => "No service selected",
        })
        .block(ctrl_block)
        .alignment(Alignment::Center);
        f.render_widget(empty_p, chunks[1]);
    }
}

fn render_disks(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_disk_status()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Mount Points Table
            Constraint::Length(4), // SMART Status Box
        ])
        .split(inner);

    // Mount Points
    let (title_mounts, title_smart) = match i18n.lang {
        Language::Russian => (" Точки монтирования (df -h) ", " Состояние SMART "),
        Language::English => (" Mount Points (df -h) ", " SMART Health "),
    };

    let mounts_block = Block::default()
        .title(title_mounts)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut lines = Vec::new();

    // Table Header
    let header_line = match i18n.lang {
        Language::Russian => Line::from(vec![
            Span::styled(
                format!("{:<18}", "Файловая система"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<14}", "Монтирование"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Размер"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Занято"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Свободно"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("Заполнение", Style::default().fg(Color::Cyan).bold()),
        ]),
        Language::English => Line::from(vec![
            Span::styled(
                format!("{:<18}", "Filesystem"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<14}", "Mount point"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Size"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Used"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled(
                format!("{:<8}", "Avail"),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("Usage", Style::default().fg(Color::Cyan).bold()),
        ]),
    };
    lines.push(header_line);
    lines.push(Line::from(Span::styled(
        "─".repeat(74),
        Style::default().fg(Color::DarkGray),
    )));

    for p in &app.disks_state.partitions {
        let fill = (p.use_percent as usize) / 10;
        let empty = 10usize.saturating_sub(fill);
        let bar_color = if p.use_percent >= 85 {
            Color::Red
        } else if p.use_percent >= 70 {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let fs_short = if p.filesystem.len() > 17 {
            format!("{}…", &p.filesystem[..16])
        } else {
            p.filesystem.clone()
        };
        let mount_short = if p.mount.len() > 13 {
            format!("{}…", &p.mount[..12])
        } else {
            p.mount.clone()
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<18}", fs_short),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:<14}", mount_short),
                Style::default().fg(Color::LightCyan),
            ),
            Span::styled(
                format!("{:<8}", p.size),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{:<8}", p.used), Style::default().fg(Color::White)),
            Span::styled(format!("{:<8}", p.avail), Style::default().fg(Color::Green)),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled("█".repeat(fill), Style::default().fg(bar_color).bold()),
            Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("] {:>3}%", p.use_percent),
                Style::default().fg(bar_color),
            ),
        ]));
    }

    let mounts_p = Paragraph::new(lines).block(mounts_block);
    f.render_widget(mounts_p, chunks[0]);

    // SMART status
    let smart_block = Block::default()
        .title(title_smart)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let smart_color = if app.disks_state.smart_status.contains("PASSED")
        || app.disks_state.smart_status.contains("OK")
    {
        Color::Green
    } else if app.disks_state.smart_status.contains("FAILED") {
        Color::Red
    } else {
        Color::Yellow
    };

    let smart_p = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("● ", Style::default().fg(smart_color)),
        Span::styled(
            &app.disks_state.smart_status,
            Style::default().fg(Color::White),
        ),
    ]))
    .block(smart_block);
    f.render_widget(smart_p, chunks[1]);
}

fn render_network(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_network_connections()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Routing & DNS summary
            Constraint::Min(0),    // Network Interfaces
        ])
        .split(inner);

    // Summary Box
    let (title_summary, title_ifaces) = match i18n.lang {
        Language::Russian => (" Маршрутизация и DNS ", " Сетевые интерфейсы "),
        Language::English => (" Routing & DNS ", " Network Interfaces "),
    };

    let summary_block = Block::default()
        .title(title_summary)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let gw = app.network_state.default_gateway.as_deref().unwrap_or("—");
    let dns = if app.network_state.dns_servers.is_empty() {
        "—".to_string()
    } else {
        app.network_state.dns_servers.join(", ")
    };

    let summary_lines = vec![Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "Основной шлюз: ",
                Language::English => "Default Gateway: ",
            },
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(gw, Style::default().fg(Color::Cyan)),
        Span::raw("   │   "),
        Span::styled("DNS: ", Style::default().fg(Color::DarkGray)),
        Span::styled(dns, Style::default().fg(Color::Yellow)),
    ])];
    let summary_p = Paragraph::new(summary_lines).block(summary_block);
    f.render_widget(summary_p, chunks[0]);

    // Interfaces
    let ifaces_block = Block::default()
        .title(title_ifaces)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut iface_lines = Vec::new();
    for iface in &app.network_state.interfaces {
        let is_up = iface.state.to_uppercase() == "UP";
        let (badge_text, badge_style) = if is_up {
            ("[ UP ]", Style::default().fg(Color::Green).bold())
        } else {
            ("[ DOWN ]", Style::default().fg(Color::DarkGray))
        };

        let mac = iface.mac.as_deref().unwrap_or("—");
        let ips = if iface.ips.is_empty() {
            "—".to_string()
        } else {
            iface.ips.join(", ")
        };

        iface_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(badge_text, badge_style),
            Span::raw(" "),
            Span::styled(
                format!("{:<16}", iface.name),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("MAC: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:<18}", mac), Style::default().fg(Color::White)),
            Span::styled("IP: ", Style::default().fg(Color::DarkGray)),
            Span::styled(ips, Style::default().fg(Color::White)),
        ]));
    }

    if iface_lines.is_empty() {
        iface_lines.push(Line::from(match i18n.lang {
            Language::Russian => "  Сетевые интерфейсы не обнаружены",
            Language::English => "  No network interfaces detected",
        }));
    }

    let ifaces_p = Paragraph::new(iface_lines).block(ifaces_block);
    f.render_widget(ifaces_p, chunks[1]);
}

fn render_logs(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let total = app.logs_state.lines.len();
    let block = Block::default()
        .title(format!(" {} ({} строк) ", i18n.menu_system_logs(), total))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.logs_state.lines.is_empty() {
        let msg = match i18n.lang {
            Language::Russian => {
                "В системном журнале нет зарегистрированных ошибок (journalctl -p err)."
            }
            Language::English => {
                "No error records registered in systemd journal (journalctl -p err)."
            }
        };
        let empty_p = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Green),
        )))
        .alignment(Alignment::Center);
        f.render_widget(empty_p, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let offset = app.logs_state.scroll_offset;

    let lines: Vec<Line> = app
        .logs_state
        .lines
        .iter()
        .skip(offset)
        .take(visible_height)
        .map(|l| {
            let lower = l.to_lowercase();
            let style = if lower.contains("error")
                || lower.contains("failed")
                || lower.contains("emerg")
                || lower.contains("crit")
            {
                Style::default().fg(Color::Red)
            } else if lower.contains("warn") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Line::from(Span::styled(l.clone(), style))
        })
        .collect();

    let logs_p = Paragraph::new(lines);
    f.render_widget(logs_p, inner);
}

fn render_diagnostics(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let block = Block::default()
        .title(format!(" {} ", i18n.menu_diagnostics()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 2x2 grid
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bottom_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let (title_temp, title_bat, title_kern, title_swap) = match i18n.lang {
        Language::Russian => (
            " Температуры датчиков ",
            " Питание и батарея ",
            " Состояние ядра ",
            " Swap память ",
        ),
        Language::English => (
            " Temperature Sensors ",
            " Power & Battery ",
            " Kernel Status ",
            " Swap Memory ",
        ),
    };

    let temp_block = Block::default()
        .title(title_temp)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut temp_lines = Vec::new();
    for sensor in app.diagnostics_state.temperatures.iter().take(6) {
        let t_color = if sensor.temp_c >= 80.0 {
            Color::Red
        } else if sensor.temp_c >= 60.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        temp_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{:<20}", sensor.label),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("{:>5.1} °C", sensor.temp_c),
                Style::default().fg(t_color).bold(),
            ),
        ]));
    }

    if temp_lines.is_empty() {
        temp_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Датчики температуры не обнаружены",
                    Language::English => "No temperature sensors detected",
                },
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    f.render_widget(Paragraph::new(temp_lines).block(temp_block), top_cols[0]);

    let bat_block = Block::default()
        .title(title_bat)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let bat_lines = if let Some(b) = &app.diagnostics_state.battery {
        let fill = (b.capacity_percent as usize) / 10;
        let empty = 10usize.saturating_sub(fill);
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Батарея: ",
                        Language::English => "Battery: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&b.name, Style::default().fg(Color::White).bold()),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Заряд:   ",
                        Language::English => "Charge:  ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled("█".repeat(fill), Style::default().fg(Color::Green).bold()),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("] {}%", b.capacity_percent),
                    Style::default().fg(Color::Green).bold(),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Статус:  ",
                        Language::English => "Status:  ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&b.status, Style::default().fg(Color::Cyan)),
            ]),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("● ", Style::default().fg(Color::Green)),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Настольный компьютер (питание от сети)",
                        Language::English => "Desktop PC (AC Power)",
                    },
                    Style::default().fg(Color::White).bold(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Батарея питания не обнаружена.",
                        Language::English => "No battery found.",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ]
    };

    f.render_widget(Paragraph::new(bat_lines).block(bat_block), top_cols[1]);

    let kern_block = Block::default()
        .title(title_kern)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let (tainted_text, tainted_color) = if app.diagnostics_state.kernel_tainted {
        match i18n.lang {
            Language::Russian => ("Внимание: Ядро модифицировано (tainted)", Color::Yellow),
            Language::English => ("Warning: Kernel is tainted", Color::Yellow),
        }
    } else {
        match i18n.lang {
            Language::Russian => ("✓ Чистое ядро (Clean / Untainted)", Color::Green),
            Language::English => ("✓ Clean Kernel (Untainted)", Color::Green),
        }
    };

    let kern_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Версия ядра: ",
                    Language::English => "Kernel version: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                &app.diagnostics_state.kernel_version,
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(tainted_text, Style::default().fg(tainted_color).bold()),
        ]),
    ];

    f.render_widget(Paragraph::new(kern_lines).block(kern_block), bottom_cols[0]);

    let swap_block = Block::default()
        .title(title_swap)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let swap_fill = (app.diagnostics_state.swap_percent as usize) / 10;
    let swap_empty = 10usize.saturating_sub(swap_fill);
    let swap_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Всего:  ",
                    Language::English => "Total:  ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} MB", app.diagnostics_state.swap_total_mb),
                Style::default().fg(Color::White),
            ),
            Span::raw("   │   "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Занято: ",
                    Language::English => "Used:   ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} MB", app.diagnostics_state.swap_used_mb),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "█".repeat(swap_fill),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::styled("░".repeat(swap_empty), Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("] {:>5.1}%", app.diagnostics_state.swap_percent),
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(swap_lines).block(swap_block), bottom_cols[1]);
}

fn render_confirm_dialog(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let dialog_area = centered_rect_exact(58, 8, area);

    // Clear background beneath modal
    f.render_widget(Clear, dialog_area);

    let cmd_str = app
        .active_dialog_command()
        .unwrap_or_else(|| "command".to_string());

    let block = Block::default()
        .title(i18n.confirm_title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow).bold());

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Будет выполнена системная команда:",
                    Language::English => "The following command will be executed:",
                },
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::raw("  ▶ "),
            Span::styled(cmd_str, Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[ y: Подтвердить ]",
                Style::default().fg(Color::Green).bold(),
            ),
            Span::raw("   "),
            Span::styled(
                "[ n / Esc: Отмена ]",
                Style::default().fg(Color::Red).bold(),
            ),
        ]),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, dialog_area);
}

fn centered_rect_exact(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

fn unicode_width(s: &str) -> usize {
    s.chars().count()
}

fn parse_ansi_line(s: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut fg: Option<Color> = None;
    let mut bg: Option<Color> = None;

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut text_buf = String::new();

    while i < chars.len() {
        if chars[i] == '\x1b' && i + 1 < chars.len() && chars[i + 1] == '[' {
            if !text_buf.is_empty() {
                let mut style = Style::default();
                if let Some(f) = fg {
                    style = style.fg(f);
                }
                if let Some(b) = bg {
                    style = style.bg(b);
                }
                spans.push(Span::styled(std::mem::take(&mut text_buf), style));
            }

            i += 2; // skip '\x1b['
            let mut seq = String::new();
            while i < chars.len() && chars[i] != 'm' {
                seq.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && chars[i] == 'm' {
                i += 1;
            }

            let parts: Vec<&str> = seq.split(';').collect();
            if parts.is_empty() || parts[0] == "0" || parts[0].is_empty() {
                fg = None;
                bg = None;
            } else if parts.len() == 5 && parts[0] == "38" && parts[1] == "2" {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[2].parse::<u8>(),
                    parts[3].parse::<u8>(),
                    parts[4].parse::<u8>(),
                ) {
                    fg = Some(Color::Rgb(r, g, b));
                }
            } else if parts.len() == 5 && parts[0] == "48" && parts[1] == "2" {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[2].parse::<u8>(),
                    parts[3].parse::<u8>(),
                    parts[4].parse::<u8>(),
                ) {
                    bg = Some(Color::Rgb(r, g, b));
                }
            }
        } else {
            text_buf.push(chars[i]);
            i += 1;
        }
    }

    if !text_buf.is_empty() {
        let mut style = Style::default();
        if let Some(f) = fg {
            style = style.fg(f);
        }
        if let Some(b) = bg {
            style = style.bg(b);
        }
        spans.push(Span::styled(text_buf, style));
    }

    Line::from(spans)
}

fn render_bluetooth(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    if !app.bluetooth_state.bluetooth_available {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Bluetooth ",
                Language::English => " Bluetooth ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let msg = match i18n.lang {
            Language::Russian => {
                "Bluetooth не обнаружен (адаптер выключен или bluetoothctl недоступен)"
            }
            Language::English => {
                "Bluetooth not detected (adapter disabled or bluetoothctl unavailable)"
            }
        };
        let p = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left: Devices list
    let list_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => " Сопряженные устройства ",
            Language::English => " Paired Devices ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let max_visible = chunks[0].height.saturating_sub(2) as usize;
    let sel_idx = app.bluetooth_state.selected_index;
    let start_idx = if sel_idx >= max_visible {
        sel_idx + 1 - max_visible
    } else {
        0
    };

    let items: Vec<ListItem> = if app.bluetooth_state.devices.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Нет сопряженных устройств",
                    Language::English => "No paired devices",
                },
                Style::default().fg(Color::DarkGray),
            ),
        ]))]
    } else {
        app.bluetooth_state
            .devices
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(max_visible)
            .map(|(idx, dev)| {
                let is_sel = idx == sel_idx;
                let cursor = if is_sel { "> " } else { "  " };

                let (badge_style, badge_text) = if dev.connected {
                    (Style::default().fg(Color::Green).bold(), "[ Подключено ]")
                } else {
                    (Style::default().fg(Color::DarkGray), "[Отключено]")
                };

                let name_style = if is_sel {
                    Style::default().fg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::White)
                };

                let battery_text = match dev.battery_percent {
                    Some(pct) => format!(" ⚡{pct}%"),
                    None => String::new(),
                };

                let dev_name = if dev.name.len() > 18 {
                    format!("{}…", &dev.name[..17])
                } else {
                    format!("{:<18}", dev.name)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                    Span::styled(dev_name, name_style),
                    Span::raw(" "),
                    Span::styled(badge_text, badge_style),
                    Span::styled(battery_text, Style::default().fg(Color::LightCyan)),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(list_block);
    f.render_widget(list, chunks[0]);

    // Right: Adapter status & actions
    let ctrl_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => " Управление Bluetooth ",
            Language::English => " Bluetooth Control ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let (pwr_style, pwr_text) = if app.bluetooth_state.adapter_powered {
        (
            Style::default().fg(Color::Green).bold(),
            match i18n.lang {
                Language::Russian => "ВКЛЮЧЕН",
                Language::English => "ON",
            },
        )
    } else {
        (
            Style::default().fg(Color::Red).bold(),
            match i18n.lang {
                Language::Russian => "ВЫКЛЮЧЕН",
                Language::English => "OFF",
            },
        )
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Адаптер: ",
                    Language::English => "Adapter: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                &app.bluetooth_state.adapter_name,
                Style::default().fg(Color::White).bold(),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Питание: ",
                    Language::English => "Power: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(pwr_text, pwr_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "─".repeat(chunks[1].width.saturating_sub(6) as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(dev) = app.bluetooth_state.selected_device() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Выбранное устройство:",
                    Language::English => "Selected Device:",
                },
                Style::default().fg(Color::Cyan).bold(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Имя: ",
                    Language::English => "Name: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&dev.name, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("MAC: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&dev.mac, Style::default().fg(Color::LightCyan)),
        ]));
        if let Some(pct) = dev.battery_percent {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Заряд: ",
                        Language::English => "Battery: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{pct}%"), Style::default().fg(Color::Green).bold()),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                if dev.connected {
                    match i18n.lang {
                        Language::Russian => "c / Enter: Отключить",
                        Language::English => "c / Enter: Disconnect",
                    }
                } else {
                    match i18n.lang {
                        Language::Russian => "c / Enter: Подключить",
                        Language::English => "c / Enter: Connect",
                    }
                },
                Style::default().fg(Color::Yellow),
            ),
        ]));
    }

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "p: Вкл/Выкл питание адаптера",
                Language::English => "p: Toggle adapter power",
            },
            Style::default().fg(Color::Cyan),
        ),
    ]));

    let p = Paragraph::new(lines).block(ctrl_block);
    f.render_widget(p, chunks[1]);
}

fn render_audio(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    if !app.audio_state.audio_available {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Аудио ",
                Language::English => " Audio ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let msg = match i18n.lang {
            Language::Russian => "Аудиосистема не обнаружена (утилита pactl не найдена в системе)",
            Language::English => "Audio system not detected (pactl utility not found)",
        };
        let p = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let tab0_style = if app.audio_state.active_tab == 0 {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let tab1_style = if app.audio_state.active_tab == 1 {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let tab_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "[ 1. Выходные устройства (Sinks) ]",
                Language::English => "[ 1. Output Devices (Sinks) ]",
            },
            tab0_style,
        ),
        Span::raw("    "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "[ 2. Входные устройства (Sources) ]",
                Language::English => "[ 2. Input Devices (Sources) ]",
            },
            tab1_style,
        ),
        Span::raw("    "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "(Tab: переключить)",
                Language::English => "(Tab: switch tab)",
            },
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let tab_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(tab_line).block(tab_block), top_chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(top_chunks[1]);

    let current_devices = if app.audio_state.active_tab == 0 {
        &app.audio_state.sinks
    } else {
        &app.audio_state.sources
    };
    let sel_idx = if app.audio_state.active_tab == 0 {
        app.audio_state.selected_sink_idx
    } else {
        app.audio_state.selected_source_idx
    };

    let list_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => {
                if app.audio_state.active_tab == 0 {
                    " Выходные устройства "
                } else {
                    " Микрофоны "
                }
            }
            Language::English => {
                if app.audio_state.active_tab == 0 {
                    " Output Sinks "
                } else {
                    " Input Sources "
                }
            }
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let max_visible = chunks[0].height.saturating_sub(2) as usize;
    let start_idx = if sel_idx >= max_visible {
        sel_idx + 1 - max_visible
    } else {
        0
    };

    let items: Vec<ListItem> = if current_devices.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Устройства не найдены",
                    Language::English => "No devices found",
                },
                Style::default().fg(Color::DarkGray),
            ),
        ]))]
    } else {
        current_devices
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(max_visible)
            .map(|(idx, dev)| {
                let is_sel = idx == sel_idx;
                let cursor = if is_sel { "> " } else { "  " };

                let def_marker = if dev.is_default { "* " } else { "  " };
                let def_style = Style::default().fg(Color::LightCyan).bold();

                let name_style = if is_sel {
                    Style::default().fg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::White)
                };

                let vol_style = if dev.is_muted {
                    Style::default().fg(Color::Red).bold()
                } else {
                    Style::default().fg(Color::Green)
                };

                let vol_badge = if dev.is_muted {
                    "[MUTE]".to_string()
                } else {
                    format!("[{:>3}%]", dev.volume_percent)
                };

                let short_name = if dev.name.len() > 24 {
                    format!("{}…", &dev.name[..23])
                } else {
                    format!("{:<24}", dev.name)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                    Span::styled(def_marker, def_style),
                    Span::styled(short_name, name_style),
                    Span::raw(" "),
                    Span::styled(vol_badge, vol_style),
                ]))
            })
            .collect()
    };

    let list = List::new(items).block(list_block);
    f.render_widget(list, chunks[0]);

    // Right: Controls & details
    let ctrl_block = Block::default()
        .title(match i18n.lang {
            Language::Russian => " Управление звуком ",
            Language::English => " Audio Controls ",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut lines = vec![Line::from("")];
    if let Some(dev) = app.audio_state.selected_device() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Устройство: ",
                    Language::English => "Device: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&dev.name, Style::default().fg(Color::White).bold()),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&dev.id, Style::default().fg(Color::LightCyan)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "По умолчанию: ",
                    Language::English => "Default: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                if dev.is_default {
                    match i18n.lang {
                        Language::Russian => "Да (активное)",
                        Language::English => "Yes (active)",
                    }
                } else {
                    match i18n.lang {
                        Language::Russian => "Нет",
                        Language::English => "No",
                    }
                },
                if dev.is_default {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]));

        let filled = (dev.volume_percent as usize * 15) / 100;
        let empty = 15usize.saturating_sub(filled);
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Громкость: ",
                    Language::English => "Volume: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                bar,
                Style::default().fg(if dev.is_muted {
                    Color::Red
                } else {
                    Color::Green
                }),
            ),
            Span::raw(" "),
            Span::styled(
                format!("{}%", dev.volume_percent),
                Style::default().fg(Color::White).bold(),
            ),
            if dev.is_muted {
                Span::styled(" (Muted)", Style::default().fg(Color::Red).bold())
            } else {
                Span::raw("")
            },
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "─".repeat(chunks[1].width.saturating_sub(6) as usize),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "Enter: Сделать по умолчанию",
                    Language::English => "Enter: Set as default",
                },
                Style::default().fg(Color::Cyan),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "+ / -: Громкость (+/-5%)",
                    Language::English => "+ / -: Volume (+/-5%)",
                },
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                match i18n.lang {
                    Language::Russian => "m: Вкл/Выкл звук (Mute)",
                    Language::English => "m: Toggle Mute",
                },
                Style::default().fg(Color::LightCyan),
            ),
        ]));
    }

    let p = Paragraph::new(lines).block(ctrl_block);
    f.render_widget(p, chunks[1]);
}

fn render_apps(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let tab0_style = if app.apps_state.active_tab == 0 {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let tab1_style = if app.apps_state.active_tab == 1 {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let tab_line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "[ 1. Поиск и установка ]",
                Language::English => "[ 1. Search & Install ]",
            },
            tab0_style,
        ),
        Span::raw("    "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "[ 2. Удаление установленного ]",
                Language::English => "[ 2. Installed & Remove ]",
            },
            tab1_style,
        ),
        Span::raw("    "),
        Span::styled(
            match i18n.lang {
                Language::Russian => "(Tab: сменить режим)",
                Language::English => "(Tab: switch mode)",
            },
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let tab_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(Paragraph::new(tab_line).block(tab_block), top_chunks[0]);

    if app.apps_state.active_tab == 0 {
        // Tab 0: Search & Install
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(top_chunks[1]);

        let search_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Поиск пакета или Flatpak ",
                Language::English => " Search Package or Flatpak ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if app.apps_state.search_input_active {
                Style::default().fg(Color::Yellow).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let input_text = if app.apps_state.search_input_active {
            format!(
                " {}_ (Enter: искать, Esc: выход)",
                app.apps_state.search_query
            )
        } else if app.apps_state.search_query.is_empty() {
            match i18n.lang {
                Language::Russian => " Нажмите '/' для ввода поискового запроса...".to_string(),
                Language::English => " Press '/' to enter search query...".to_string(),
            }
        } else {
            format!(
                " {} (нажмите '/' для изменения)",
                app.apps_state.search_query
            )
        };

        f.render_widget(
            Paragraph::new(input_text).block(search_block),
            content_chunks[0],
        );

        let res_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(content_chunks[1]);

        let list_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Результаты поиска ",
                Language::English => " Search Results ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let max_visible = res_chunks[0].height.saturating_sub(2) as usize;
        let sel_idx = app.apps_state.search_idx;
        let start_idx = if sel_idx >= max_visible {
            sel_idx + 1 - max_visible
        } else {
            0
        };

        let items: Vec<ListItem> = if app.apps_state.search_results.is_empty() {
            vec![ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Введите запрос в строку поиска выше",
                        Language::English => "Enter query in search bar above",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]))]
        } else {
            app.apps_state
                .search_results
                .iter()
                .enumerate()
                .skip(start_idx)
                .take(max_visible)
                .map(|(idx, item)| {
                    let is_sel = idx == sel_idx;
                    let cursor = if is_sel { "> " } else { "  " };

                    let (source_style, source_tag) = match item.source {
                        crate::tui::screens::apps::AppSource::Flatpak => {
                            (Style::default().fg(Color::Magenta), "[flatpak]")
                        }
                        crate::tui::screens::apps::AppSource::Native => {
                            (Style::default().fg(Color::Blue), "[repo]")
                        }
                    };

                    let name_style = if is_sel {
                        Style::default().fg(Color::Cyan).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let short_name = if item.name.len() > 18 {
                        format!("{}…", &item.name[..17])
                    } else {
                        format!("{:<18}", item.name)
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                        Span::styled(short_name, name_style),
                        Span::raw(" "),
                        Span::styled(source_tag, source_style),
                        if item.is_installed {
                            Span::styled(" [установлен]", Style::default().fg(Color::Green))
                        } else {
                            Span::raw("")
                        },
                    ]))
                })
                .collect()
        };

        f.render_widget(List::new(items).block(list_block), res_chunks[0]);

        let details_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Сведения о приложении ",
                Language::English => " App Details ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut lines = vec![Line::from("")];
        if let Some(item) = app.apps_state.search_results.get(app.apps_state.search_idx) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Название: ",
                        Language::English => "Name: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&item.name, Style::default().fg(Color::White).bold()),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("ID / Пакет: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&item.id_or_pkg, Style::default().fg(Color::LightCyan)),
            ]));
            if !item.version.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        match i18n.lang {
                            Language::Russian => "Версия: ",
                            Language::English => "Version: ",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(&item.version, Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&item.description, Style::default().fg(Color::Gray)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Enter / i: Установить выбранное приложение",
                        Language::English => "Enter / i: Install selected application",
                    },
                    Style::default().fg(Color::Yellow).bold(),
                ),
            ]));
        }

        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .block(details_block),
            res_chunks[1],
        );
    } else {
        // Tab 1: Installed & Remove
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(top_chunks[1]);

        let list_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Установленные приложения ",
                Language::English => " Installed Applications ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let max_visible = chunks[0].height.saturating_sub(2) as usize;
        let sel_idx = app.apps_state.installed_idx;
        let start_idx = if sel_idx >= max_visible {
            sel_idx + 1 - max_visible
        } else {
            0
        };

        let items: Vec<ListItem> = if app.apps_state.installed_apps.is_empty() {
            vec![ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Список приложений пуст или загружается...",
                        Language::English => "Installed apps list is empty...",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]))]
        } else {
            app.apps_state
                .installed_apps
                .iter()
                .enumerate()
                .skip(start_idx)
                .take(max_visible)
                .map(|(idx, item)| {
                    let is_sel = idx == sel_idx;
                    let cursor = if is_sel { "> " } else { "  " };

                    let (source_style, source_tag) = match item.source {
                        crate::tui::screens::apps::AppSource::Flatpak => {
                            (Style::default().fg(Color::Magenta), "[flatpak]")
                        }
                        crate::tui::screens::apps::AppSource::Native => {
                            (Style::default().fg(Color::Blue), "[repo]")
                        }
                    };

                    let name_style = if is_sel {
                        Style::default().fg(Color::Cyan).bold()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let short_name = if item.name.len() > 18 {
                        format!("{}…", &item.name[..17])
                    } else {
                        format!("{:<18}", item.name)
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                        Span::styled(short_name, name_style),
                        Span::raw(" "),
                        Span::styled(source_tag, source_style),
                    ]))
                })
                .collect()
        };

        f.render_widget(List::new(items).block(list_block), chunks[0]);

        let details_block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Сведения и удаление ",
                Language::English => " Details & Removal ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut lines = vec![Line::from("")];
        if let Some(item) = app
            .apps_state
            .installed_apps
            .get(app.apps_state.installed_idx)
        {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Название: ",
                        Language::English => "Name: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&item.name, Style::default().fg(Color::White).bold()),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("ID / Пакет: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&item.id_or_pkg, Style::default().fg(Color::LightCyan)),
            ]));
            if !item.version.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        match i18n.lang {
                            Language::Russian => "Версия: ",
                            Language::English => "Version: ",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(&item.version, Style::default().fg(Color::White)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&item.description, Style::default().fg(Color::Gray)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Enter / d: Удалить выбранное приложение",
                        Language::English => "Enter / d: Uninstall selected application",
                    },
                    Style::default().fg(Color::Red).bold(),
                ),
            ]));
        }

        f.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .block(details_block),
            chunks[1],
        );
    }
}

fn render_power(f: &mut Frame, area: Rect, app: &App, i18n: &I18n) {
    let has_profiles = app.power_state.has_powerprofiles;
    let has_bat = app.power_state.has_battery;
    let has_bright = app.power_state.has_brightness;

    // Desktop check: if neither battery, power profiles, nor brightness exist
    if !has_profiles && !has_bat && !has_bright {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Питание системы ",
                Language::English => " System Power ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Статус электропитания: ",
                        Language::English => "Power supply status: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Питание от электросети (Настольный ПК)",
                        Language::English => "AC Line Power (Desktop PC)",
                    },
                    Style::default().fg(Color::Green).bold(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Мобильные функции (управление батареей, подсветка дисплея и профили энергопотребления)",
                        Language::English => "Mobile power features (battery status, screen backlight, power-saving profiles)",
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "скрыты, так как они не применимы к данному оборудованию.",
                        Language::English => "are hidden as they do not apply to this hardware configuration.",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];

        let p = Paragraph::new(lines).block(block);
        f.render_widget(p, area);
        return;
    }

    // Build constraints based only on available blocks
    let mut constraints = Vec::new();
    if has_profiles {
        constraints.push(Constraint::Length(7));
    }
    if has_bat {
        constraints.push(Constraint::Length(7));
    }
    if has_bright {
        constraints.push(Constraint::Length(6));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    if has_profiles {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Профиль энергопотребления ",
                Language::English => " Power Profile ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut lines = vec![Line::from("")];
        for (idx, prof) in app.power_state.profiles.iter().enumerate() {
            let is_sel = idx == app.power_state.selected_profile_idx;
            let is_cur = prof == &app.power_state.current_profile;

            let radio = if is_cur { "(•) " } else { "( ) " };
            let cursor = if is_sel { "> " } else { "  " };

            let prof_style = if is_sel {
                Style::default().fg(Color::Cyan).bold()
            } else if is_cur {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            let tag = if is_cur {
                match i18n.lang {
                    Language::Russian => " [текущий]",
                    Language::English => " [active]",
                }
            } else {
                ""
            };

            lines.push(Line::from(vec![
                Span::styled(cursor, Style::default().fg(Color::Cyan).bold()),
                Span::styled(radio, Style::default().fg(Color::LightCyan)),
                Span::styled(prof.as_str(), prof_style),
                Span::styled(tag, Style::default().fg(Color::Green)),
            ]));
        }

        f.render_widget(Paragraph::new(lines).block(block), chunks[chunk_idx]);
        chunk_idx += 1;
    }

    if has_bat {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Аккумулятор ноутбука ",
                Language::English => " Laptop Battery ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let mut lines = vec![Line::from("")];
        if let Some(bat) = &app.power_state.battery {
            let filled = (bat.percent as usize * 15) / 100;
            let empty = 15usize.saturating_sub(filled);
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    bar,
                    Style::default().fg(if bat.percent > 20 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{}%", bat.percent),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("({})", bat.status),
                    Style::default().fg(Color::LightCyan),
                ),
            ]));

            if let Some(health) = bat.health_percent {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        match i18n.lang {
                            Language::Russian => "Состояние аккумулятора (Health): ",
                            Language::English => "Battery Health: ",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(format!("{health}%"), Style::default().fg(Color::Green)),
                ]));
            }

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "Устройство: ",
                        Language::English => "Device: ",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&bat.name, Style::default().fg(Color::LightCyan)),
            ]));
        }

        f.render_widget(Paragraph::new(lines).block(block), chunks[chunk_idx]);
        chunk_idx += 1;
    }

    if has_bright {
        let block = Block::default()
            .title(match i18n.lang {
                Language::Russian => " Яркость экрана ",
                Language::English => " Screen Brightness ",
            })
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let pct = app.power_state.brightness_percent;
        let filled = (pct as usize * 15) / 100;
        let empty = 15usize.saturating_sub(filled);
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(bar, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(format!("{pct}%"), Style::default().fg(Color::White).bold()),
                Span::raw("  "),
                Span::styled(
                    match i18n.lang {
                        Language::Russian => "(+/-: изменить на 10%)",
                        Language::English => "(+/-: adjust by 10%)",
                    },
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];

        f.render_widget(Paragraph::new(lines).block(block), chunks[chunk_idx]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{I18n, Language};
    use crate::tui::app::{App, Screen};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_app_navigation_and_selection() {
        let mut app = App::new();
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.screen, Screen::MainMenu);

        // Move down
        app.next_item();
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.selected_screen(), Screen::PackageUpdates);

        // Move up
        app.prev_item();
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.selected_screen(), Screen::SystemInfo);

        // Cycle up from 0 to last (item 11 = PowerManagement)
        app.prev_item();
        assert_eq!(app.selected_index, App::MENU_ITEM_COUNT - 1);
        assert_eq!(app.selected_screen(), Screen::PowerManagement);

        // Cycle down from last to 0
        app.next_item();
        assert_eq!(app.selected_index, 0);

        // Select item 0 -> SystemInfo
        app.select();
        assert_eq!(app.screen, Screen::SystemInfo);
        assert!(app.system_report.is_some());

        // Esc returns to MainMenu
        app.go_back();
        assert_eq!(app.screen, Screen::MainMenu);
        assert!(app.running);

        // Esc on MainMenu sets running = false
        app.go_back();
        assert!(!app.running);
    }

    #[test]
    fn test_terminal_too_small_warning() {
        let i18n = I18n {
            lang: Language::Russian,
        };
        let app = App::new();
        let backend = TestBackend::new(60, 18);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw(f, &app, &i18n);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Внимание") || content.contains("80"));
    }

    #[test]
    fn test_main_menu_draw_80x24() {
        let i18n = I18n {
            lang: Language::Russian,
        };
        let app = App::new();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw(f, &app, &i18n);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Echo Terminal Center"));
        assert!(content.contains("Системная информация"));
        assert!(content.contains("Навигация"));
    }

    #[test]
    fn test_all_12_screens_draw_80x24() {
        let i18n = I18n {
            lang: Language::Russian,
        };
        let screens = [
            Screen::SystemInfo,
            Screen::PackageUpdates,
            Screen::CacheCleaning,
            Screen::ServiceManagement,
            Screen::DiskStatus,
            Screen::NetworkConnections,
            Screen::SystemLogs,
            Screen::Diagnostics,
            Screen::Bluetooth,
            Screen::Audio,
            Screen::AppManagement,
            Screen::PowerManagement,
        ];

        for screen in screens {
            let mut app = App::new();
            app.screen = screen;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();

            terminal
                .draw(|f| {
                    draw(f, &app, &i18n);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content()
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();

            assert!(content.contains("Echo Terminal Center"));
        }
    }

    #[test]
    fn test_menu_and_screens_draw_80x20() {
        let i18n = I18n {
            lang: Language::Russian,
        };
        let mut app = App::new();

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw(f, &app, &i18n);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Echo Terminal Center"));
        assert!(content.contains("Системная информация"));
        assert!(content.contains("Bluetooth"));
        assert!(content.contains("Аудио"));
        assert!(content.contains("Питание"));

        let screens = [
            Screen::SystemInfo,
            Screen::PackageUpdates,
            Screen::CacheCleaning,
            Screen::ServiceManagement,
            Screen::DiskStatus,
            Screen::NetworkConnections,
            Screen::SystemLogs,
            Screen::Diagnostics,
            Screen::Bluetooth,
            Screen::Audio,
            Screen::AppManagement,
            Screen::PowerManagement,
        ];

        for screen in screens {
            app.screen = screen;
            let b = TestBackend::new(80, 20);
            let mut t = Terminal::new(b).unwrap();
            t.draw(|f| {
                draw(f, &app, &i18n);
            })
            .unwrap();
        }
    }

    #[test]
    fn test_confirm_dialog_draw() {
        let i18n = I18n {
            lang: Language::Russian,
        };
        let mut app = App::new();
        app.screen = Screen::PackageUpdates;
        app.packages_state.confirm_upgrade = true;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                draw(f, &app, &i18n);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content()
            .iter()
            .map(|c| c.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(content.contains("Подтверждение"));
        assert!(content.contains("Подтвердить"));
    }

    #[test]
    fn test_parse_ansi_line_colors() {
        let sample = "\x1b[38;2;255;0;128m▄\x1b[0m\x1b[48;2;0;100;200m▀\x1b[0m";
        let line = parse_ansi_line(sample);
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "▄");
        assert_eq!(line.spans[0].style.fg, Some(Color::Rgb(255, 0, 128)));
        assert_eq!(line.spans[1].content, "▀");
        assert_eq!(line.spans[1].style.bg, Some(Color::Rgb(0, 100, 200)));
    }
}
