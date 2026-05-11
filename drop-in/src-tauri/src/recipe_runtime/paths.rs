use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn root_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let root = base.join("apps");
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
