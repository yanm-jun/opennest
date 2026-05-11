use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

use super::healthcheck;
use super::http_readiness;
use super::logs;
use super::paths;
use super::recipe_loader::OpenNestRecipe;
use super::status::{RecipePortMapping, RecipeStatus};
use super::status_store;

const PORT_SCAN_LIMIT: u16 = 150;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortResolutionResult {
    pub app_id: String,
    pub checked_at: String,
    pub ok: bool,
    pub state: String,
    pub message: String,
    pub mappings: Vec<RecipePortMapping>,
    pub dashboard_url: Option<String>,
    pub readiness_url: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortOverrideFile {
    app_id: String,
    generated_at: String,
    mappings: Vec<RecipePortMapping>,
}

fn port_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, app_id)?.join("port-overrides.json"))
}

fn load_override_file(app: &AppHandle, app_id: &str) -> Result<Option<PortOverrideFile>, String> {
    let path = port_file(app, app_id)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|error| format!("failed to read port-overrides.json: {error}"))?;
    let data = serde_json::from_str::<PortOverrideFile>(&text)
        .map_err(|error| format!("port-overrides.json is corrupted: {error}"))?;
    Ok(Some(data))
}

fn save_override_file(app: &AppHandle, app_id: &str, mappings: &[RecipePortMapping]) -> Result<(), String> {
    let path = port_file(app, app_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("failed to create port override dir: {error}"))?;
    }
    let data = PortOverrideFile {
        app_id: app_id.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        mappings: mappings.to_vec(),
    };
    let text = serde_json::to_string_pretty(&data).map_err(|error| format!("failed to serialize port override: {error}"))?;
    fs::write(path, text).map_err(|error| format!("failed to write port-overrides.json: {error}"))
}

pub fn clear_overrides(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let path = port_file(app, app_id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("failed to remove port-overrides.json: {error}"))?;
    }
    Ok(())
}

fn is_local_port_free(port: u16) -> bool {
    !healthcheck::check_tcp("127.0.0.1", port, 450).ok
}

fn find_available_port(start: u16) -> Option<u16> {
    let mut candidate = start.saturating_add(1);
    for _ in 0..PORT_SCAN_LIMIT {
        if candidate == 0 {
            return None;
        }
        if is_local_port_free(candidate) {
            return Some(candidate);
        }
        candidate = candidate.saturating_add(1);
    }
    None
}

pub fn load_mappings(app: &AppHandle, recipe: &OpenNestRecipe) -> Vec<RecipePortMapping> {
    match load_override_file(app, &recipe.id) {
        Ok(Some(file)) if file.app_id == recipe.id => file.mappings,
        _ => recipe.ports.iter().map(|port| RecipePortMapping {
            host: "127.0.0.1".to_string(),
            requested_port: *port,
            resolved_port: *port,
            changed: false,
        }).collect(),
    }
}

pub fn effective_port(app: &AppHandle, recipe: &OpenNestRecipe, requested_port: u16) -> u16 {
    load_mappings(app, recipe)
        .into_iter()
        .find(|mapping| mapping.requested_port == requested_port)
        .map(|mapping| mapping.resolved_port)
        .unwrap_or(requested_port)
}

pub fn effective_primary_port(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<u16> {
    recipe.primary_port().map(|port| effective_port(app, recipe, port))
}

fn parse_local_url(url: &str) -> Option<(String, String, u16, String)> {
    let (scheme, rest) = if let Some(rest) = url.strip_prefix("http://") {
        ("http".to_string(), rest)
    } else if let Some(rest) = url.strip_prefix("https://") {
        ("https".to_string(), rest)
    } else {
        return None;
    };

    let (host_port, path) = match rest.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (rest, "/".to_string()),
    };
    let default_port = if scheme == "https" { 443 } else { 80 };
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port_text)) if !host.contains(']') => {
            let port = port_text.parse::<u16>().ok()?;
            (host.to_string(), port)
        }
        _ => (host_port.to_string(), default_port),
    };
    let normalized_host = match host.as_str() {
        "localhost" => "127.0.0.1".to_string(),
        _ => host,
    };
    Some((scheme, normalized_host, port, path))
}

