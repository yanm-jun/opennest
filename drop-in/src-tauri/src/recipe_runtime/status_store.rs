use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

use super::paths;
use super::recipe_loader;
use super::secret_redaction_registry;
use super::status::{RecipePortMapping, RecipeStatus};

fn status_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, app_id)?.join("status.json"))
}

pub fn load(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let path = status_file(app, app_id)?;
    if !path.exists() {
        return Ok(RecipeStatus::default_for(app_id));
    }

    let text = fs::read_to_string(&path).map_err(|e| format!("failed to read status file: {e}"))?;
    match serde_json::from_str::<RecipeStatus>(&text) {
        Ok(mut status) => {
            // Keep status files resilient to manual edits / older versions.
            if status.app_id.trim().is_empty() {
                status.app_id = app_id.to_string();
            }
            Ok(status)
        }
        Err(error) => {
            let mut status = RecipeStatus::default_for(app_id);
            status.install_state = "error".to_string();
            status.run_state = "error".to_string();
            status.last_error = Some(format!("status.json is corrupted: {error}"));
            Ok(status)
        }
    }
}

pub fn save(app: &AppHandle, status: &RecipeStatus) -> Result<(), String> {
    let path = status_file(app, &status.app_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create status dir: {e}"))?;
    }
    let content = serde_json::to_string_pretty(status).map_err(|e| format!("failed to serialize status: {e}"))?;
    fs::write(path, content).map_err(|e| format!("failed to write status file: {e}"))
}

pub fn dashboard_url_for(app_id: &str) -> Option<String> {
    recipe_loader::load_recipe(app_id).ok().and_then(|recipe| recipe.dashboard_url())
}



pub fn mark_plan_accepted(
    app: &AppHandle,
    app_id: &str,
    plan_version: String,
    plan_digest: String,
    risk_level: String,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.plan_reviewed = true;
    status.plan_accepted_at = Some(Utc::now().to_rfc3339());
    status.plan_version = Some(plan_version);
    status.plan_digest = Some(plan_digest);
    status.plan_risk_level = Some(risk_level);
    status.last_error = None;
    save(app, &status)?;
    Ok(status)
}

