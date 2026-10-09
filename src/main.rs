mod collectors;
mod i18n;
mod layout;
mod logo_data;
mod report;
mod tui;

use i18n::I18n;
use report::SystemReport;
use std::env;
use std::io::IsTerminal;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "fetch" => {
                let i18n = I18n::detect();
                let report = SystemReport::collect();
                layout::render(&report, &i18n);
                return;
            }
            "--json" => {
                let report = SystemReport::collect();
                match report.to_json() {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        eprintln!("Error serializing report to JSON: {e}");
                        process::exit(1);
                    }
                }
                return;
            }
            "--short" => {
                let report = SystemReport::collect();
                println!("{}", report.to_short());
                return;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                println!("echo-terminal {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                print_help();
                process::exit(1);
            }
        }
    }

    // Default: TUI mode
    if !std::io::stdout().is_terminal() {
        eprintln!("Error: stdout is not a TTY. Use --json, --short, or 'echo-terminal fetch' for scripts.");
        process::exit(1);
    }

    // TUI entrypoint
    let i18n = I18n::detect();
    if let Err(e) = tui::run(&i18n) {
        eprintln!("Terminal error: {e}");
        process::exit(1);
    }
}

fn print_help() {
    println!("echo-terminal - System Center & Information for Echo\n");
    println!("USAGE:");
    println!("    echo-terminal [COMMAND | OPTIONS]\n");
    println!("COMMANDS:");
    println!("    fetch            Print system information card directly to terminal\n");
    println!("OPTIONS:");
    println!("    --json           Print system information as JSON");
    println!("    --short          Print compact one-line summary (for scripts / status bars)");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
}