fn build_local_url(scheme: &str, host: &str, port: u16, path: &str) -> String {
    let default_port = if scheme == "https" { 443 } else { 80 };
    if port == default_port {
        format!("{scheme}://{host}{path}")
    } else {
        format!("{scheme}://{host}:{port}{path}")
    }
}

pub fn rewrite_local_url(app: &AppHandle, recipe: &OpenNestRecipe, url: &str) -> String {
    let Some((scheme, host, requested_port, path)) = parse_local_url(url) else {
        return url.to_string();
    };
    if host != "127.0.0.1" && host != "0.0.0.0" {
        return url.to_string();
    }
    let resolved = effective_port(app, recipe, requested_port);
    build_local_url(&scheme, "127.0.0.1", resolved, &path)
}

pub fn effective_dashboard_url(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<String> {
    recipe.dashboard_url().map(|url| rewrite_local_url(app, recipe, &url))
}

pub fn effective_readiness_url(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<String> {
    recipe
        .start
        .as_ref()
        .and_then(|start| start.healthcheck.clone())
        .or_else(|| recipe.dashboard_url())
        .map(|url| rewrite_local_url(app, recipe, &url))
}

pub fn health_host_port(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<(String, u16)> {
    if let Some(url) = effective_readiness_url(app, recipe) {
        if let Ok(parsed) = http_readiness::parse_readiness_url(&url) {
            return Some((parsed.host, parsed.port));
        }
    }
    recipe.primary_port().map(|requested| ("127.0.0.1".to_string(), effective_port(app, recipe, requested)))
}

pub fn resolve_ports(app: &AppHandle, recipe: &OpenNestRecipe, apply: bool) -> Result<PortResolutionResult, String> {
    let checked_at = Utc::now().to_rfc3339();
    if recipe.ports.is_empty() {
        let result = PortResolutionResult {
            app_id: recipe.id.clone(),
            checked_at,
            ok: true,
            state: "not_required".to_string(),
            message: "Recipe does not declare local ports.".to_string(),
            mappings: Vec::new(),
            dashboard_url: effective_dashboard_url(app, recipe),
            readiness_url: effective_readiness_url(app, recipe),
            warnings: Vec::new(),
        };
        return Ok(result);
    }

    let current_status = status_store::load(app, &recipe.id).unwrap_or_else(|_| RecipeStatus::default_for(&recipe.id));
    let existing_mappings = load_mappings(app, recipe);
    let mut mappings = Vec::new();
    let mut warnings = Vec::new();
    let mut ok = true;

    for requested in &recipe.ports {
        if let Some(existing) = existing_mappings.iter().find(|mapping| mapping.requested_port == *requested && mapping.changed) {
            if is_local_port_free(existing.resolved_port) || current_status.run_state == "running" || current_status.readiness_state.as_deref() == Some("ready") {
                mappings.push(existing.clone());
                warnings.push(format!(
                    "Existing port override retained for {}: {} → {}.",
                    recipe.name, existing.requested_port, existing.resolved_port
                ));
                continue;
            }
            warnings.push(format!(
                "Existing port override {} → {} is no longer available; OpenNest will search again.",
                existing.requested_port, existing.resolved_port
            ));
        }

        let occupied = !is_local_port_free(*requested);
        if !occupied || current_status.run_state == "running" || current_status.readiness_state.as_deref() == Some("ready") {
            mappings.push(RecipePortMapping {
                host: "127.0.0.1".to_string(),
                requested_port: *requested,
                resolved_port: *requested,
                changed: false,
            });
            if occupied {
                warnings.push(format!("Port {requested} is reachable, but {} appears to be running already; keeping the configured port.", recipe.name));
            }
            continue;
        }

        if recipe.runtime == "external-compose" {
            ok = false;
            warnings.push(format!(
                "Port {requested} is occupied. {} uses an upstream external Compose stack, so OpenNest will not rewrite its official compose/env automatically in this MVP.",
                recipe.name
            ));
            mappings.push(RecipePortMapping {
                host: "127.0.0.1".to_string(),
                requested_port: *requested,
                resolved_port: *requested,
                changed: false,
            });
            continue;
        }

        match find_available_port(*requested) {
            Some(resolved) => {
                warnings.push(format!("Port {requested} is occupied. OpenNest will use {resolved} for {}.", recipe.name));
                mappings.push(RecipePortMapping {
                    host: "127.0.0.1".to_string(),
                    requested_port: *requested,
                    resolved_port: resolved,
                    changed: true,
                });
            }
            None => {
                ok = false;
                warnings.push(format!("Port {requested} is occupied and no available port was found in the next {PORT_SCAN_LIMIT} ports."));
                mappings.push(RecipePortMapping {
                    host: "127.0.0.1".to_string(),
                    requested_port: *requested,
                    resolved_port: *requested,
                    changed: false,
                });
            }
        }
    }

    if apply && ok {
        save_override_file(app, &recipe.id, &mappings)?;
    }

    let changed_count = mappings.iter().filter(|mapping| mapping.changed).count();
    let state = if !ok {
        "blocked".to_string()
    } else if changed_count > 0 {
        "resolved".to_string()
    } else {
        "unchanged".to_string()
    };
    let message = if !ok {
        "Port resolution failed; at least one required port remains blocked.".to_string()
    } else if changed_count > 0 {
        format!("Port resolution completed with {changed_count} remapped port(s).")
    } else {
        "All declared ports are available as configured.".to_string()
    };

    let result = PortResolutionResult {
        app_id: recipe.id.clone(),
        checked_at: checked_at.clone(),
        ok,
        state: state.clone(),
        message: message.clone(),
        mappings: mappings.clone(),
        dashboard_url: effective_dashboard_url(app, recipe),
        readiness_url: effective_readiness_url(app, recipe),
        warnings,
    };

    let _ = status_store::mark_port_resolution(
        app,
        &recipe.id,
        state,
        checked_at,
        message,
        mappings,
        result.dashboard_url.clone(),
        result.readiness_url.clone(),
    );

    Ok(result)
}

pub fn ensure_ports(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<PortResolutionResult, String> {
    let result = resolve_ports(app, recipe, true)?;
    logs::append(app, &recipe.id, "ports", &format!("{} dashboard={:?} readiness={:?}", result.message, result.dashboard_url, result.readiness_url))?;
    for mapping in &result.mappings {
        logs::append(app, &recipe.id, "ports", &format!("port mapping requested={} resolved={} changed={}", mapping.requested_port, mapping.resolved_port, mapping.changed))?;
    }
    if result.ok {
        Ok(result)
    } else {
        Err(result.message)
    }
}

pub fn rewrite_compose_content(app: &AppHandle, recipe: &OpenNestRecipe, compose_content: &str) -> Result<String, String> {
    let result = ensure_ports(app, recipe)?;
    let mut content = compose_content.to_string();
    for mapping in result.mappings.iter().filter(|mapping| mapping.changed) {
        let from_double = format!("\"{}:", mapping.requested_port);
        let to_double = format!("\"{}:", mapping.resolved_port);
        let from_single = format!("'{}:", mapping.requested_port);
        let to_single = format!("'{}:", mapping.resolved_port);
        let from_plain = format!("- {}:", mapping.requested_port);
        let to_plain = format!("- {}:", mapping.resolved_port);
        content = content
            .replace(&from_double, &to_double)
            .replace(&from_single, &to_single)
            .replace(&from_plain, &to_plain);
    }
    Ok(content)
}
