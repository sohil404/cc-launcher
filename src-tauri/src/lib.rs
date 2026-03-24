mod claude_meta;
mod commands;
mod config;
mod launcher;
mod scanner;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let cfg = config::load_config();
    let projects = scanner::scan_directories(&cfg.scan_dirs, cfg.scan_depth, &cfg.pinned, &cfg.recents);

    let mut builder = MenuBuilder::new(app);

    // Pinned projects
    let pinned: Vec<_> = projects.iter().filter(|p| p.pinned).collect();
    if !pinned.is_empty() {
        let header = MenuItemBuilder::with_id("_header_pinned", "Pinned")
            .enabled(false)
            .build(app)?;
        builder = builder.item(&header);

        for p in &pinned {
            let item = MenuItemBuilder::with_id(
                format!("launch:{}", p.path),
                &p.name,
            ).build(app)?;
            builder = builder.item(&item);
        }
        builder = builder.item(&PredefinedMenuItem::separator(app)?);
    }

    // Recent projects (top 5)
    let recent: Vec<_> = projects.iter()
        .filter(|p| !p.pinned && (p.last_launched.is_some() || p.claude.last_active.is_some()))
        .take(5)
        .collect();

    if !recent.is_empty() {
        let header = MenuItemBuilder::with_id("_header_recent", "Recent")
            .enabled(false)
            .build(app)?;
        builder = builder.item(&header);

        for p in &recent {
            let label = match &p.claude.last_active_ago {
                Some(ago) => format!("{}  ·  {}", p.name, ago),
                None => p.name.clone(),
            };
            let item = MenuItemBuilder::with_id(
                format!("launch:{}", p.path),
                &label,
            ).build(app)?;
            builder = builder.item(&item);
        }
        builder = builder.item(&PredefinedMenuItem::separator(app)?);
    }

    // Actions
    let show = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
    let refresh = MenuItemBuilder::with_id("refresh_tray", "Refresh Projects").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit CC Launcher").build(app)?;

    builder = builder
        .item(&show)
        .item(&refresh)
        .item(&PredefinedMenuItem::separator(app)?)
        .item(&quit);

    builder.build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_projects,
            commands::launch_project,
            commands::toggle_pin,
            commands::get_config,
            commands::update_config,
            commands::rescan_projects,
            commands::get_available_terminals,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let menu = build_tray_menu(&handle)?;

            let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(icon)
                .icon_as_template(true)
                .tooltip("CC Launcher")
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    let id = event.id().as_ref();

                    if id.starts_with("launch:") {
                        let path = &id[7..];
                        let cfg = config::load_config();
                        let _ = launcher::launch_in_terminal(path, &cfg.launch_flags, &cfg.terminal);
                        let mut cfg = cfg;
                        config::add_recent(&mut cfg, path);
                        let _ = config::save_config(&cfg);
                        if let Ok(new_menu) = build_tray_menu(app) {
                            if let Some(tray) = app.tray_by_id("main-tray") {
                                let _ = tray.set_menu(Some(new_menu));
                            }
                        }
                        return;
                    }

                    match id {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "refresh_tray" => {
                            if let Ok(new_menu) = build_tray_menu(app) {
                                if let Some(tray) = app.tray_by_id("main-tray") {
                                    let _ = tray.set_menu(Some(new_menu));
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .menu_on_left_click(true)
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running CC Launcher");
}
