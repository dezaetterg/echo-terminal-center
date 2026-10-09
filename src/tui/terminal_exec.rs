use crate::i18n::{I18n, Language};
use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout, Write};
use std::process::Command;

pub fn execute_external(cmd: &str, args: &[&str], i18n: &I18n) -> io::Result<bool> {
    terminal::disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;

    let title = match i18n.lang {
        Language::Russian => "Выполнение команды:",
        Language::English => "Executing command:",
    };
    let full_cmd = format!("{cmd} {}", args.join(" "));
    println!("\n\x1b[1;36m=== {title} {full_cmd} ===\x1b[0m\n");

    // Inherit stdio for interactive sudo and package manager prompts
    let status = Command::new(cmd).args(args).status();

    let success = match status {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("Error executing command: {e}");
            false
        }
    };

    let prompt = match i18n.lang {
        Language::Russian => {
            "\n\x1b[1;33m[Нажмите Enter для возврата в Echo Terminal Center...]\x1b[0m"
        }
        Language::English => {
            "\n\x1b[1;33m[Press Enter to return to Echo Terminal Center...]\x1b[0m"
        }
    };
    print!("{prompt}");
    let _ = stdout().flush();

    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);

    terminal::enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;

    Ok(success)
}
