use std::fs;
use std::path::PathBuf;
use std::process::Command;

use tauri::AppHandle;

use super::command_timeout;
use super::logs;
use super::http_readiness;
use super::paths;
use super::recipe_loader::OpenNestRecipe;
use super::status::RecipeStatus;
use super::status_store;

const DEFAULT_REF: &str = "main";

fn app_source_dir(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, app_id)?.join("source"))
}

fn compose_dir_from_recipe(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<PathBuf, String> {
    let source = app_source_dir(app, &recipe.id)?;
    let compose_subdir = recipe
        .install
        .as_ref()
        .and_then(|install| install.compose_dir.as_deref())
        .unwrap_or("docker");
    Ok(source.join(compose_subdir))
}

fn compose_file_path(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<PathBuf, String> {
    let dir = compose_dir_from_recipe(app, recipe)?;
    let yaml = dir.join("docker-compose.yaml");
    if yaml.exists() {
        return Ok(yaml);
    }
    let yml = dir.join("docker-compose.yml");
    if yml.exists() {
        return Ok(yml);
    }
    Err(format!(
        "No docker-compose.yaml or docker-compose.yml found in {}. Official source sync may be incomplete.",
        dir.display()
    ))
}

fn run_in_compose_dir(
    app: &AppHandle,
    recipe: &OpenNestRecipe,
    category: &str,
    action: &str,
    args: &[&str],
    timeout_ms: u64,
) -> Result<command_timeout::TimedCommandOutput, String> {
    let compose_dir = compose_dir_from_recipe(app, recipe)?;
    if !compose_dir.exists() {
        return Err(format!(
            "External compose directory does not exist: {}. Install/sync {} first.",
            compose_dir.display(),
            recipe.name
        ));
    }

    let mut command = Command::new("docker");
    command.current_dir(&compose_dir);
    command.arg("compose");
    for arg in args {
        command.arg(arg);
    }

    let output = command_timeout::run_with_timeout(command, timeout_ms)
        .map_err(|error| format!("{action} failed to start or wait: {error}"))?;

    logs::append(
        app,
        &recipe.id,
        category,
        &format!(
            "{action} finished working_dir={} duration_ms={} timed_out={} exit_code={:?}",
            compose_dir.display(),
            output.duration_ms,
            output.timed_out,
            output.exit_code
        ),
    )?;
    logs::append(app, &recipe.id, category, &output.stdout)?;
    logs::append(app, &recipe.id, category, &output.stderr)?;

    Ok(output)
}

fn ensure_git_available(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let mut command = Command::new("git");
    command.arg("--version");
    let output = command_timeout::run_with_timeout(command, command_timeout::CHECK_TIMEOUT_MS)
        .map_err(|error| format!("git --version failed to start or wait: {error}"))?;
    logs::append(app, app_id, "external-compose", &format!("git --version: {}{}", output.stdout, output.stderr))?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("git --version"))
    }
}

fn install_spec_value<'a>(recipe: &'a OpenNestRecipe, field: &str) -> Option<&'a str> {
    recipe.install.as_ref().and_then(|install| match field {
        "repo" => install.repo.as_deref(),
        "ref" => install.git_ref.as_deref(),
        "envExample" => install.env_example.as_deref(),
        "envTarget" => install.env_target.as_deref(),
        _ => None,
    })
}

