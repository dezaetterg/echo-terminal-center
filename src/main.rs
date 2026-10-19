mod collectors;
mod i18n;
mod layout;
mod logo_data;
mod report;
mod tui;

use anyhow::{bail, Context, Result};
use i18n::I18n;
use report::SystemReport;
use std::env;
use std::io::IsTerminal;

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "fetch" => {
                let i18n = I18n::detect();
                let report = SystemReport::collect();
                layout::render(&report, &i18n);
                return Ok(());
            }
            "--json" => {
                let report = SystemReport::collect();
                let json = report.to_json().context("serializing report to JSON")?;
                println!("{json}");
                return Ok(());
            }
            "--short" => {
                let report = SystemReport::collect();
                println!("{}", report.to_short());
                return Ok(());
            }
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-V" | "--version" => {
                println!("echo-terminal {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            unknown => {
                bail!("unknown argument: {unknown}\n\nRun `echo-terminal --help` for usage.");
            }
        }
    }

    if !std::io::stdout().is_terminal() {
        bail!("stdout is not a TTY — use --json, --short, or 'echo-terminal fetch' for scripts");
    }

    let i18n = I18n::detect();
    tui::run(&i18n).context("TUI runtime error")?;

    Ok(())
}

fn print_help() {
    println!("echo-terminal — System Center & Information\n");
    println!("USAGE:");
    println!("    echo-terminal [COMMAND | OPTIONS]\n");
    println!("COMMANDS:");
    println!("    fetch            Print system information card to terminal\n");
    println!("OPTIONS:");
    println!("    --json           Print system information as JSON");
    println!("    --short          Print compact one-line summary (for scripts / status bars)");
    println!("    -h, --help       Print help information");
    println!("    -V, --version    Print version information");
}
