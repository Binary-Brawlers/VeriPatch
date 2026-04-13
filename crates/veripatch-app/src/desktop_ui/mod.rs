mod commands;
mod storage;
mod types;

use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};

pub fn run() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();
            let app_state = storage::load_or_initialize_state(handle)?;
            app.manage(app_state);

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

            let edit_menu = SubmenuBuilder::new(handle, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .separator()
                .select_all()
                .build()?;

            let open_settings = MenuItemBuilder::with_id("open_settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(handle)?;

            let theme_light = MenuItemBuilder::with_id("theme_light", "Light").build(handle)?;
            let theme_dark = MenuItemBuilder::with_id("theme_dark", "Dark").build(handle)?;
            let theme_system = MenuItemBuilder::with_id("theme_system", "System").build(handle)?;

            let theme_sub = SubmenuBuilder::new(handle, "Theme")
                .item(&theme_light)
                .item(&theme_dark)
                .item(&theme_system)
                .build()?;

            let view_menu = SubmenuBuilder::new(handle, "View")
                .item(&open_settings)
                .separator()
                .item(&theme_sub)
                .build()?;

            let menu = MenuBuilder::new(handle)
                .item(&file_menu)
                .item(&edit_menu)
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
                    "open_settings" => {
                        let _ = window.eval("window.openSettingsFromMenu()");
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_run_history,
            commands::set_theme,
            commands::add_project,
            commands::remove_project,
            commands::select_project,
            commands::set_input_source,
            commands::set_clipboard_diff,
            commands::pick_patch_file,
            commands::run_verification,
            commands::export_markdown_report,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run VeriPatch");

    Ok(())
}
