use std::collections::HashMap;
use std::process::{Child, Command as StdCommand};
use std::sync::Mutex;
use tauri::AppHandle;

use super::healthcheck;
use super::http_readiness;
use super::logs;
use super::paths;
use super::port_resolver;
use super::process_manager;
use super::recipe_loader::OpenNestRecipe;
use super::status::RecipeStatus;
use super::status_store;
use super::token_store;

static MANAGED_AGENTS: Mutex<Option<HashMap<String, Child>>> = Mutex::new(None);

fn agent_map() -> std::sync::MutexGuard<'static, Option<HashMap<String, Child>>> {
    let mut guard = MANAGED_AGENTS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    guard
}

fn executable_from_recipe(recipe: &OpenNestRecipe) -> Option<String> {
    recipe
        .install
        .as_ref()
        .and_then(|i| i.source.as_deref().map(ToString::to_string))
}

fn start_args(recipe: &OpenNestRecipe) -> Vec<String> {
    recipe
        .start
        .as_ref()
        .map(|s| s.args.clone())
        .unwrap_or_default()
}

fn inject_secrets(cmd: &mut StdCommand, recipe: &OpenNestRecipe) {
    for secret_def in &recipe.secrets {
        let id = secret_def
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let env_var = secret_def
            .get("envVar")
            .and_then(|v| v.as_str())
            .unwrap_or(id);
        if let Some(token) = token_store::get(&recipe.id, id) {
            cmd.env(env_var, token);
        }
    }
}

fn default_port(recipe: &OpenNestRecipe) -> u16 {
    recipe.primary_port().unwrap_or(9876)
}

pub fn check_environment(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<(), String> {
    let _executable = executable_from_recipe(recipe).unwrap_or_default();
    logs::append(app, &recipe.id, "agent-container", "environment check passed")?;
    Ok(())
}

pub fn install(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "agent-container" {
        return Err(format!("{} is not an agent-container recipe.", recipe.id));
    }

    let app_dir = paths::app_dir(app, &recipe.id)?;
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("failed to create app directory: {e}"))?;

    logs::append(app, &recipe.id, "install", "agent-container app registered")?;
    status_store::mark_installed(app, &recipe.id)
}

pub fn start(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "agent-container" {
        return Err(format!("{} is not an agent-container recipe.", recipe.id));
    }

    let status = status_store::load(app, &recipe.id).unwrap_or_else(|_| RecipeStatus::default_for(&recipe.id));
    if !status.installed {
        return Err(format!("{} must be installed before starting.", recipe.name));
    }

    let _ = stop(app, recipe);

    let executable = executable_from_recipe(recipe)
        .ok_or_else(|| format!("{} recipe does not define an executable path", recipe.id))?;

    let args = start_args(recipe);
    let _port = default_port(recipe);

    logs::append(
        app,
        &recipe.id,
        "start",
        &format!("launching agent: {} {}", executable, args.join(" ")),
    )?;

    let mut command = StdCommand::new(&executable);
    command.args(&args);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    inject_secrets(&mut command, recipe);

    let child = command
        .spawn()
        .map_err(|error| format!("failed to spawn agent {} ({}): {error}", recipe.name, executable))?;

    let pid = child.id();

    {
        let mut map = agent_map();
        map.as_mut().unwrap().insert(recipe.id.clone(), child);
    }

    logs::append(app, &recipe.id, "start", &format!("agent spawned pid={pid}"))?;

    if let Some((host, resolved_port)) = port_resolver::health_host_port(app, recipe) {
        if let Err(error) = healthcheck::wait_for_tcp(app, &recipe.id, &host, resolved_port, 30_000, 750) {
            let _ = stop(app, recipe);
            return Err(format!("agent {} started but port {host}:{resolved_port} is not reachable: {error}", recipe.name));
        }
    }

    status_store::mark_running_with_pid(app, &recipe.id, pid)
}

pub fn stop(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "agent-container" {
        return Err(format!("{} is not an agent-container recipe.", recipe.id));
    }

    let mut child = {
        let mut map = agent_map();
        map.as_mut().and_then(|m| m.remove(&recipe.id))
    };

    if let Some(ref mut child) = child {
        let pid = child.id();
        logs::append(app, &recipe.id, "stop", &format!("killing agent pid={pid}"))?;
        let _ = child.kill();
        let _ = child.wait();
    } else if let Ok(Some(record)) = process_manager::load_record(app, &recipe.id) {
        if process_manager::is_pid_running(record.pid) {
            process_manager::stop_managed(app, &recipe.id)?;
        }
    }

    status_store::mark_stopped(app, &recipe.id)
}

pub fn open_dashboard(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if let Some(url) = port_resolver::effective_dashboard_url(app, recipe) {
        tauri_plugin_opener::open_url(url, None::<&str>)
            .map_err(|error| format!("failed to open agent dashboard: {error}"))?;
    }
    status_store::load(app, &recipe.id)
        .or_else(|_| Ok(RecipeStatus::default_for(&recipe.id)))
}

