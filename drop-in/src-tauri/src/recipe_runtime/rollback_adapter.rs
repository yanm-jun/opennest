use std::fs;
use std::path::{Path, PathBuf};

use tauri::AppHandle;

use super::{docker_compose, external_compose, logs, native_cli, paths, process_manager, recipe_loader::OpenNestRecipe, status::RecipeStatus, status_store, token_store};

#[derive(Debug, Clone, Copy)]
pub enum UninstallMode {
    KeepData,
    RemoveData,
}

impl UninstallMode {
    pub fn remove_data(self) -> bool {
        matches!(self, Self::RemoveData)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::KeepData => "keep-data",
            Self::RemoveData => "remove-data",
        }
    }
}

fn app_dir(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    paths::app_dir(app, app_id)
}

fn ensure_safe_app_path(app: &AppHandle, app_id: &str, path: &Path) -> Result<(), String> {
    let root = paths::root_dir(app)?.canonicalize().map_err(|error| format!("failed to canonicalize OpenNest apps root: {error}"))?;
    let target = if path.exists() {
        path.canonicalize().map_err(|error| format!("failed to canonicalize target path {}: {error}", path.display()))?
    } else {
        path.to_path_buf()
    };

    // Non-existing children cannot be canonicalized; validate their parent chain instead.
    let comparable = if target.exists() {
        target
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };
    let comparable = if comparable.exists() {
        comparable.canonicalize().map_err(|error| format!("failed to canonicalize parent path {}: {error}", comparable.display()))?
    } else {
        comparable
    };

    if comparable.starts_with(&root) {
        Ok(())
    } else {
        Err(format!(
            "refusing to remove path outside OpenNest apps root. app_id={} root={} target={}",
            app_id,
            root.display(),
            path.display()
        ))
    }
}

fn remove_dir_if_exists(app: &AppHandle, app_id: &str, path: &Path, reason: &str) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    ensure_safe_app_path(app, app_id, path)?;
    logs::append(app, app_id, "cleanup", &format!("removing directory for {reason}: {}", path.display()))?;
    fs::remove_dir_all(path).map_err(|error| format!("failed to remove {}: {error}", path.display()))
}

fn remove_file_if_exists(app: &AppHandle, app_id: &str, path: &Path, reason: &str) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    ensure_safe_app_path(app, app_id, path)?;
    logs::append(app, app_id, "cleanup", &format!("removing file for {reason}: {}", path.display()))?;
    fs::remove_file(path).map_err(|error| format!("failed to remove {}: {error}", path.display()))
}

fn best_effort_stop(app: &AppHandle, recipe: &OpenNestRecipe, remove_volumes: bool) -> Result<(), String> {
    match recipe.runtime.as_str() {
        "native-cli" => {
            let message = native_cli::stop_app(app, &recipe.id).unwrap_or_else(|error| format!("best-effort stop failed: {error}"));
            logs::append(app, &recipe.id, "cleanup", &message)?;
        }
        "docker-compose" => {
            match docker_compose::compose_down(app, &recipe.id, remove_volumes) {
                Ok(()) => logs::append(app, &recipe.id, "cleanup", "docker compose down completed")?,
                Err(error) => logs::append(app, &recipe.id, "cleanup", &format!("docker compose down best-effort failed: {error}"))?,
            }
        }
        "external-compose" => {
            match external_compose::down(app, recipe, remove_volumes) {
                Ok(()) => logs::append(app, &recipe.id, "cleanup", "external docker compose down completed")?,
                Err(error) => logs::append(app, &recipe.id, "cleanup", &format!("external docker compose down best-effort failed: {error}"))?,
            }
        }
        _ => {}
    }
    Ok(())
}

