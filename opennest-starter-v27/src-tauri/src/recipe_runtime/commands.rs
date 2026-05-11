use tauri::AppHandle;

use super::install_plan::RecipeInstallPlan;
use super::port_resolver::PortResolutionResult;
use super::resource_preflight::ResourcePreflightReport;
use super::status::{OpenClawSetupInput, RecipeSecretInput, RecipeStatus, RecipeSummary, RuntimeActionError, RuntimeActionResult};
use super::{recipe_loader, runtime_router};

#[tauri::command]
pub async fn recipe_list_apps() -> Result<Vec<RecipeSummary>, String> {
    recipe_loader::list_recipes()
}

#[tauri::command]
pub async fn recipe_get_status(app: AppHandle, app_id: String) -> Result<RecipeStatus, String> {
    runtime_router::get_status(&app, &app_id)
}


#[tauri::command]
pub async fn recipe_get_install_plan(app: AppHandle, app_id: String) -> Result<RecipeInstallPlan, String> {
    runtime_router::get_install_plan(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_run_resource_preflight(app: AppHandle, app_id: String) -> Result<ResourcePreflightReport, String> {
    runtime_router::run_resource_preflight(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_resolve_ports(app: AppHandle, app_id: String) -> Result<PortResolutionResult, String> {
    runtime_router::resolve_ports(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_accept_install_plan(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::accept_install_plan(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_clear_install_plan_acceptance(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::clear_install_plan_acceptance(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_check_environment(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::check_environment(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_install(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::install(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_start(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::start(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_stop(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::stop(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_restart(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::restart(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_open_dashboard(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::open_dashboard(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_check_health(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::check_health(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_check_gateway_status(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::check_gateway_status(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_check_runtime_status(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::check_runtime_status(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_check_readiness(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::check_readiness(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_read_logs(app: AppHandle, app_id: String) -> Result<Vec<String>, String> {
    runtime_router::read_logs(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_run_doctor(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::run_doctor(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_run_onboarding(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::run_onboarding(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_repair(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::repair(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_get_runtime_error(app: AppHandle, app_id: String) -> Result<Option<RuntimeActionError>, String> {
    runtime_router::get_runtime_error(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_retry_runtime_error(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::retry_runtime_error(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_repair_runtime_error(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::repair_runtime_error(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_save_secrets(app: AppHandle, app_id: String, secrets: Vec<RecipeSecretInput>) -> Result<RuntimeActionResult, String> {
    runtime_router::save_secrets(&app, &app_id, secrets)
}

#[tauri::command]
pub async fn recipe_configure_openclaw(app: AppHandle, input: OpenClawSetupInput) -> Result<RuntimeActionResult, String> {
    runtime_router::configure_openclaw(&app, input)
}

#[tauri::command]
pub async fn recipe_rollback_failed_install(app: AppHandle, app_id: String) -> Result<RuntimeActionResult, String> {
    runtime_router::rollback_failed_install(&app, &app_id)
}

#[tauri::command]
pub async fn recipe_import_user_recipe(recipe_json: String) -> Result<super::recipe_loader::OpenNestRecipe, String> {
    super::recipe_loader::import_user_recipe(&recipe_json)
}

#[tauri::command]
pub async fn recipe_remove_user_recipe(app_id: String) -> Result<(), String> {
    super::recipe_loader::remove_user_recipe(&app_id)
}

#[tauri::command]
pub fn recipe_fetch_marketplace() -> Result<Vec<super::recipe_loader::OpenNestRecipe>, String> {
    super::recipe_loader::fetch_marketplace_recipes()
}

#[tauri::command]
pub async fn recipe_uninstall(app: AppHandle, app_id: String, remove_data: bool) -> Result<RuntimeActionResult, String> {
    runtime_router::uninstall(&app, &app_id, remove_data)
}
