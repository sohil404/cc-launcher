use crate::{config, launcher, scanner};

#[tauri::command]
pub fn get_projects() -> Result<Vec<scanner::Project>, String> {
    let cfg = config::load_config();
    let projects =
        scanner::scan_directories(&cfg.scan_dirs, cfg.scan_depth, &cfg.pinned, &cfg.recents);
    Ok(projects)
}

#[tauri::command]
pub fn launch_project(path: String, mode: Option<String>) -> Result<(), String> {
    let mut cfg = config::load_config();
    let flags = match mode.as_deref() {
        Some("continue") => {
            if cfg.launch_flags.is_empty() {
                "--continue".to_string()
            } else {
                format!("--continue {}", cfg.launch_flags)
            }
        }
        Some("resume") => {
            if cfg.launch_flags.is_empty() {
                "--resume".to_string()
            } else {
                format!("--resume {}", cfg.launch_flags)
            }
        }
        _ => cfg.launch_flags.clone(),
    };
    launcher::launch_in_terminal(&path, &flags, &cfg.terminal)?;
    config::add_recent(&mut cfg, &path);
    config::save_config(&cfg)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_pin(path: String) -> Result<bool, String> {
    let mut cfg = config::load_config();
    let is_pinned = if cfg.pinned.contains(&path) {
        cfg.pinned.retain(|p| p != &path);
        false
    } else {
        cfg.pinned.push(path);
        true
    };
    config::save_config(&cfg)?;
    Ok(is_pinned)
}

#[tauri::command]
pub fn get_config() -> Result<config::AppConfig, String> {
    Ok(config::load_config())
}

#[tauri::command]
pub fn update_config(config_update: config::AppConfig) -> Result<(), String> {
    config::save_config(&config_update)
}

#[tauri::command]
pub fn rescan_projects() -> Result<Vec<scanner::Project>, String> {
    get_projects()
}

#[tauri::command]
pub fn get_available_terminals() -> Result<Vec<String>, String> {
    Ok(launcher::detect_available_terminals())
}
