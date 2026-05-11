pub mod recipe_runtime;

fn sync_openclaw_desktop_state(handle: &tauri::AppHandle) {
    if let Ok(recipe) = recipe_runtime::recipe_loader::load_recipe("openclaw") {
        let _ = recipe_runtime::status_store::mark_running(handle, "openclaw");
        let _ = recipe_runtime::port_resolver::clear_overrides(handle, "openclaw");
        let _ = recipe_runtime::status_store::mark_port_resolution(
            handle,
            "openclaw",
            "unchanged".to_string(),
            chrono::Utc::now().to_rfc3339(),
            "OpenClaw Desktop is using its managed local port.".to_string(),
            vec![recipe_runtime::status::RecipePortMapping {
                host: "127.0.0.1".to_string(),
                requested_port: 18789,
                resolved_port: 18789,
                changed: false,
            }],
            recipe.dashboard_url(),
            recipe_runtime::port_resolver::effective_readiness_url(handle, &recipe),
        );
        let _ = recipe_runtime::status_store::mark_healthy(handle, "openclaw");
    }
}

fn spawn_openclaw_startup_repair(handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        if let Err(error) = recipe_runtime::native_cli::auto_heal_openclaw_startup(&handle, 18789) {
            let _ = recipe_runtime::logs::append(&handle, "openclaw", "startup", &error);
            let _ = recipe_runtime::status_store::mark_error(&handle, "openclaw", error);
        } else {
            sync_openclaw_desktop_state(&handle);
        }
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            recipe_runtime::paths::state_root_dir(&handle)?;
            recipe_runtime::status_store::sync_library_state_on_startup(&handle)?;
            spawn_openclaw_startup_repair(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            recipe_runtime::commands::recipe_list_apps,
            recipe_runtime::commands::recipe_get_status,
            recipe_runtime::commands::recipe_get_install_plan,
            recipe_runtime::commands::recipe_run_resource_preflight,
            recipe_runtime::commands::recipe_resolve_ports,
            recipe_runtime::commands::recipe_accept_install_plan,
            recipe_runtime::commands::recipe_clear_install_plan_acceptance,
            recipe_runtime::commands::recipe_check_environment,
            recipe_runtime::commands::recipe_install,
            recipe_runtime::commands::recipe_start,
            recipe_runtime::commands::recipe_stop,
            recipe_runtime::commands::recipe_restart,
            recipe_runtime::commands::recipe_open_dashboard,
            recipe_runtime::commands::recipe_check_health,
            recipe_runtime::commands::recipe_check_gateway_status,
            recipe_runtime::commands::recipe_check_runtime_status,
            recipe_runtime::commands::recipe_check_readiness,
            recipe_runtime::commands::recipe_read_logs,
            recipe_runtime::commands::recipe_run_doctor,
            recipe_runtime::commands::recipe_run_onboarding,
            recipe_runtime::commands::recipe_repair,
            recipe_runtime::commands::recipe_get_runtime_error,
            recipe_runtime::commands::recipe_retry_runtime_error,
            recipe_runtime::commands::recipe_repair_runtime_error,
            recipe_runtime::commands::recipe_save_secrets,
            recipe_runtime::commands::recipe_configure_openclaw,
            recipe_runtime::commands::recipe_rollback_failed_install,
            recipe_runtime::commands::recipe_import_user_recipe,
            recipe_runtime::commands::recipe_remove_user_recipe,
            recipe_runtime::commands::recipe_fetch_marketplace,
            recipe_runtime::commands::recipe_uninstall,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenNest Desktop Starter");
}

pub fn run_openclaw_desktop() {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            recipe_runtime::paths::state_root_dir(&handle)?;
            recipe_runtime::status_store::sync_library_state_on_startup(&handle)?;
            recipe_runtime::native_cli::auto_heal_openclaw_startup(&handle, 18789)
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
            sync_openclaw_desktop_state(&handle);

            let port = 18789;
            let app_url = recipe_runtime::native_cli::openclaw_dashboard(&handle, port)
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;

            if let Some(main_window) = handle.get_webview_window("main") {
                let _ = main_window.close();
            }

            let external_url = app_url
                .parse()
                .map_err(|error| -> Box<dyn std::error::Error> { format!("invalid OpenClaw Desktop URL: {error}").into() })?;

            WebviewWindowBuilder::new(&handle, "openclaw-desktop", WebviewUrl::External(external_url))
                .title("OpenClaw Desktop")
                .inner_size(1440.0, 920.0)
                .min_inner_size(1100.0, 720.0)
                .resizable(true)
                .focused(true)
                .build()
                .map_err(|error| -> Box<dyn std::error::Error> { format!("failed to open OpenClaw Desktop window: {error}").into() })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenClaw Desktop");
}
