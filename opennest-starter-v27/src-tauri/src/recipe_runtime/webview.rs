use chrono::Utc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};


use super::http_readiness;
use super::logs;
use super::paths;
use super::port_resolver;
use super::recipe_loader::OpenNestRecipe;
use super::status::RecipeStatus;
use super::status_store;

fn window_label(app_id: &str) -> String {
    let slug = app_id.replace(|ch: char| !ch.is_ascii_alphanumeric(), "-");
    format!("app-window-{slug}")
}

fn resolve_dashboard_url(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<String> {
    recipe
        .dashboard
        .as_ref()
        .and_then(|dash| dash.url.as_deref().or(dash.fallback_url.as_deref()).map(ToString::to_string))
        .or_else(|| port_resolver::effective_dashboard_url(app, recipe))
}

fn build_webview_url(raw: &str) -> Result<WebviewUrl, String> {
    raw.parse::<url::Url>()
        .map(WebviewUrl::External)
        .map_err(|error| format!("invalid webview URL '{raw}': {error}"))
}

pub fn check_environment(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<(), String> {
    let url_str = resolve_dashboard_url(app, recipe)
        .ok_or_else(|| format!("{} recipe does not define a dashboard URL", recipe.id))?;

    build_webview_url(&url_str)?;

    logs::append(
        app,
        &recipe.id,
        "webview",
        &format!("environment check passed; resolved URL: {url_str}"),
    )?;

    Ok(())
}

pub fn install(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "webview" {
        return Err(format!("{} is not a webview recipe.", recipe.id));
    }

    let app_dir = paths::app_dir(app, &recipe.id)?;
    std::fs::create_dir_all(&app_dir).map_err(|error| format!("failed to create app directory: {error}"))?;

    let url_str = resolve_dashboard_url(app, recipe)
        .unwrap_or_else(|| "about:blank".to_string());

    logs::append(
        app,
        &recipe.id,
        "install",
        &format!("webview app registered; dashboard URL: {url_str}"),
    )?;

    status_store::mark_installed(app, &recipe.id)
}

pub fn start(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "webview" {
        return Err(format!("{} is not a webview recipe.", recipe.id));
    }

    let url_str = resolve_dashboard_url(app, recipe)
        .ok_or_else(|| format!("{} recipe does not define a dashboard URL", recipe.id))?;

    let status = status_store::load(app, &recipe.id).unwrap_or_else(|_| RecipeStatus::default_for(&recipe.id));
    if !status.installed {
        return Err(format!("{} must be installed before starting.", recipe.name));
    }

    let label = window_label(&recipe.id);
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.close();
        thread::sleep(Duration::from_millis(300));
    }

    let resolved_url = build_webview_url(&url_str)?;
    let title = format!("{} | OpenNest", recipe.name);

    logs::append(
        app,
        &recipe.id,
        "start",
        &format!("opening webview window label={label} url={url_str}"),
    )?;

    WebviewWindowBuilder::new(app, &label, resolved_url)
        .title(&title)
        .inner_size(1440.0, 920.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .focused(true)
        .build()
        .map_err(|error| format!("failed to build webview window for {}: {error}", recipe.id))?;

    status_store::mark_running(app, &recipe.id)
}

pub fn stop(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    if recipe.runtime != "webview" {
        return Err(format!("{} is not a webview recipe.", recipe.id));
    }

    let label = window_label(&recipe.id);
    match app.get_webview_window(&label) {
        Some(window) => {
            let _ = window.close();
            logs::append(
                app,
                &recipe.id,
                "stop",
                &format!("closed webview window label={label}"),
            )?;
        }
        None => {
            logs::append(
                app,
                &recipe.id,
                "stop",
                &format!("webview window label={label} was not open"),
            )?;
        }
    }

    status_store::mark_stopped(app, &recipe.id)
}

pub fn open_dashboard(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let label = window_label(&recipe.id);

    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.set_focus();
        return status_store::load(app, &recipe.id)
            .or_else(|_| Ok(RecipeStatus::default_for(&recipe.id)));
    }

    start(app, recipe)
}

pub fn check_health(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let label = window_label(&recipe.id);
    let window_open = app.get_webview_window(&label).is_some();

    if window_open {
        status_store::mark_healthy(app, &recipe.id)
    } else {
        if let Some(_url) = http_readiness::readiness_url_for(app, recipe) {
            return http_readiness::ensure_ready(app, recipe);
        }
        status_store::mark_unhealthy(
            app,
            &recipe.id,
            format!("webview window label={label} is not open and no HTTP readiness URL configured"),
        )
        .map(|mut status| {
            status.run_state = "stopped".to_string();
            status
        })
    }
}

pub fn check_readiness(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let label = window_label(&recipe.id);
    if app.get_webview_window(&label).is_some() {
        let now = Utc::now().to_rfc3339();
        let mut status = status_store::load(app, &recipe.id)?;
        status.readiness_state = Some("ready".to_string());
        status.readiness_checked_at = Some(now);
        status_store::save(app, &status)?;
        Ok(status)
    } else {
        Err(format!("webview window label={label} is not open"))
    }
}

pub fn read_logs(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<Vec<String>, String> {
    logs::read_tail(app, &recipe.id, 200)
}

pub fn uninstall(app: &AppHandle, recipe: &OpenNestRecipe, remove_data: bool) -> Result<RecipeStatus, String> {
    let label = window_label(&recipe.id);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.close();
    }

    if remove_data {
        let app_dir = paths::app_dir(app, &recipe.id)?;
        if app_dir.exists() {
            std::fs::remove_dir_all(&app_dir)
                .map_err(|error| format!("failed to remove app directory {}: {error}", app_dir.display()))?;
        }
    }

    status_store::mark_uninstalled(app, &recipe.id, None)
}

pub fn reconcile_status(app: &AppHandle, recipe: &OpenNestRecipe, mut status: RecipeStatus) -> RecipeStatus {
    if recipe.runtime != "webview" {
        return status;
    }

    let label = window_label(&recipe.id);
    let window_open = app.get_webview_window(&label).is_some();

    if window_open {
        if status.run_state != "running" {
            status.run_state = "running".to_string();
            status.health_state = Some("healthy".to_string());
            status.readiness_state = Some("ready".to_string());
        }
    } else if status.installed {
        status.run_state = "stopped".to_string();
        status.health_state = None;
        status.readiness_state = None;
    }

    status
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::recipe_loader::RecipeDashboardSpec;

    #[test]
    fn test_window_label_generates_slug() {
        let label = window_label("my-test-app");
        assert!(label.starts_with("app-window-"));
        assert!(!label.contains(" "));
    }

    #[test]
    fn test_window_label_handles_special_chars() {
        let label = window_label("foo@bar#baz!");
        assert!(label.starts_with("app-window-"));
        assert!(!label.contains('@'));
        assert!(!label.contains('#'));
        assert!(!label.contains('!'));
    }

    #[test]
    fn test_default_url_from_dashboard() {
        let recipe = OpenNestRecipe {
            dashboard: Some(RecipeDashboardSpec {
                strategy: "url".into(),
                url: Some("http://localhost:8080".into()),
                fallback_url: None,
                command: None,
                args: vec![],
            }),
            ..minimal_recipe("test-web")
        };
        let url = resolve_dashboard_url_for_test(&recipe);
        assert_eq!(url, Some("http://localhost:8080".into()));
    }

    fn minimal_recipe(id: &str) -> OpenNestRecipe {
        OpenNestRecipe {
            schema_version: "2.0.0".into(),
            id: id.into(),
            name: format!("Test {id}"),
            summary: "Test recipe".into(),
            description: None,
            runtime: "webview".into(),
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
            ports: vec![],
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

    fn resolve_dashboard_url_for_test(recipe: &OpenNestRecipe) -> Option<String> {
        recipe
            .dashboard
            .as_ref()
            .and_then(|dash| dash.url.as_deref().or(dash.fallback_url.as_deref()).map(ToString::to_string))
    }
}