pub fn clear_plan_acceptance(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.plan_reviewed = false;
    status.plan_accepted_at = None;
    status.plan_version = None;
    status.plan_digest = None;
    status.plan_risk_level = None;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_node_runtime(
    app: &AppHandle,
    app_id: &str,
    source: Option<String>,
    version: Option<String>,
    node_path: Option<String>,
    npm_path: Option<String>,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.node_runtime_source = source;
    status.node_runtime_version = version;
    status.node_runtime_path = node_path;
    status.npm_path = npm_path;
    if status.run_state == "unknown" {
        status.run_state = "stopped".to_string();
    }
    save(app, &status)?;
    Ok(status)
}


pub fn mark_uninstalled(app: &AppHandle, app_id: &str, note: Option<String>) -> Result<RecipeStatus, String> {
    let mut status = RecipeStatus::default_for(app_id);
    status.installed = false;
    status.install_state = "not_installed".to_string();
    status.run_state = "stopped".to_string();
    status.dashboard_url = dashboard_url_for(app_id);
    status.last_stopped_at = Some(Utc::now().to_rfc3339());
    status.last_error = note.map(|value| secret_redaction_registry::redact(&value));
    save(app, &status)?;
    Ok(status)
}

pub fn mark_installing(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.install_state = "installing".to_string();
    status.last_error = None;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_installed(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.installed = true;
    status.install_state = "installed".to_string();
    if status.run_state == "unknown" || status.run_state == "error" {
        status.run_state = "stopped".to_string();
    }
    status.dashboard_url = dashboard_url_for(app_id);
    status.last_error = None;
    status.pid = None;
    status.health_state = None;
    status.health_checked_at = None;
    status.readiness_state = None;
    status.readiness_checked_at = None;
    status.readiness_url = None;
    status.readiness_status_code = None;
    status.readiness_latency_ms = None;
    status.services.clear();
    save(app, &status)?;
    Ok(status)
}

pub fn mark_install_error(app: &AppHandle, app_id: &str, error: impl Into<String>) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.install_state = "error".to_string();
    status.last_error = Some(secret_redaction_registry::redact(&error.into()));
    save(app, &status)?;
    Ok(status)
}

pub fn mark_starting(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.run_state = "starting".to_string();
    status.last_error = None;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_running(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.installed = true;
    if status.install_state == "not_installed" || status.install_state == "error" {
        status.install_state = "installed".to_string();
    }
    status.run_state = "running".to_string();
    status.dashboard_url = dashboard_url_for(app_id);
    status.last_started_at = Some(Utc::now().to_rfc3339());
    status.last_error = None;
    if status.health_state.is_none() {
        status.health_state = Some("unknown".to_string());
    }
    if status.readiness_state.is_none() {
        status.readiness_state = Some("unknown".to_string());
    }
    status.services.clear();
    save(app, &status)?;
    Ok(status)
}

pub fn mark_running_with_pid(app: &AppHandle, app_id: &str, pid: u32) -> Result<RecipeStatus, String> {
    let mut status = mark_running(app, app_id)?;
    status.pid = Some(pid);
    save(app, &status)?;
    Ok(status)
}

pub fn mark_running_services(app: &AppHandle, app_id: &str, services: Vec<String>) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    let was_running = status.run_state == "running";
    status.installed = true;
    if status.install_state == "not_installed" || status.install_state == "error" {
        status.install_state = "installed".to_string();
    }
    status.run_state = "running".to_string();
    status.dashboard_url = dashboard_url_for(app_id);
    if !was_running || status.last_started_at.is_none() {
        status.last_started_at = Some(Utc::now().to_rfc3339());
    }
    status.last_error = None;
    status.pid = None;
    status.health_state = Some("healthy".to_string());
    status.health_checked_at = Some(Utc::now().to_rfc3339());
    if status.readiness_state.is_none() || status.readiness_state.as_deref() == Some("not_ready") {
        status.readiness_state = Some("checking".to_string());
    }
    status.services = services;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_stopping(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.run_state = "stopping".to_string();
    save(app, &status)?;
    Ok(status)
}

pub fn mark_stopped(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.run_state = "stopped".to_string();
    status.last_stopped_at = Some(Utc::now().to_rfc3339());
    status.last_error = None;
    status.pid = None;
    status.health_state = None;
    status.health_checked_at = None;
    status.readiness_state = None;
    status.readiness_checked_at = None;
    status.readiness_url = None;
    status.readiness_status_code = None;
    status.readiness_latency_ms = None;
    status.services.clear();
    save(app, &status)?;
    Ok(status)
}

pub fn mark_error(app: &AppHandle, app_id: &str, error: impl Into<String>) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.run_state = "error".to_string();
    status.last_error = Some(secret_redaction_registry::redact(&error.into()));
    status.pid = None;
    status.health_state = Some("unhealthy".to_string());
    status.health_checked_at = Some(Utc::now().to_rfc3339());
    status.readiness_state = Some("not_ready".to_string());
    status.readiness_checked_at = Some(Utc::now().to_rfc3339());
    status.services.clear();
    save(app, &status)?;
    Ok(status)
}


pub fn mark_healthy(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.health_state = Some("healthy".to_string());
    status.health_checked_at = Some(Utc::now().to_rfc3339());
    status.last_error = None;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_unhealthy(app: &AppHandle, app_id: &str, error: impl Into<String>) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.health_state = Some("unhealthy".to_string());
    status.health_checked_at = Some(Utc::now().to_rfc3339());
    status.last_error = Some(secret_redaction_registry::redact(&error.into()));
    status.readiness_state = Some("not_ready".to_string());
    status.readiness_checked_at = Some(Utc::now().to_rfc3339());
    if status.run_state == "running" || status.run_state == "starting" {
        status.run_state = "error".to_string();
    }
    save(app, &status)?;
    Ok(status)
}


pub fn mark_http_ready(
    app: &AppHandle,
    app_id: &str,
    url: &str,
    status_code: Option<u16>,
    latency_ms: Option<u128>,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.readiness_state = Some("ready".to_string());
    status.readiness_checked_at = Some(Utc::now().to_rfc3339());
    status.readiness_url = Some(url.to_string());
    status.readiness_status_code = status_code;
    status.readiness_latency_ms = latency_ms;
    status.last_error = None;
    save(app, &status)?;
    Ok(status)
}

pub fn mark_http_not_ready(
    app: &AppHandle,
    app_id: &str,
    url: &str,
    error: impl Into<String>,
    status_code: Option<u16>,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.readiness_state = Some("not_ready".to_string());
    status.readiness_checked_at = Some(Utc::now().to_rfc3339());
    status.readiness_url = Some(url.to_string());
    status.readiness_status_code = status_code;
    status.readiness_latency_ms = None;
    status.last_error = Some(secret_redaction_registry::redact(&error.into()));
    if matches!(status.run_state.as_str(), "running" | "starting") {
        status.run_state = "error".to_string();
    }
    save(app, &status)?;
    Ok(status)
}

pub fn mark_readiness_unknown(app: &AppHandle, app_id: &str, message: impl Into<String>) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.readiness_state = Some("unknown".to_string());
    status.readiness_checked_at = Some(Utc::now().to_rfc3339());
    status.readiness_status_code = None;
    status.readiness_latency_ms = None;
    status.last_error = Some(secret_redaction_registry::redact(&message.into()));
    save(app, &status)?;
    Ok(status)
}

