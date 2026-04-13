mod commands;
mod types;

use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};

pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            // ── Menu bar ───────────────────────────────────────────
            let add_project = MenuItemBuilder::with_id("add_project", "Add Project…")
                .accelerator("CmdOrCtrl+O")
                .build(handle)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit VeriPatch")
                .accelerator("CmdOrCtrl+Q")
                .build(handle)?;

            let file_menu = SubmenuBuilder::new(handle, "File")
                .item(&add_project)
                .separator()
                .item(&quit)
                .build()?;

            let theme_light = MenuItemBuilder::with_id("theme_light", "Light").build(handle)?;
            let theme_dark = MenuItemBuilder::with_id("theme_dark", "Dark").build(handle)?;
            let theme_system = MenuItemBuilder::with_id("theme_system", "System").build(handle)?;

            let theme_sub = SubmenuBuilder::new(handle, "Theme")
                .item(&theme_light)
                .item(&theme_dark)
                .item(&theme_system)
                .build()?;

            let view_menu = SubmenuBuilder::new(handle, "View")
                .item(&theme_sub)
                .build()?;

            let menu = MenuBuilder::new(handle)
                .item(&file_menu)
                .item(&view_menu)
                .build()?;

            app.set_menu(menu)?;

            // ── Menu event handler ─────────────────────────────────
            let handle2 = handle.clone();
            app.on_menu_event(move |_app, event| {
                let window = handle2.get_webview_window("main").unwrap();
                match event.id().0.as_str() {
                    "quit" => std::process::exit(0),
                    "add_project" => {
                        let _ = window.eval("window.addProjectFromMenu()");
                    }
                    "theme_light" => {
                        let _ = window.eval("window.setThemeFromMenu('light')");
                    }
                    "theme_dark" => {
                        let _ = window.eval("window.setThemeFromMenu('dark')");
                    }
                    "theme_system" => {
                        let _ = window.eval("window.setThemeFromMenu('system')");
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .manage(types::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::set_theme,
            commands::add_project,
            commands::remove_project,
            commands::select_project,
            commands::set_input_source,
            commands::set_clipboard_diff,
            commands::pick_patch_file,
            commands::run_verification,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run VeriPatch");

    Ok(())
}
