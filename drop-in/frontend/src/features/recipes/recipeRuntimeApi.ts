import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  OpenNestRecipeSummary,
  PortResolutionResult,
  RecipeInstallPlan,
  RecipeProgressEvent,
  RecipeSecretInput,
  RecipeStatus,
  ResourcePreflightReport,
  RuntimeActionResult,
} from "./types";

export const RECIPE_PROGRESS_EVENT = "opennest://recipe-progress";

export async function listRecipes(): Promise<OpenNestRecipeSummary[]> {
  return invoke("recipe_list_apps");
}

export async function getRecipeInstallPlan(appId: string): Promise<RecipeInstallPlan> {
  return invoke("recipe_get_install_plan", { appId });
}

export async function runResourcePreflight(appId: string): Promise<ResourcePreflightReport> {
  return invoke("recipe_run_resource_preflight", { appId });
}

export async function resolveRecipePorts(appId: string): Promise<PortResolutionResult> {
  return invoke("recipe_resolve_ports", { appId });
}

export async function acceptRecipeInstallPlan(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_accept_install_plan", { appId });
}

export async function clearRecipeInstallPlanAcceptance(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_clear_install_plan_acceptance", { appId });
}

export async function checkRecipeEnvironment(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_check_environment", { appId });
}

export async function installRecipe(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_install", { appId });
}

export async function startRecipe(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_start", { appId });
}

export async function stopRecipe(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_stop", { appId });
}

export async function restartRecipe(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_restart", { appId });
}

export async function openRecipeDashboard(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_open_dashboard", { appId });
}

export async function checkRecipeHealth(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_check_health", { appId });
}

export async function checkRecipeReadiness(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_check_readiness", { appId });
}

export async function readRecipeLogs(appId: string): Promise<string[]> {
  return invoke("recipe_read_logs", { appId });
}

export async function runRecipeDoctor(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_run_doctor", { appId });
}

export async function runRecipeOnboarding(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_run_onboarding", { appId });
}

export async function repairRecipe(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_repair", { appId });
}

export async function saveRecipeSecrets(appId: string, secrets: RecipeSecretInput[]): Promise<RuntimeActionResult> {
  return invoke("recipe_save_secrets", { appId, secrets });
}

export async function getRecipeStatus(appId: string): Promise<RecipeStatus> {
  return invoke("recipe_get_status", { appId });
}

export async function rollbackFailedInstall(appId: string): Promise<RuntimeActionResult> {
  return invoke("recipe_rollback_failed_install", { appId });
}

export async function uninstallRecipe(appId: string, removeData: boolean): Promise<RuntimeActionResult> {
  return invoke("recipe_uninstall", { appId, removeData });
}

export async function listenToRecipeProgress(
  appId: string,
  callback: (event: RecipeProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<RecipeProgressEvent>(RECIPE_PROGRESS_EVENT, (event) => {
    if (event.payload.appId === appId) {
      callback(event.payload);
    }
  });
}
