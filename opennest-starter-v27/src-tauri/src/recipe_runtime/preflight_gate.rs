use tauri::AppHandle;

use super::install_plan::{self, RecipeInstallPlan};
use super::logs;
use super::recipe_loader::OpenNestRecipe;
use super::status::RecipeStatus;
use super::status_store;

fn current_plan(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeInstallPlan, String> {
    install_plan::build(app, recipe)
}

pub fn accept_install_plan(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let plan = current_plan(app, recipe)?;
    let status = status_store::mark_plan_accepted(
        app,
        &recipe.id,
        plan.plan_version.clone(),
        plan.plan_digest.clone(),
        plan.risk_level.clone(),
    )?;
    logs::append(
        app,
        &recipe.id,
        "preflight",
        &format!(
            "install plan accepted version={} digest={} risk={}",
            plan.plan_version, plan.plan_digest, plan.risk_level
        ),
    )?;
    Ok(status)
}

pub fn ensure_install_allowed(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<(), String> {
    let plan = current_plan(app, recipe)?;
    let status = status_store::load(app, &recipe.id)?;

    if status.plan_reviewed
        && status.plan_version.as_deref() == Some(plan.plan_version.as_str())
        && status.plan_digest.as_deref() == Some(plan.plan_digest.as_str())
    {
        return Ok(());
    }

    if status.plan_reviewed {
        let _ = logs::append(
            app,
            &recipe.id,
            "preflight",
            &format!(
                "install blocked because accepted plan is stale. accepted_version={:?} accepted_digest={:?} current_version={} current_digest={}",
                status.plan_version, status.plan_digest, plan.plan_version, plan.plan_digest
            ),
        );
        return Err(format!(
            "Install plan has changed or is stale. Review and accept the current plan before installing. Current digest: {}",
            plan.plan_digest
        ));
    }

    let _ = logs::append(
        app,
        &recipe.id,
        "preflight",
        &format!(
            "install blocked because plan has not been accepted. required_version={} required_digest={} risk={}",
            plan.plan_version, plan.plan_digest, plan.risk_level
        ),
    );
    Err(format!(
        "Review and accept the install plan before installing {}. Required digest: {}",
        recipe.name, plan.plan_digest
    ))
}

pub fn clear_install_plan_acceptance(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let status = status_store::clear_plan_acceptance(app, &recipe.id)?;
    logs::append(app, &recipe.id, "preflight", "install plan acceptance cleared")?;
    Ok(status)
}