pub fn check_environment(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<(), String> {
    super::docker_compose::check_docker(app, &recipe.id)?;
    ensure_git_available(app, &recipe.id)?;
    Ok(())
}

pub fn install(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "external-compose" {
        return Err(format!("{} is not an external-compose recipe.", recipe.id));
    }

    check_environment(app, recipe)?;

    let repo = install_spec_value(recipe, "repo")
        .ok_or_else(|| format!("Recipe {} install.repo is required for external-compose.", recipe.id))?;
    let git_ref = install_spec_value(recipe, "ref").unwrap_or(DEFAULT_REF);
    let source_dir = app_source_dir(app, &recipe.id)?;

    if source_dir.join(".git").exists() {
        logs::append(app, &recipe.id, "install", &format!("updating official source: {}", source_dir.display()))?;
        let mut fetch = Command::new("git");
        fetch.current_dir(&source_dir).args(["fetch", "--depth", "1", "origin", git_ref]);
        let fetch_output = command_timeout::run_with_timeout(fetch, command_timeout::INSTALL_TIMEOUT_MS)
            .map_err(|error| format!("git fetch failed to start or wait: {error}"))?;
        logs::append(app, &recipe.id, "install", &fetch_output.stdout)?;
        logs::append(app, &recipe.id, "install", &fetch_output.stderr)?;
        if !fetch_output.success {
            return Err(fetch_output.failure_message("git fetch official source"));
        }

        let mut checkout = Command::new("git");
        checkout.current_dir(&source_dir).args(["checkout", "FETCH_HEAD"]);
        let checkout_output = command_timeout::run_with_timeout(checkout, command_timeout::CHECK_TIMEOUT_MS)
            .map_err(|error| format!("git checkout failed to start or wait: {error}"))?;
        logs::append(app, &recipe.id, "install", &checkout_output.stdout)?;
        logs::append(app, &recipe.id, "install", &checkout_output.stderr)?;
        if !checkout_output.success {
            return Err(checkout_output.failure_message("git checkout official source"));
        }
    } else {
        if source_dir.exists() {
            return Err(format!(
                "Source directory exists but is not a git repository: {}. Move it aside or use Repair/Reinstall after cleanup.",
                source_dir.display()
            ));
        }
        if let Some(parent) = source_dir.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("failed to create source parent dir: {error}"))?;
        }
        logs::append(app, &recipe.id, "install", &format!("cloning official source repo={repo} ref={git_ref} into {}", source_dir.display()))?;
        let mut clone = Command::new("git");
        clone
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(git_ref)
            .arg(repo)
            .arg(&source_dir);
        let clone_output = command_timeout::run_with_timeout(clone, command_timeout::INSTALL_TIMEOUT_MS)
            .map_err(|error| format!("git clone failed to start or wait: {error}"))?;
        logs::append(app, &recipe.id, "install", &clone_output.stdout)?;
        logs::append(app, &recipe.id, "install", &clone_output.stderr)?;
        if !clone_output.success {
            return Err(clone_output.failure_message("git clone official source"));
        }
    }

    initialize_env(app, recipe)?;
    let compose_file = compose_file_path(app, recipe)?;
    logs::append(app, &recipe.id, "install", &format!("external compose ready: {}", compose_file.display()))?;

    status_store::mark_installed(app, &recipe.id)
}

