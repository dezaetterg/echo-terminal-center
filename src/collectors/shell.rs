use std::env;
use std::path::Path;

pub fn get_shell() -> String {
    if let Ok(shell_path) = env::var("SHELL") {
        if let Some(file_name) = Path::new(&shell_path).file_name() {
            let name = file_name.to_string_lossy().trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }

    "—".to_string()
}
