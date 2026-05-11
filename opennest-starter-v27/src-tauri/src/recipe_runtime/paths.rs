use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn appdata_root(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

pub fn root_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = legacy_root_dir().unwrap_or(appdata_root(app)?.join("apps"));
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

pub fn app_dir(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    let dir = root_dir(app)?.join(app_id);
    fs::create_dir_all(dir.join("logs")).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn log_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(app_dir(app, app_id)?.join("logs").join(format!("{}.log", app_id)))
}

pub fn compose_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(app_dir(app, app_id)?.join("docker-compose.yml"))
}

pub fn state_root_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join("OpenNest").join("state").join("apps"),
        None => appdata_root(app)?.join("state").join("apps"),
    };
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

pub fn state_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(state_root_dir(app)?.join(format!("{app_id}.json")))
}

fn legacy_root_dir() -> Option<PathBuf> {
    let root = std::env::var_os("APPDATA")
        .map(PathBuf::from)?
        .join("com.opennest.desktop")
        .join("apps");
    if root.exists() {
        Some(root)
    } else {
        None
    }
}
