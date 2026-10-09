pub mod app;
pub mod screens;
pub mod terminal_exec;
pub mod ui;

use crate::i18n::I18n;
use app::{App, Screen};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::panic;
use std::time::Duration;

pub fn run(i18n: &I18n) -> io::Result<()> {
    // Restore terminal on panic
    let default_panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, cursor::Show);
        default_panic_hook(panic_info);
    }));

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_loop(&mut terminal, &mut app, i18n);

    let _ = terminal::disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show);
    let _ = terminal.show_cursor();

    res
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    i18n: &I18n,
) -> io::Result<()> {
    let mut needs_redraw = true;

    while app.running {
        if needs_redraw {
            terminal.draw(|f| ui::draw(f, app, i18n))?;
            needs_redraw = false;
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    needs_redraw = true;
                    // Global exit on Ctrl+C
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        app.running = false;
                        continue;
                    }

                    // Active confirmation modal
                    if app.has_active_dialog() {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                if app.packages_state.confirm_upgrade {
                                    let (cmd, args) = app.packages_state.upgrade_command();
                                    let _ = terminal_exec::execute_external(cmd, &args, i18n);
                                    app.packages_state.confirm_upgrade = false;
                                    app.packages_state.load();
                                    terminal.clear()?;
                                } else if app.cache_state.confirm_clean {
                                    let (cmd, args) = app.cache_state.clean_command();
                                    let _ = terminal_exec::execute_external(cmd, &args, i18n);
                                    app.cache_state.confirm_clean = false;
                                    app.cache_state.load();
                                    terminal.clear()?;
                                } else if let Some((srv, action)) =
                                    app.services_state.confirm_action.take()
                                {
                                    let _ = terminal_exec::execute_external(
                                        "sudo",
                                        &["systemctl", action, &srv],
                                        i18n,
                                    );
                                    app.services_state.load();
                                    terminal.clear()?;
                                } else if let Some((cmd, args)) =
                                    app.apps_state.confirm_action.take()
                                {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ = terminal_exec::execute_external(&cmd, &args_refs, i18n);
                                    app.apps_state.load();
                                    terminal.clear()?;
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.close_dialog();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Search input mode
                    if app.screen == Screen::AppManagement && app.apps_state.search_input_active {
                        match key.code {
                            KeyCode::Enter => {
                                app.apps_state.search_input_active = false;
                                app.apps_state.perform_search();
                            }
                            KeyCode::Esc => {
                                app.apps_state.search_input_active = false;
                            }
                            KeyCode::Backspace => {
                                app.apps_state.search_query.pop();
                            }
                            KeyCode::Char(c) => {
                                app.apps_state.search_query.push(c);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match app.screen {
                        Screen::MainMenu => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.running = false;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.prev_item();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.next_item();
                            }
                            KeyCode::Enter => {
                                app.select();
                            }
                            _ => {}
                        },
                        Screen::SystemInfo => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::PackageUpdates => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Enter | KeyCode::Char('u') => {
                                if !app.packages_state.updates.is_empty() {
                                    app.packages_state.confirm_upgrade = true;
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.packages_state.scroll_up();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.packages_state.scroll_down(15);
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::CacheCleaning => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Enter | KeyCode::Char('c') => {
                                app.cache_state.confirm_clean = true;
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::ServiceManagement => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.services_state.prev();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.services_state.next();
                            }
                            KeyCode::Char('r') => {
                                if let Some(s) = app.services_state.selected_service() {
                                    app.services_state.confirm_action =
                                        Some((s.name.clone(), "restart"));
                                }
                            }
                            KeyCode::Char('s') => {
                                if let Some(s) = app.services_state.selected_service() {
                                    app.services_state.confirm_action =
                                        Some((s.name.clone(), "stop"));
                                }
                            }
                            _ => {}
                        },
                        Screen::DiskStatus => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::NetworkConnections => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::SystemLogs => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.logs_state.scroll_up();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.logs_state.scroll_down(15);
                            }
                            KeyCode::PageUp => {
                                app.logs_state.page_up(15);
                            }
                            KeyCode::PageDown => {
                                app.logs_state.page_down(15, 15);
                            }
                            KeyCode::Home => {
                                app.logs_state.scroll_home();
                            }
                            KeyCode::End => {
                                app.logs_state.scroll_end(15);
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::Diagnostics => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::Bluetooth => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.bluetooth_state.prev();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.bluetooth_state.next();
                            }
                            KeyCode::Char('p') => {
                                let (cmd, args) = app.bluetooth_state.toggle_power_cmd();
                                let _ = std::process::Command::new(cmd).args(&args).output();
                                app.bluetooth_state.load();
                            }
                            KeyCode::Enter | KeyCode::Char('c') => {
                                if let Some((cmd, args)) = app.bluetooth_state.connect_toggle_cmd()
                                {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.bluetooth_state.load();
                                }
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::Audio => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Tab => {
                                app.audio_state.switch_tab();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.audio_state.prev();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.audio_state.next();
                            }
                            KeyCode::Enter => {
                                if let Some((cmd, args)) = app.audio_state.set_default_cmd() {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.audio_state.load();
                                }
                            }
                            KeyCode::Char('m') => {
                                if let Some((cmd, args)) = app.audio_state.toggle_mute_cmd() {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.audio_state.load();
                                }
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                if let Some((cmd, args)) = app.audio_state.volume_up_cmd() {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.audio_state.load();
                                }
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                if let Some((cmd, args)) = app.audio_state.volume_down_cmd() {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.audio_state.load();
                                }
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::AppManagement => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Tab => {
                                app.apps_state.switch_tab();
                            }
                            KeyCode::Char('/') => {
                                app.apps_state.search_input_active = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.apps_state.prev();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.apps_state.next();
                            }
                            KeyCode::Enter | KeyCode::Char('i')
                                if app.apps_state.active_tab == 0 =>
                            {
                                if let Some((cmd, args)) = app.apps_state.prepare_install_command()
                                {
                                    app.apps_state.confirm_action = Some((cmd, args));
                                }
                            }
                            KeyCode::Enter | KeyCode::Char('d')
                                if app.apps_state.active_tab == 1 =>
                            {
                                if let Some((cmd, args)) = app.apps_state.prepare_remove_command() {
                                    app.apps_state.confirm_action = Some((cmd, args));
                                }
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                        Screen::PowerManagement => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.go_back();
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.power_state.prev_profile();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.power_state.next_profile();
                            }
                            KeyCode::Enter | KeyCode::Char('p') => {
                                if let Some((cmd, args)) = app.power_state.set_profile_cmd() {
                                    let args_refs: Vec<&str> =
                                        args.iter().map(|s| s.as_str()).collect();
                                    let _ =
                                        std::process::Command::new(cmd).args(&args_refs).output();
                                    app.power_state.load();
                                }
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                if let Some((cmd, args)) = app.power_state.brightness_up_cmd() {
                                    let _ = std::process::Command::new(cmd).args(&args).output();
                                    app.power_state.load();
                                }
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                if let Some((cmd, args)) = app.power_state.brightness_down_cmd() {
                                    let _ = std::process::Command::new(cmd).args(&args).output();
                                    app.power_state.load();
                                }
                            }
                            KeyCode::Char('r') => {
                                app.refresh_current_screen();
                            }
                            _ => {}
                        },
                    }
                }
                Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