pub fn mark_resource_preflight(
    app: &AppHandle,
    app_id: &str,
    state: impl Into<String>,
    blocking_count: u32,
    warning_count: u32,
    checked_at: String,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.resource_preflight_state = Some(state.into());
    status.resource_preflight_checked_at = Some(checked_at);
    status.resource_preflight_blocking_count = Some(blocking_count);
    status.resource_preflight_warning_count = Some(warning_count);
    if blocking_count == 0 {
        if status.last_error.as_deref().map(|value| value.contains("Resource preflight")).unwrap_or(false) {
            status.last_error = None;
        }
    } else {
        status.last_error = Some(format!("Resource preflight blocked installation/start: {blocking_count} required failure(s)."));
    }
    save(app, &status)?;
    Ok(status)
}


pub fn mark_port_resolution(
    app: &AppHandle,
    app_id: &str,
    state: impl Into<String>,
    checked_at: String,
    message: impl Into<String>,
    mappings: Vec<RecipePortMapping>,
    dashboard_url: Option<String>,
    readiness_url: Option<String>,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, app_id)?;
    status.port_resolution_state = Some(state.into());
    status.port_resolution_checked_at = Some(checked_at);
    status.port_resolution_message = Some(message.into());
    status.port_mappings = mappings;
    status.effective_dashboard_url = dashboard_url.clone();
    status.effective_readiness_url = readiness_url.clone();
    if let Some(url) = dashboard_url {
        status.dashboard_url = Some(url);
    }
    if let Some(url) = readiness_url {
        status.readiness_url = Some(url);
    }
    save(app, &status)?;
    Ok(status)
}

pub fn mark_progress_event(
    app: &AppHandle,
    event: &super::progress_events::RecipeProgressEvent,
) -> Result<RecipeStatus, String> {
    let mut status = load(app, &event.app_id)?;
    let is_new_operation = status.progress_operation_id.as_deref() != Some(event.operation_id.as_str());
    status.progress_state = Some(event.state.clone());
    status.progress_operation = Some(event.operation.clone());
    status.progress_operation_id = Some(event.operation_id.clone());
    status.progress_phase = Some(event.phase.clone());
    status.progress_message = Some(secret_redaction_registry::redact(&event.message));
    status.progress_percent = Some(event.percent);
    status.progress_step = Some(event.step);
    status.progress_total_steps = Some(event.total_steps);
    status.progress_updated_at = Some(event.timestamp.clone());
    if is_new_operation || status.progress_started_at.is_none() {
        status.progress_started_at = Some(event.timestamp.clone());
        status.progress_finished_at = None;
        status.progress_error = None;
    }
    match event.state.as_str() {
        "succeeded" => {
            status.progress_finished_at = Some(event.timestamp.clone());
            status.progress_error = None;
        }
        "failed" => {
            status.progress_finished_at = Some(event.timestamp.clone());
            status.progress_error = event.error.clone().map(|value| secret_redaction_registry::redact(&value));
        }
        _ => {}
    }
    save(app, &status)?;
    Ok(status)
}