fn cleanup_runtime_artifacts(app: &AppHandle, recipe: &OpenNestRecipe, remove_data: bool) -> Result<(), String> {
    let dir = app_dir(app, &recipe.id)?;
    match recipe.runtime.as_str() {
        "native-cli" => {
            process_manager::clear_record(app, &recipe.id).ok();
            if remove_data {
                remove_dir_if_exists(app, &recipe.id, &dir, &format!("full {} uninstall", recipe.name))?;
            } else {
                remove_dir_if_exists(app, &recipe.id, &dir.join("cli"), &format!("{} CLI uninstall", recipe.name))?;
                // Keep state/config/logs so the user can reinstall without losing local setup evidence.
            }
        }
        "docker-compose" => {
            if remove_data {
                remove_dir_if_exists(app, &recipe.id, &dir, "full docker-compose app uninstall")?;
            } else {
                // Keep compose file, bind-mounted data, and logs. Containers/networks were removed by docker compose down.
                logs::append(app, &recipe.id, "cleanup", "kept app workspace and compose file because keep-data mode was selected")?;
            }
        }
        "external-compose" => {
            if remove_data {
                remove_dir_if_exists(app, &recipe.id, &dir, "full external-compose app uninstall")?;
            } else {
                logs::append(app, &recipe.id, "cleanup", "kept official source, .env, and logs because keep-data mode was selected")?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn clear_known_secrets_if_requested(recipe: &OpenNestRecipe, remove_data: bool) -> Result<(), String> {
    if !remove_data {
        return Ok(());
    }
    let secret_ids: Vec<String> = recipe
        .secrets
        .iter()
        .filter_map(|secret| secret.get("id").and_then(|value| value.as_str()).map(ToOwned::to_owned))
        .collect();
    token_store::delete_many(&recipe.id, &secret_ids)
}

pub fn rollback_failed_install(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let status = status_store::load(app, &recipe.id)?;
    if status.installed && status.install_state == "installed" {
        return Err(format!(
            "{} is already installed. Rollback is only for failed or partial installs; use Uninstall instead.",
            recipe.name
        ));
    }

    logs::append(app, &recipe.id, "rollback", "starting failed-install rollback")?;
    best_effort_stop(app, recipe, false)?;

    let dir = app_dir(app, &recipe.id)?;
    match recipe.runtime.as_str() {
        "native-cli" => {
            process_manager::clear_record(app, &recipe.id).ok();
            remove_dir_if_exists(app, &recipe.id, &dir.join("cli"), &format!("rollback {} CLI partial install", recipe.name))?;
        }
        "docker-compose" => {
            remove_file_if_exists(app, &recipe.id, &paths::compose_file(app, &recipe.id)?, "rollback generated compose file")?;
        }
        "external-compose" => {
            remove_dir_if_exists(app, &recipe.id, &dir.join("source"), "rollback external official source checkout")?;
        }
        _ => {}
    }

    let status = status_store::mark_uninstalled(app, &recipe.id, Some("Rollback completed. Partial install artifacts were removed; logs/status were preserved.".to_string()))?;
    logs::append(app, &recipe.id, "rollback", "failed-install rollback completed")?;
    Ok(status)
}

pub fn uninstall(app: &AppHandle, recipe: &OpenNestRecipe, mode: UninstallMode) -> Result<RecipeStatus, String> {
    logs::append(app, &recipe.id, "uninstall", &format!("starting uninstall mode={}", mode.label()))?;

    best_effort_stop(app, recipe, mode.remove_data())?;
    cleanup_runtime_artifacts(app, recipe, mode.remove_data())?;
    clear_known_secrets_if_requested(recipe, mode.remove_data())?;

    let note = if mode.remove_data() {
        "Uninstalled and removed OpenNest-managed app data. Docker volumes were removed when supported by the runtime."
    } else {
        "Uninstalled runtime resources while keeping app data, logs, compose files, official source, state, and secrets."
    };

    let status = status_store::mark_uninstalled(app, &recipe.id, Some(note.to_string()))?;
    logs::append(app, &recipe.id, "uninstall", note)?;
    Ok(status)
}
