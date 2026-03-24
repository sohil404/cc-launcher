use std::process::Command;

#[cfg(target_os = "macos")]
use std::path::Path;

#[cfg(target_os = "macos")]
const MAC_TERMINALS: &[(&str, &[&str])] = &[
    ("iterm2", &["/Applications/iTerm.app", "/Users/*/Applications/iTerm.app"]),
    ("warp", &["/Applications/Warp.app"]),
    ("alacritty", &["/Applications/Alacritty.app"]),
    ("kitty", &["/Applications/kitty.app"]),
    ("terminal", &["/System/Applications/Utilities/Terminal.app"]),
];

#[cfg(target_os = "macos")]
fn mac_terminal_exists(paths: &[&str]) -> bool {
    for pattern in paths {
        if pattern.contains('*') {
            if let Some(home) = dirs::home_dir() {
                let expanded = pattern.replace("/Users/*", &home.to_string_lossy());
                if Path::new(&expanded).exists() {
                    return true;
                }
            }
        } else if Path::new(pattern).exists() {
            return true;
        }
    }
    false
}

/// Shell-escape a string for use in single-quoted shell contexts.
/// Replaces ' with '\'' (end quote, escaped quote, start quote).
fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Escape a string for use inside AppleScript double-quoted strings.
fn applescript_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn launch_in_terminal(
    project_path: &str,
    flags: &str,
    terminal: &str,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let chosen = if terminal == "auto" {
            detect_mac_terminal()
        } else {
            terminal.to_string()
        };

        match chosen.as_str() {
            "iterm2" => launch_iterm2(project_path, flags),
            "warp" => launch_warp(project_path, flags),
            "alacritty" => launch_alacritty(project_path, flags),
            "kitty" => launch_kitty(project_path, flags),
            _ => launch_terminal_app(project_path, flags),
        }?;
    }

    #[cfg(target_os = "linux")]
    {
        launch_linux_terminal(project_path, flags, terminal)?;
    }

    #[cfg(target_os = "windows")]
    {
        launch_windows_terminal(project_path, flags)?;
    }

    Ok(())
}

