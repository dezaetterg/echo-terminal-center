use std::env;

pub fn get_de_info() -> Option<String> {
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .ok()?;

    if desktop.is_empty() {
        return None;
    }

    let session = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let de_name = desktop.split(':').next_back().unwrap_or(&desktop).trim();

    if !session.is_empty() && session != "unspecified" {
        Some(format!("{de_name} ({session})"))
    } else {
        Some(de_name.to_string())
    }
}
