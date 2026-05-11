use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use tauri::AppHandle;

use super::{paths, secret_redaction_registry};

pub fn redact(input: &str) -> String {
    secret_redaction_registry::redact(input)
}

pub fn append(app: &AppHandle, app_id: &str, category: &str, line: &str) -> Result<(), String> {
    let path = paths::log_file(app, app_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path).map_err(|e| e.to_string())?;
    let safe = redact(line);
    writeln!(file, "{} [{}] {}", Utc::now().to_rfc3339(), category, safe).map_err(|e| e.to_string())
}

pub fn read_tail(app: &AppHandle, app_id: &str, limit: usize) -> Result<Vec<String>, String> {
    let path = paths::log_file(app, app_id)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let start = lines.len().saturating_sub(limit);
    Ok(lines[start..].to_vec())
}