pub fn detect_available_terminals() -> Vec<String> {
    let mut terminals = Vec::new();

    #[cfg(target_os = "macos")]
    {
        for (name, paths) in MAC_TERMINALS {
            if mac_terminal_exists(paths) {
                terminals.push(name.to_string());
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let linux_terms = [
            "gnome-terminal", "konsole", "alacritty", "kitty",
            "xfce4-terminal", "wezterm",
        ];
        for term in &linux_terms {
            if Command::new("which")
                .arg(term)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                terminals.push(term.to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        terminals.push("windows-terminal".to_string());
        terminals.push("cmd".to_string());
    }

    terminals
}

// ── macOS terminals ──

#[cfg(target_os = "macos")]
fn detect_mac_terminal() -> String {
    for (name, paths) in MAC_TERMINALS {
        if *name != "terminal" && mac_terminal_exists(paths) {
            return name.to_string();
        }
    }
    "terminal".to_string()
}

#[cfg(target_os = "macos")]
fn build_shell_cmd(project_path: &str, flags: &str) -> String {
    // Safe: path is single-quote escaped, flags are user-controlled but intentional
    format!("cd '{}' && claude {}", shell_escape(project_path), flags)
}

#[cfg(target_os = "macos")]
fn launch_terminal_app(project_path: &str, flags: &str) -> Result<(), String> {
    let cmd = build_shell_cmd(project_path, flags);
    Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"Terminal\"\n\
                    activate\n\
                    do script \"{}\"\n\
                end tell",
                applescript_escape(&cmd)
            ),
        ])
        .spawn()
        .map_err(|e| format!("Failed to open Terminal: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_iterm2(project_path: &str, flags: &str) -> Result<(), String> {
    let cmd = build_shell_cmd(project_path, flags);
    Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"iTerm\"\n\
                    activate\n\
                    tell current window\n\
                        create tab with default profile\n\
                        tell current session\n\
                            write text \"{}\"\n\
                        end tell\n\
                    end tell\n\
                end tell",
                applescript_escape(&cmd)
            ),
        ])
        .spawn()
        .map_err(|e| format!("Failed to open iTerm2: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_warp(project_path: &str, flags: &str) -> Result<(), String> {
    let cmd = build_shell_cmd(project_path, flags);
    // Warp doesn't have great AppleScript support; use open + keystroke
    Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"Warp\" to activate\n\
                delay 0.5\n\
                tell application \"System Events\"\n\
                    keystroke \"t\" using command down\n\
                    delay 0.3\n\
                    keystroke \"{}\"\n\
                    key code 36\n\
                end tell",
                applescript_escape(&cmd)
            ),
        ])
        .spawn()
        .map_err(|e| format!("Failed to open Warp: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_alacritty(project_path: &str, flags: &str) -> Result<(), String> {
    let shell_cmd = build_shell_cmd(project_path, flags);
    Command::new("/Applications/Alacritty.app/Contents/MacOS/alacritty")
        .args(["-e", "bash", "-c", &shell_cmd])
        .spawn()
        .map_err(|e| format!("Failed to open Alacritty: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_kitty(project_path: &str, flags: &str) -> Result<(), String> {
    let shell_cmd = build_shell_cmd(project_path, flags);
    Command::new("/Applications/kitty.app/Contents/MacOS/kitty")
        .args(["bash", "-c", &shell_cmd])
        .spawn()
        .map_err(|e| format!("Failed to open Kitty: {}", e))?;
    Ok(())
}

// ── Linux (M7 fix: pass args correctly per terminal) ──

#[cfg(target_os = "linux")]
fn launch_linux_terminal(project_path: &str, flags: &str, terminal: &str) -> Result<(), String> {
    let shell_cmd = format!("cd '{}' && claude {}", shell_escape(project_path), flags);

    // Terminals that take: terminal [args] bash -c "command"
    let term_configs: &[(&str, &[&str])] = &[
        ("gnome-terminal", &["--"]),
        ("konsole", &["-e"]),
        ("xfce4-terminal", &["--command"]),
        ("wezterm", &["start", "--"]),
        ("x-terminal-emulator", &["-e"]),
    ];

    // Terminals that take: terminal -e bash -c "command" as separate args
    let direct_exec: &[&str] = &["alacritty", "kitty"];

    let launch = |term: &str| -> Result<(), String> {
        if direct_exec.contains(&term) {
            Command::new(term)
                .args(["-e", "bash", "-c", &shell_cmd])
                .spawn()
                .map_err(|e| format!("Failed to open {}: {}", term, e))?;
            return Ok(());
        }

        let args = term_configs
            .iter()
            .find(|(name, _)| *name == term)
            .map(|(_, args)| *args)
            .unwrap_or(&["--"]);

        let mut c = Command::new(term);
        for a in args {
            c.arg(a);
        }
        // Pass bash -c as separate arguments, not as one string
        c.args(["bash", "-c", &shell_cmd])
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", term, e))?;
        Ok(())
    };

    if terminal != "auto" {
        return launch(terminal);
    }

    // Auto-detect
    let all_terms: &[&str] = &[
        "gnome-terminal", "konsole", "alacritty", "kitty",
        "xfce4-terminal", "wezterm", "x-terminal-emulator",
    ];

    for term in all_terms {
        if Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return launch(term);
        }
    }
    Err("No supported terminal emulator found".to_string())
}

// ── Windows (M6 fix: proper quoting for cmd.exe) ──

#[cfg(target_os = "windows")]
fn launch_windows_terminal(project_path: &str, flags: &str) -> Result<(), String> {
    // Windows uses double quotes and cd /d for drive changes
    let escaped_path = project_path.replace('"', "\"\"");
    let cmd = format!("cd /d \"{}\" && claude {}", escaped_path, flags);

    // Try Windows Terminal first
    let wt_result = Command::new("wt.exe")
        .args(["new-tab", "cmd", "/k", &cmd])
        .spawn();

    if wt_result.is_ok() {
        return Ok(());
    }

    // Fallback to PowerShell
    let ps_cmd = format!(
        "Set-Location '{}'; claude {}",
        project_path.replace('\'', "''"),
        flags
    );
    let ps_result = Command::new("powershell")
        .args(["-NoExit", "-Command", &ps_cmd])
        .spawn();

    if ps_result.is_ok() {
        return Ok(());
    }

    // Last resort: cmd.exe
    Command::new("cmd")
        .args(["/k", &cmd])
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;
    Ok(())
}
