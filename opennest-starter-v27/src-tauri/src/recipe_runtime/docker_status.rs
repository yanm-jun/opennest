use tauri::AppHandle;

use super::docker_compose;
use super::http_readiness;
use super::paths;
use super::recipe_loader::OpenNestRecipe;
use super::status::RecipeStatus;
use super::status_store;

fn is_active_state(state: &str) -> bool {
    matches!(state, "running" | "starting" | "stopping")
}

pub fn running_services(app: &AppHandle, app_id: &str) -> Result<Vec<String>, String> {
    let compose_path = paths::compose_file(app, app_id)?;
    if !compose_path.exists() {
        return Err(format!("Compose file is missing: {}. Install the app first.", compose_path.display()));
    }
    docker_compose::compose_ps_running_services(app, app_id)
}

pub fn reconcile_status(app: &AppHandle, recipe: &OpenNestRecipe, status: RecipeStatus) -> RecipeStatus {
    if recipe.runtime != "docker-compose" || !status.installed {
        return status;
    }

    match running_services(app, &recipe.id) {
        Ok(services) if !services.is_empty() => {
            let running = status_store::mark_running_services(app, &recipe.id, services)
                .unwrap_or_else(|_| status);
            http_readiness::check_once(app, recipe, running)
        },
        Ok(_) => {
            if is_active_state(&status.run_state) {
                status_store::mark_stopped(app, &recipe.id).unwrap_or(status)
            } else {
                let mut next = status;
                next.services.clear();
                next.health_state = Some("unknown".to_string());
                status_store::save(app, &next).ok();
                next
            }
        }
        Err(error) => {
            if is_active_state(&status.run_state) {
                status_store::mark_error(app, &recipe.id, format!("Docker status check failed: {error}"))
                    .unwrap_or(status)
            } else {
                status
            }
        }
    }
}

pub fn ensure_running(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let services = running_services(app, &recipe.id)?;
    if services.is_empty() {
        let error = format!("{} started command completed, but docker compose ps reports no running services.", recipe.name);
        let _ = status_store::mark_unhealthy(app, &recipe.id, error.clone());
        return Err(error);
    }

    let _running = status_store::mark_running_services(app, &recipe.id, services)?;
    match http_readiness::ensure_ready(app, recipe) {
        Ok(ready_status) => Ok(ready_status),
        Err(error) => Err(format!("{} services are running, but dashboard is not HTTP-ready: {error}", recipe.name)),
    }
}

pub fn mark_after_stop(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let services = running_services(app, &recipe.id).unwrap_or_default();
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