pub fn initialize_env(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<(), String> {
    let compose_dir = compose_dir_from_recipe(app, recipe)?;
    if !compose_dir.exists() {
        return Err(format!("Compose directory is missing: {}", compose_dir.display()));
    }

    let target_name = install_spec_value(recipe, "envTarget").unwrap_or(".env");
    let target = compose_dir.join(target_name);
    if target.exists() {
        logs::append(app, &recipe.id, "install", &format!("env already exists: {}", target.display()))?;
        return Ok(());
    }

    let configured_example = install_spec_value(recipe, "envExample").unwrap_or(".env.example");
    let candidates = [configured_example, ".env.example", ".env.default"];
    let mut copied_from: Option<PathBuf> = None;
    for candidate in candidates {
        let source = compose_dir.join(candidate);
        if source.exists() {
            fs::copy(&source, &target).map_err(|error| {
                format!("failed to copy {} to {}: {error}", source.display(), target.display())
            })?;
            copied_from = Some(source);
            break;
        }
    }

    match copied_from {
        Some(source) => logs::append(
            app,
            &recipe.id,
            "install",
            &format!("initialized env file: {} from {}", target.display(), source.display()),
        ),
        None => {
            let message = format!(
                "No env template found in {}. Some upstream versions create .env automatically; otherwise edit upstream docker env manually before start.",
                compose_dir.display()
            );
            logs::append(app, &recipe.id, "install", &message)
        }
    }
}

pub fn start(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let _ = compose_file_path(app, recipe)?;
    let output = run_in_compose_dir(
        app,
        recipe,
        "start",
        "docker compose up",
        &["up", "-d"],
        command_timeout::COMPOSE_UP_TIMEOUT_MS,
    )?;
    if !output.success {
        return Err(output.failure_message("docker compose up"));
    }
    ensure_running(app, recipe)
}

pub fn stop(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let output = run_in_compose_dir(
        app,
        recipe,
        "stop",
        "docker compose stop",
        &["stop"],
        command_timeout::STOP_TIMEOUT_MS,
    )?;
    if !output.success {
        return Err(output.failure_message("docker compose stop"));
    }

    let services = running_services(app, recipe).unwrap_or_default();
    if services.is_empty() {
        status_store::mark_stopped(app, &recipe.id)
    } else {
        let message = format!(
            "{} stop command completed, but these services are still running: {}",
            recipe.name,
            services.join(", ")
        );
        let _ = status_store::mark_running_services(app, &recipe.id, services)?;
        let _ = status_store::mark_unhealthy(app, &recipe.id, message.clone());
        Err(message)
    }
}

pub fn running_services(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<Vec<String>, String> {
    let _ = compose_file_path(app, recipe)?;
    let output = run_in_compose_dir(
        app,
        recipe,
        "docker-status",
        "docker compose ps --status running --services",
        &["ps", "--status", "running", "--services"],
        command_timeout::CHECK_TIMEOUT_MS,
    )?;
    if !output.success {
        return Err(output.failure_message("docker compose ps --status running --services"));
    }
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

pub fn ensure_running(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let services = running_services(app, recipe)?;
    if services.is_empty() {
        let error = format!("{} compose command completed, but docker compose ps reports no running services.", recipe.name);
        let _ = status_store::mark_unhealthy(app, &recipe.id, error.clone());
        return Err(error);
    }

    let _running = status_store::mark_running_services(app, &recipe.id, services)?;
    match http_readiness::ensure_ready(app, recipe) {
        Ok(ready_status) => Ok(ready_status),
        Err(error) => Err(format!("{} services are running, but dashboard is not HTTP-ready: {error}", recipe.name)),
    }
}

pub fn reconcile_status(app: &AppHandle, recipe: &OpenNestRecipe, status: RecipeStatus) -> RecipeStatus {
    if recipe.runtime != "external-compose" || !status.installed {
        return status;
    }

    match running_services(app, recipe) {
        Ok(services) if !services.is_empty() => {
            let running = status_store::mark_running_services(app, &recipe.id, services)
                .unwrap_or_else(|_| status);
            http_readiness::check_once(app, recipe, running)
        },
        Ok(_) => {
            if matches!(status.run_state.as_str(), "running" | "starting" | "stopping") {
                status_store::mark_stopped(app, &recipe.id).unwrap_or(status)
            } else {
                status
            }
        }
        Err(error) => {
            if matches!(status.run_state.as_str(), "running" | "starting" | "stopping") {
                status_store::mark_error(app, &recipe.id, format!("External compose status check failed: {error}"))
                    .unwrap_or(status)
            } else {
                status
            }
        }
    }
}

pub fn logs(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<Vec<String>, String> {
    let output = run_in_compose_dir(
        app,
        recipe,
        "logs",
        "docker compose logs",
        &["logs", "--tail", "200"],
        command_timeout::LOGS_TIMEOUT_MS,
    )?;
    if output.timed_out {
        return Err(output.failure_message("docker compose logs"));
    }
    let text = format!("{}{}", output.stdout, output.stderr);
    Ok(text.lines().map(logs::redact).collect())
}


pub fn down(app: &AppHandle, recipe: &OpenNestRecipe, remove_volumes: bool) -> Result<(), String> {
    let args: Vec<&str> = if remove_volumes {
        vec!["down", "--remove-orphans", "-v"]
    } else {
        vec!["down", "--remove-orphans"]
    };
    let output = run_in_compose_dir(
        app,
        recipe,
        "uninstall",
        "docker compose down",
        &args,
        command_timeout::STOP_TIMEOUT_MS,
    )?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose down"))
    }
}