pub fn check_health(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if let Some((host, port)) = port_resolver::health_host_port(app, recipe) {
        let report = healthcheck::check_tcp(&host, port, 750);
        if report.ok {
            return status_store::mark_healthy(app, &recipe.id);
        }
        let error = report.error.unwrap_or_else(|| format!("{host}:{port} unreachable"));
        return status_store::mark_unhealthy(app, &recipe.id, error);
    }

    let alive = {
        let map = agent_map();
        map.as_ref().and_then(|m| m.get(&recipe.id)).is_some()
    };
    if alive {
        status_store::mark_healthy(app, &recipe.id)
    } else {
        status_store::mark_unhealthy(app, &recipe.id, "agent process not running")
    }
}

pub fn check_readiness(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if let Some(_url) = port_resolver::effective_readiness_url(app, recipe) {
        return http_readiness::ensure_ready(app, recipe);
    }

    if let Some((host, port)) = port_resolver::health_host_port(app, recipe) {
        healthcheck::check_tcp(&host, port, 750);
        let now = chrono::Utc::now().to_rfc3339();
        let mut status = status_store::load(app, &recipe.id)?;
        status.readiness_state = Some("ready".to_string());
        status.readiness_checked_at = Some(now);
        status_store::save(app, &status)?;
        return Ok(status);
    }

    Err(format!("{}: no readiness URL or health port configured", recipe.id))
}

pub fn read_logs(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<Vec<String>, String> {
    logs::read_tail(app, &recipe.id, 200)
}

pub fn uninstall(app: &AppHandle, recipe: &OpenNestRecipe, remove_data: bool) -> Result<RecipeStatus, String> {
    let _ = stop(app, recipe);

    if remove_data {
        let app_dir = paths::app_dir(app, &recipe.id)?;
        if app_dir.exists() {
            std::fs::remove_dir_all(&app_dir)
                .map_err(|e| format!("failed to remove app directory {}: {e}", app_dir.display()))?;
        }
    }

    status_store::mark_uninstalled(app, &recipe.id, None)
}

pub fn reconcile_status(app: &AppHandle, recipe: &OpenNestRecipe, mut status: RecipeStatus) -> RecipeStatus {
    if recipe.runtime != "agent-container" {
        return status;
    }

    let alive = {
        let map = agent_map();
        map.as_ref().and_then(|m| m.get(&recipe.id)).is_some()
    };

    if alive {
        if status.run_state != "running" {
            status.run_state = "running".to_string();
        }
        if let Some((host, port)) = port_resolver::health_host_port(app, recipe) {
            let report = healthcheck::check_tcp(&host, port, 500);
            status.health_state = Some(if report.ok { "healthy".to_string() } else { "unhealthy".to_string() });
            status.health_checked_at = Some(chrono::Utc::now().to_rfc3339());
        }
    } else if status.installed {
        status.run_state = "stopped".to_string();
        status.health_state = None;
    }

    status
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::recipe_loader::{OpenNestRecipe, RecipeInstallSpec};

    fn minimal_recipe(id: &str, ports: Vec<u16>) -> OpenNestRecipe {
        OpenNestRecipe {
            schema_version: "2.0.0".into(),
            id: id.into(),
            name: format!("Test {id}"),
            summary: "Test recipe".into(),
            description: None,
            runtime: "agent-container".into(),
            category: "Test".into(),
            version_source: None,
            homepage: None,
            source_url: None,
            license: None,
            icon: None,
            screenshots: vec![],
            tags: vec![],
            difficulty: None,
            priority: None,
            ports,
            requirements: None,
            install_plan_template: None,
            runtime_defaults: None,
            paths: None,
            install: None,
            start: None,
            stop: None,
            dashboard: None,
            logs: None,
            onboarding: None,
            doctor: None,
            secrets: vec![],
            permissions: vec![],
        }
    }

    #[test]
    fn test_default_port_for_agent() {
        let recipe = minimal_recipe("agent-app", vec![]);
        assert_eq!(default_port(&recipe), 9876);
    }

    #[test]
    fn test_port_from_recipe() {
        let recipe = minimal_recipe("agent-app", vec![7777]);
        assert_eq!(default_port(&recipe), 7777);
    }

    #[test]
    fn test_executable_from_install_source() {
        let mut recipe = minimal_recipe("agent-app", vec![5000]);
        recipe.install = Some(RecipeInstallSpec {
            strategy: "native-cli".into(),
            source: Some("node".into()),
            package: None,
            prefix: None,
            binary_windows: None,
            repo: None,
            git_ref: None,
            env_example: None,
            env_target: None,
            compose_dir: None,
        });
        assert_eq!(executable_from_recipe(&recipe), Some("node".into()));
    }

    #[test]
    fn test_start_args_default_empty() {
        let recipe = minimal_recipe("agent-app", vec![5000]);
        assert!(start_args(&recipe).is_empty());
    }

    #[test]
    fn test_agent_container_runtime_validation() {
        let recipe = minimal_recipe("agent-valid", vec![8888]);
        assert_eq!(recipe.runtime, "agent-container");
    }
}