use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use super::install_plan::RecipeInstallPlan;
use super::status::{OpenClawSetupInput, RecipeSecretInput, RecipeStatus, RuntimeActionError, RuntimeActionResult};
use super::{
    docker_compose, docker_status, external_compose, healthcheck, http_readiness, install_plan, logs, native_cli, node_runtime, process_manager, port_resolver, preflight_gate, progress_events, recipe_loader, resource_preflight, rollback_adapter, secret_redaction_registry, status_store, token_store, agent_container, mcp_server, webview,
};

fn validate_recipe(app_id: &str) -> Result<recipe_loader::OpenNestRecipe, RuntimeActionResult> {
    recipe_loader::load_recipe(app_id)
        .map_err(|error| RuntimeActionResult::fail(app_id, error, None))
}

fn current_status(app: &AppHandle, recipe: &recipe_loader::OpenNestRecipe) -> RecipeStatus {
    let status = status_store::load(app, &recipe.id).unwrap_or_else(|_| RecipeStatus::default_for(&recipe.id));

    match recipe.runtime.as_str() {
        // OpenClaw is a managed native process in this adapter, so every status read
        // must reconcile the stored status with the real PID + port health.
        "native-cli" if recipe.id == "openclaw" => process_manager::reconcile_status(app, status),
        "docker-compose" => docker_status::reconcile_status(app, recipe, status),
        "external-compose" => external_compose::reconcile_status(app, recipe, status),
        "webview" => webview::reconcile_status(app, recipe, status),
        "mcp-server" => mcp_server::reconcile_status(app, recipe, status),
        "agent-container" => agent_container::reconcile_status(app, recipe, status),
        _ => status,
    }
}

fn current_status_by_id(app: &AppHandle, app_id: &str) -> RecipeStatus {
    match recipe_loader::load_recipe(app_id) {
        Ok(recipe) => current_status(app, &recipe),
        Err(_) => status_store::load(app, app_id).unwrap_or_else(|_| RecipeStatus::default_for(app_id)),
    }
}

fn fail_with_persisted_status(app: &AppHandle, app_id: &str, error: impl Into<String>) -> RuntimeActionResult {
    let error_text = error.into();
    let status = status_store::mark_error(app, app_id, error_text.clone())
        .unwrap_or_else(|_| current_status_by_id(app, app_id));
    RuntimeActionResult::fail(app_id, error_text, Some(status))
}


fn finish_progress(app: &AppHandle, app_id: &str, operation_id: &str, operation: &str, total_steps: u32, result: &RuntimeActionResult) {
    progress_events::finish_from_result(
        app,
        app_id,
        operation_id,
        operation,
        total_steps,
        result.ok,
        result.message.clone(),
        result.error.as_ref().map(|error| error.message.clone()),
    );
}

fn app_window_label(app_id: &str) -> String {
    format!("app-window-{}", app_id.replace(|ch: char| !ch.is_ascii_alphanumeric(), "-"))
}

fn open_embedded_app_window(app: &AppHandle, app_id: &str, title: &str, url: &str) -> Result<(), String> {
    let label = app_window_label(app_id);
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.close();
    }

    let external_url = url
        .parse()
        .map_err(|error| format!("invalid app URL {url}: {error}"))?;

    WebviewWindowBuilder::new(app, &label, WebviewUrl::External(external_url))
        .title(title)
        .inner_size(1440.0, 920.0)
        .min_inner_size(1100.0, 720.0)
        .resizable(true)
        .focused(true)
        .build()
        .map_err(|error| format!("failed to open app window: {error}"))?;

    Ok(())
}

fn install_fail_with_persisted_status(app: &AppHandle, app_id: &str, error: impl Into<String>) -> RuntimeActionResult {
    let error_text = error.into();
    let status = status_store::mark_install_error(app, app_id, error_text.clone())
        .unwrap_or_else(|_| current_status_by_id(app, app_id));
    RuntimeActionResult::fail(app_id, error_text, Some(status))
}

pub fn get_status(app: &AppHandle, app_id: &str) -> Result<RecipeStatus, String> {
    let recipe = recipe_loader::load_recipe(app_id)?;
    Ok(current_status(app, &recipe))
}

pub fn get_install_plan(app: &AppHandle, app_id: &str) -> Result<RecipeInstallPlan, String> {
    let recipe = recipe_loader::load_recipe(app_id)?;
    install_plan::build(app, &recipe)
}

pub fn run_resource_preflight(app: &AppHandle, app_id: &str) -> Result<resource_preflight::ResourcePreflightReport, String> {
    let recipe = recipe_loader::load_recipe(app_id)?;
    let total_steps = 2;
    let operation = "resource-preflight";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Running resource preflight for {}.", recipe.name));
    progress_events::step(app, &recipe.id, &progress_id, operation, "check-resources", 1, total_steps, "Checking machine resources, ports, tools, and network.");
    match resource_preflight::run(app, &recipe) {
        Ok(report) => {
            if report.ok {
                progress_events::succeeded(app, &recipe.id, &progress_id, operation, total_steps, report.summary.clone());
            } else {
                progress_events::failed(app, &recipe.id, &progress_id, operation, total_steps, "Resource preflight blocked this app.", report.summary.clone());
            }
            Ok(report)
        }
        Err(error) => {
            progress_events::failed(app, &recipe.id, &progress_id, operation, total_steps, "Resource preflight failed.", error.clone());
            Err(error)
        }
    }
}

pub fn resolve_ports(app: &AppHandle, app_id: &str) -> Result<port_resolver::PortResolutionResult, String> {
    let recipe = recipe_loader::load_recipe(app_id)?;
    let total_steps = 2;
    let operation = "resolve-ports";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Resolving ports for {}.", recipe.name));
    progress_events::step(app, &recipe.id, &progress_id, operation, "scan-and-map", 1, total_steps, "Checking requested ports and finding safe replacements when needed.");
    match port_resolver::resolve_ports(app, &recipe, true) {
        Ok(result) => {
            if result.ok {
                progress_events::succeeded(app, &recipe.id, &progress_id, operation, total_steps, result.message.clone());
            } else {
                progress_events::failed(app, &recipe.id, &progress_id, operation, total_steps, "Port resolution failed.", result.message.clone());
            }
            Ok(result)
        }
        Err(error) => {
            progress_events::failed(app, &recipe.id, &progress_id, operation, total_steps, "Port resolution failed.", error.clone());
            Err(error)
        }
    }
}

pub fn accept_install_plan(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    match preflight_gate::accept_install_plan(app, &recipe) {
        Ok(status) => Ok(RuntimeActionResult::ok(
            &recipe.id,
            "Install plan accepted. You can now install this app.",
            Some(status),
        )),
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}

pub fn clear_install_plan_acceptance(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    match preflight_gate::clear_install_plan_acceptance(app, &recipe) {
        Ok(status) => Ok(RuntimeActionResult::ok(
            &recipe.id,
            "Install plan acceptance cleared.",
            Some(status),
        )),
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}

pub fn check_environment(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let total_steps = 2;
    let operation = "check-environment";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Checking {} environment.", recipe.name));
    progress_events::step(app, &recipe.id, &progress_id, operation, "runtime-check", 1, total_steps, "Checking required runtime tools.");

    let result = match recipe.runtime.as_str() {
        "native-cli" if recipe.id == "openclaw" => {
            logs::append(app, &recipe.id, "environment", "checking OpenClaw Node runtime environment")?;
            let report = node_runtime::inspect_runtime(app);
            let status = status_store::mark_node_runtime(
                app,
                &recipe.id,
                report.source.clone(),
                report.version.clone(),
                report.node_path.clone(),
                report.npm_path.clone(),
            )?;

            if report.usable {
                RuntimeActionResult::ok(&recipe.id, report.message, Some(status))
            } else {
                RuntimeActionResult::fail(&recipe.id, report.message, Some(status))
            }
        }
        "docker-compose" => match docker_compose::check_docker(app, &recipe.id) {
            Ok(_) => RuntimeActionResult::ok(&recipe.id, "Docker Compose is available.", Some(current_status(app, &recipe))),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "external-compose" => match external_compose::check_environment(app, &recipe) {
            Ok(_) => RuntimeActionResult::ok(&recipe.id, "Docker Compose and Git are available.", Some(current_status(app, &recipe))),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "agent-container" => match agent_container::check_environment(app, &recipe) {
            Ok(_) => RuntimeActionResult::ok(&recipe.id, "Agent environment OK.", Some(current_status(app, &recipe))),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "mcp-server" => match mcp_server::check_environment(app, &recipe) {
            Ok(_) => RuntimeActionResult::ok(&recipe.id, "MCP server executable found.", Some(current_status(app, &recipe))),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "webview" => match webview::check_environment(app, &recipe) {
            Ok(_) => RuntimeActionResult::ok(&recipe.id, "Webview URL is valid.", Some(current_status(app, &recipe))),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        other => fail_with_persisted_status(
            app,
            &recipe.id,
            format!("Runtime {other} environment check is not implemented in the current adapter set."),
        ),
    };
    finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
    Ok(result)
}

pub fn install(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let total_steps = 6;
    let operation = "install";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Preparing to install {}.", recipe.name));

    progress_events::step(app, &recipe.id, &progress_id, operation, "install-plan", 1, total_steps, "Generating and logging install plan.");
    let plan = install_plan::build(app, &recipe)?;
    logs::append(
        app,
        &recipe.id,
        "install-plan",
        &format!(
            "install plan generated version={} digest={} risk={} time={} disk={} ports={:?} requires_node={} requires_docker={} requires_git={} requires_network={}",
            plan.plan_version,
            plan.plan_digest,
            plan.risk_level,
            plan.estimated_time,
            plan.estimated_disk,
            plan.ports,
            plan.requires_node,
            plan.requires_docker,
            plan.requires_git,
            plan.requires_network
        ),
    )?;

    progress_events::step(app, &recipe.id, &progress_id, operation, "preflight-gate", 2, total_steps, "Checking whether the install plan was accepted.");
    if let Err(error) = preflight_gate::ensure_install_allowed(app, &recipe) {
        let result = RuntimeActionResult::fail(&recipe.id, error, Some(current_status(app, &recipe)));
        finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
        return Ok(result);
    }

    progress_events::step(app, &recipe.id, &progress_id, operation, "resource-preflight", 3, total_steps, "Running machine and dependency preflight checks.");
    let resource_report = match resource_preflight::run(app, &recipe) {
        Ok(report) => report,
        Err(error) => {
            let result = RuntimeActionResult::fail(&recipe.id, error, Some(current_status(app, &recipe)));
            finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
            return Ok(result);
        }
    };
    if !resource_report.ok {
        let result = RuntimeActionResult::fail(
            &recipe.id,
            resource_preflight::summarize_blockers(&resource_report),
            Some(current_status(app, &recipe)),
        );
        finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
        return Ok(result);
    }

    progress_events::step(app, &recipe.id, &progress_id, operation, "port-resolution", 4, total_steps, "Resolving local ports and dashboard URLs.");
    if let Err(error) = port_resolver::ensure_ports(app, &recipe) {
        let result = RuntimeActionResult::fail(&recipe.id, error, Some(current_status(app, &recipe)));
        finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
        return Ok(result);
    }

    progress_events::step(app, &recipe.id, &progress_id, operation, "install-runtime", 5, total_steps, "Installing runtime assets and writing app files.");
    let _ = status_store::mark_installing(app, &recipe.id)?;

    let result = match recipe.runtime.as_str() {
        "native-cli" if recipe.id == "openclaw" => match native_cli::install_openclaw(app) {
            Ok(_) => {
                let status = status_store::mark_installed(app, &recipe.id)?;
                let report = node_runtime::inspect_runtime(app);
                let _status = status_store::mark_node_runtime(
                    app,
                    &recipe.id,
                    report.source,
                    report.version,
                    report.node_path,
                    report.npm_path,
                ).unwrap_or(status);
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, "OpenClaw installed.", Some(current_status(app, &recipe)))
            }
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        "docker-compose" => match recipe_loader::compose_content_for(&recipe)
            .and_then(|compose| port_resolver::rewrite_compose_content(app, &recipe, compose))
            .and_then(|compose| docker_compose::write_compose(app, &recipe.id, &compose))
        {
            Ok(_) => {
                let _status = status_store::mark_installed(app, &recipe.id)?;
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{} compose file written.", recipe.name), Some(current_status(app, &recipe)))
            }
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        "external-compose" => match external_compose::install(app, &recipe) {
            Ok(_status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(
                    &recipe.id,
                    format!("{} official compose source synced and installed.", recipe.name),
                    Some(current_status(app, &recipe)),
                )
            },
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        "agent-container" => match agent_container::install(app, &recipe) {
            Ok(_status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} agent-container registered.", recipe.name), Some(current_status(app, &recipe)))
            }
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        "mcp-server" => match mcp_server::install(app, &recipe) {
            Ok(_status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} mcp-server registered.", recipe.name), Some(current_status(app, &recipe)))
            }
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        "webview" => match webview::install(app, &recipe) {
            Ok(_status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} webview app registered.", recipe.name), Some(current_status(app, &recipe)))
            }
            Err(error) => install_fail_with_persisted_status(app, &recipe.id, error),
        },
        other => install_fail_with_persisted_status(
            app,
            &recipe.id,
            format!("Runtime {other} install is not implemented in the current adapter set."),
        ),
    };
    progress_events::step(app, &recipe.id, &progress_id, operation, "finalize", 6, total_steps, "Finalizing install status.");
    finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
    Ok(result)
}

pub fn start(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let total_steps = 5;
    let operation = "start";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Starting {}.", recipe.name));

    progress_events::step(app, &recipe.id, &progress_id, operation, "port-resolution", 1, total_steps, "Resolving runtime ports before start.");
    if let Err(error) = port_resolver::ensure_ports(app, &recipe) {
        let result = RuntimeActionResult::fail(&recipe.id, error, Some(current_status(app, &recipe)));
        finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
        return Ok(result);
    }

    progress_events::step(app, &recipe.id, &progress_id, operation, "mark-starting", 2, total_steps, "Marking app as starting.");
    let _ = status_store::mark_starting(app, &recipe.id)?;

    progress_events::step(app, &recipe.id, &progress_id, operation, "runtime-start", 3, total_steps, "Starting app runtime.");
    let result = match recipe.runtime.as_str() {
        "native-cli" if recipe.id == "openclaw" => match native_cli::start_openclaw(app, 18789) {
            Ok(pid) => {
                if let Some(pid) = pid {
                    let _ = status_store::mark_running_with_pid(app, &recipe.id, pid)?;
                    let _ = status_store::mark_healthy(app, &recipe.id)?;
                    let _ = port_resolver::clear_overrides(app, &recipe.id);
                    RuntimeActionResult::ok(
                        &recipe.id,
                        format!("OpenClaw gateway started and healthcheck passed. pid={pid}"),
                        Some(current_status(app, &recipe)),
                    )
                } else {
                    let _ = status_store::mark_running(app, &recipe.id)?;
                    let _ = status_store::mark_healthy(app, &recipe.id)?;
                    let _ = port_resolver::clear_overrides(app, &recipe.id);
                    RuntimeActionResult::ok(
                        &recipe.id,
                        "OpenClaw gateway started via official gateway controller and healthcheck passed.",
                        Some(current_status(app, &recipe)),
                    )
                }
            }
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "docker-compose" => match recipe_loader::compose_content_for(&recipe)
            .and_then(|compose| port_resolver::rewrite_compose_content(app, &recipe, compose))
            .and_then(|compose| docker_compose::write_compose(app, &recipe.id, &compose))
            .and_then(|_| docker_compose::compose_up(app, &recipe.id))
            .and_then(|_| docker_status::ensure_running(app, &recipe))
        {
            Ok(status) => {
                progress_events::step(app, &recipe.id, &progress_id, operation, "readiness", 4, total_steps, "Waiting for HTTP readiness.");
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(
                    &recipe.id,
                    format!("{} started. Running services: {}", recipe.name, status.services.join(", ")),
                    Some(current_status(app, &recipe)),
                )
            },
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "external-compose" => match external_compose::start(app, &recipe) {
            Ok(status) => {
                progress_events::step(app, &recipe.id, &progress_id, operation, "readiness", 4, total_steps, "Waiting for HTTP readiness.");
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(
                    &recipe.id,
                    format!("{} started. Running services: {}", recipe.name, status.services.join(", ")),
                    Some(current_status(app, &recipe)),
                )
            },
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "agent-container" => match agent_container::start(app, &recipe) {
            Ok(status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} agent started.", recipe.name), Some(status))
            }
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "mcp-server" => match mcp_server::start(app, &recipe) {
            Ok(status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} mcp-server started.", recipe.name), Some(status))
            }
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "webview" => match webview::start(app, &recipe) {
            Ok(_status) => {
                let _ = port_resolver::resolve_ports(app, &recipe, true);
                RuntimeActionResult::ok(&recipe.id, format!("{0} webview window opened.", recipe.name), Some(current_status(app, &recipe)))
            }
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        other => fail_with_persisted_status(
            app,
            &recipe.id,
            format!("Runtime {other} start is not implemented in the current adapter set."),
        ),
    };
    progress_events::step(app, &recipe.id, &progress_id, operation, "finalize", 5, total_steps, "Finalizing start state.");
    finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
    Ok(result)
}

pub fn stop(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let total_steps = 3;
    let operation = "stop";
    let progress_id = progress_events::begin(app, &recipe.id, operation, total_steps, format!("Stopping {}.", recipe.name));
    progress_events::step(app, &recipe.id, &progress_id, operation, "mark-stopping", 1, total_steps, "Marking app as stopping.");
    let _ = status_store::mark_stopping(app, &recipe.id)?;

    progress_events::step(app, &recipe.id, &progress_id, operation, "runtime-stop", 2, total_steps, "Stopping managed runtime resources.");
    let result = match recipe.runtime.as_str() {
        "native-cli" if recipe.id == "openclaw" => match native_cli::stop_openclaw(app) {
            Ok(message) => {
                logs::append(app, &recipe.id, "stop", &message)?;
                let status = status_store::mark_stopped(app, &recipe.id)?;
                RuntimeActionResult::ok(&recipe.id, message, Some(status))
            }
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "docker-compose" => match docker_compose::compose_stop(app, &recipe.id)
            .and_then(|_| docker_status::mark_after_stop(app, &recipe))
        {
            Ok(status) => RuntimeActionResult::ok(&recipe.id, format!("{} stopped.", recipe.name), Some(status)),
            Err(error) => {
                let status = status_store::mark_unhealthy(app, &recipe.id, error.clone())
                    .unwrap_or_else(|_| current_status(app, &recipe));
                RuntimeActionResult::fail(&recipe.id, error, Some(status))
            }
        },
        "external-compose" => match external_compose::stop(app, &recipe) {
            Ok(status) => RuntimeActionResult::ok(&recipe.id, format!("{} stopped.", recipe.name), Some(status)),
            Err(error) => {
                let status = status_store::mark_unhealthy(app, &recipe.id, error.clone())
                    .unwrap_or_else(|_| current_status(app, &recipe));
                RuntimeActionResult::fail(&recipe.id, error, Some(status))
            }
        },
        "agent-container" => match agent_container::stop(app, &recipe) {
            Ok(status) => RuntimeActionResult::ok(&recipe.id, format!("{0} agent stopped.", recipe.name), Some(status)),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "mcp-server" => match mcp_server::stop(app, &recipe) {
            Ok(status) => RuntimeActionResult::ok(&recipe.id, format!("{0} mcp-server stopped.", recipe.name), Some(status)),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        "webview" => match webview::stop(app, &recipe) {
            Ok(status) => RuntimeActionResult::ok(&recipe.id, format!("{} webview window closed.", recipe.name), Some(status)),
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        other => fail_with_persisted_status(
            app,
            &recipe.id,
            format!("Runtime {other} stop is not implemented in the current adapter set."),
        ),
    };
    progress_events::step(app, &recipe.id, &progress_id, operation, "finalize", 3, total_steps, "Finalizing stop state.");
    finish_progress(app, &recipe.id, &progress_id, operation, total_steps, &result);
    Ok(result)
}

pub fn restart(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let stop_result = stop(app, app_id)?;
    if !stop_result.ok {
        return Ok(stop_result);
    }
    start(app, app_id)
}

pub fn open_dashboard(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let result = match recipe.runtime.as_str() {
        "native-cli" if recipe.id == "openclaw" => match native_cli::openclaw_dashboard(app, 18789) {
            Ok(url) => match open_embedded_app_window(app, &recipe.id, "OpenClaw Desktop", &url) {
                Ok(_) => RuntimeActionResult::ok(&recipe.id, "OpenClaw Desktop chat opened.", Some(current_status(app, &recipe))),
                Err(error) => fail_with_persisted_status(app, &recipe.id, error),
            },
            Err(error) => fail_with_persisted_status(app, &recipe.id, error),
        },
        _ => match port_resolver::effective_dashboard_url(app, &recipe) {
            Some(url) => {
                let status = current_status(app, &recipe);
                if matches!(recipe.runtime.as_str(), "docker-compose" | "external-compose")
                    && status.readiness_state.as_deref() != Some("ready")
                {
                    return Ok(RuntimeActionResult::fail(
                        &recipe.id,
                        format!("{} dashboard is not HTTP-ready yet. Start the app and run Check Readiness first.", recipe.name),
                        Some(status),
                    ));
                }

                match tauri_plugin_opener::open_url(url.clone(), None::<&str>) {
                    Ok(_) => RuntimeActionResult::ok(
                        &recipe.id,
                        format!("Dashboard opened: {url}"),
                        Some(status),
                    ),
                    Err(error) => fail_with_persisted_status(app, &recipe.id, error.to_string()),
                }
            }
            None => fail_with_persisted_status(
                app,
                &recipe.id,
                format!("Recipe {} does not define dashboard.url or dashboard.fallbackUrl.", recipe.id),
            ),
        },
    };
    Ok(result)
}

pub fn check_health(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.runtime == "docker-compose" {
        let status = docker_status::reconcile_status(app, &recipe, current_status(app, &recipe));
        return if status.run_state == "running"
            && status.health_state.as_deref() == Some("healthy")
            && status.readiness_state.as_deref() == Some("ready")
        {
            Ok(RuntimeActionResult::ok(
                &recipe.id,
                format!("{} docker services are running and dashboard is HTTP-ready: {}", recipe.name, status.services.join(", ")),
                Some(status),
            ))
        } else {
            let error = status.last_error.clone().unwrap_or_else(|| format!("{} is not fully ready yet. Services or dashboard readiness failed.", recipe.name));
            Ok(RuntimeActionResult::fail(&recipe.id, error, Some(status)))
        };
    }

    if recipe.runtime == "external-compose" {
        let status = external_compose::reconcile_status(app, &recipe, current_status(app, &recipe));
        return if status.run_state == "running"
            && status.health_state.as_deref() == Some("healthy")
            && status.readiness_state.as_deref() == Some("ready")
        {
            Ok(RuntimeActionResult::ok(
                &recipe.id,
                format!("{} external compose services are running and dashboard is HTTP-ready: {}", recipe.name, status.services.join(", ")),
                Some(status),
            ))
        } else {
            let error = status.last_error.clone().unwrap_or_else(|| format!("{} is not fully ready yet. Services or dashboard readiness failed.", recipe.name));
            Ok(RuntimeActionResult::fail(&recipe.id, error, Some(status)))
        };
    }

    let result = match port_resolver::health_host_port(app, &recipe) {
        Some((host, port)) => {
            let report = healthcheck::check_tcp(&host, port, 750);
            if report.ok {
                let _ = status_store::mark_healthy(app, &recipe.id)?;
                let status = current_status(app, &recipe);
                RuntimeActionResult::ok(
                    &recipe.id,
                    format!("{} is reachable on {}:{}.", recipe.name, host, port),
                    Some(status),
                )
            } else {
                let error = report.error.unwrap_or_else(|| format!("{} is not reachable on {}:{}.", recipe.name, host, port));
                let status = status_store::mark_unhealthy(app, &recipe.id, error.clone())
                    .unwrap_or_else(|_| current_status(app, &recipe));
                RuntimeActionResult::fail(&recipe.id, error, Some(status))
            }
        }
        None => fail_with_persisted_status(
            app,
            &recipe.id,
            format!("Recipe {} does not define a local port or healthcheck URL.", recipe.id),
        ),
    };
    Ok(result)
}


pub fn check_readiness(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    // Compose apps should be allowed to run a readiness wait even when the last
    // persisted status is error/not_ready. First prove that services are really
    // running, then wait for the HTTP dashboard endpoint.
    match recipe.runtime.as_str() {
        "docker-compose" => {
            if let Err(error) = docker_status::ensure_running(app, &recipe) {
                let status = current_status(app, &recipe);
                return Ok(RuntimeActionResult::fail(&recipe.id, error, Some(status)));
            }
        }
        "external-compose" => {
            if let Err(error) = external_compose::ensure_running(app, &recipe) {
                let status = current_status(app, &recipe);
                return Ok(RuntimeActionResult::fail(&recipe.id, error, Some(status)));
            }
        }
        _ => {
            let status = current_status(app, &recipe);
            if status.run_state != "running" {
                return Ok(RuntimeActionResult::fail(
                    &recipe.id,
                    format!("{} is not running, so dashboard readiness cannot be checked.", recipe.name),
                    Some(status),
                ));
            }
        }
    }

    match http_readiness::ensure_ready(app, &recipe) {
        Ok(status) => Ok(RuntimeActionResult::ok(
            &recipe.id,
            format!(
                "{} dashboard is HTTP-ready at {}.",
                recipe.name,
                status.readiness_url.clone().unwrap_or_else(|| "configured readiness URL".to_string())
            ),
            Some(status),
        )),
        Err(error) => {
            let status = current_status(app, &recipe);
            Ok(RuntimeActionResult::fail(&recipe.id, error, Some(status)))
        }
    }
}

pub fn read_logs(app: &AppHandle, app_id: &str) -> Result<Vec<String>, String> {
    let recipe = recipe_loader::load_recipe(app_id)?;
    if recipe.runtime == "docker-compose" {
        if let Ok(lines) = docker_compose::compose_logs(app, &recipe.id) {
            return Ok(lines);
        }
    }
    if recipe.runtime == "external-compose" {
        if let Ok(lines) = external_compose::logs(app, &recipe) {
            return Ok(lines);
        }
    }
    logs::read_tail(app, &recipe.id, 200)
}

pub fn run_doctor(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.id == "openclaw" && recipe.runtime == "native-cli" {
        return match native_cli::openclaw_doctor(app) {
            Ok(_) => Ok(RuntimeActionResult::ok(
                &recipe.id,
                "OpenClaw doctor completed. Review logs for details.",
                Some(current_status(app, &recipe)),
            )),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }

    Ok(RuntimeActionResult::fail(
        &recipe.id,
        format!("No doctor adapter is implemented for runtime {} yet.", recipe.runtime),
        Some(current_status(app, &recipe)),
    ))
}

pub fn run_onboarding(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.id == "openclaw" && recipe.runtime == "native-cli" {
        return match native_cli::openclaw_onboarding(app) {
            Ok(_) => Ok(RuntimeActionResult::ok(
                &recipe.id,
                "Official OpenClaw onboarding opened.",
                Some(current_status(app, &recipe)),
            )),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }

    Ok(RuntimeActionResult::fail(
        &recipe.id,
        format!("No onboarding adapter is implemented for runtime {}.", recipe.runtime),
        Some(current_status(app, &recipe)),
    ))
}

pub fn repair(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.runtime == "native-cli" && recipe.id == "openclaw" {
        return match node_runtime::prepare_managed_node(app, &recipe.id) {
            Ok(runtime) => {
                let status = status_store::mark_node_runtime(
                    app,
                    &recipe.id,
                    Some(runtime.source.clone()),
                    Some(runtime.version.clone()),
                    Some(runtime.node_path.display().to_string()),
                    Some(runtime.npm_path.display().to_string()),
                )?;
                logs::append(app, &recipe.id, "repair", &format!("managed Node runtime prepared: {}", runtime.describe()))?;
                Ok(RuntimeActionResult::ok(&recipe.id, "Managed Node runtime prepared.", Some(status)))
            }
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }

    if recipe.runtime == "external-compose" {
        return match external_compose::initialize_env(app, &recipe) {
            Ok(_) => Ok(RuntimeActionResult::ok(
                &recipe.id,
                "External compose environment checked/initialized.",
                Some(current_status(app, &recipe)),
            )),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }

    if recipe.runtime == "docker-compose" {
        return match recipe_loader::compose_content_for(&recipe)
            .and_then(|compose| port_resolver::rewrite_compose_content(app, &recipe, compose))
            .and_then(|compose| docker_compose::write_compose(app, &recipe.id, &compose))
        {
            Ok(_) => Ok(RuntimeActionResult::ok(
                &recipe.id,
                format!("{} compose file repaired/re-written.", recipe.name),
                Some(current_status(app, &recipe)),
            )),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }

        if recipe.runtime == "agent-container" {
        return match agent_container::install(app, &recipe) {
            Ok(status) => Ok(RuntimeActionResult::ok(&recipe.id, format!("{0} agent re-registered.", recipe.name), Some(status))),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }
        if recipe.runtime == "mcp-server" {
        return match mcp_server::install(app, &recipe) {
            Ok(status) => Ok(RuntimeActionResult::ok(&recipe.id, format!("{0} mcp-server re-registered.", recipe.name), Some(status))),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }
    if recipe.runtime == "webview" {
        return match webview::install(app, &recipe) {
            Ok(status) => Ok(RuntimeActionResult::ok(&recipe.id, format!("{0} webview re-registered.", recipe.name), Some(status))),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        };
    }
    Ok(RuntimeActionResult::fail(
        &recipe.id,
        format!("No repair adapter is implemented for runtime {} yet.", recipe.runtime),
        Some(current_status(app, &recipe)),
    ))
}

pub fn save_secrets(app: &AppHandle, app_id: &str, secrets: Vec<RecipeSecretInput>) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let secret_count = secrets.len();
    match token_store::save(&recipe.id, secrets) {
        Ok(_) => {
            let _ = logs::append(app, &recipe.id, "secrets", &format!("saved {secret_count} secret value(s); dynamic redaction registry now tracks {} value(s) in this process", secret_redaction_registry::registered_count()));
            Ok(RuntimeActionResult::ok(&recipe.id, "Secrets saved and registered for dynamic redaction.", Some(current_status(app, &recipe))))
        }
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}

pub fn check_gateway_status(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.id != "openclaw" || recipe.runtime != "native-cli" {
        return Ok(RuntimeActionResult::fail(
            &recipe.id,
            "Gateway Status is currently implemented only for the OpenClaw native CLI adapter.",
            Some(current_status(app, &recipe)),
        ));
    }

    match native_cli::gateway_status_openclaw(app) {
        Ok(output) => Ok(RuntimeActionResult {
            ok: true,
            app_id: recipe.id.clone(),
            status: Some(current_status(app, &recipe)),
            message: Some("OpenClaw gateway status completed.".to_string()),
            logs: Some(vec![output]),
            error: None,
        }),
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}

pub fn check_runtime_status(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    match recipe.runtime.as_str() {
        "docker-compose" => match docker_compose::compose_ps(app, &recipe.id) {
            Ok(output) => Ok(RuntimeActionResult {
                ok: true,
                app_id: recipe.id.clone(),
                status: Some(current_status(app, &recipe)),
                message: Some(format!("{} container status refreshed.", recipe.name)),
                logs: Some(vec![output]),
                error: None,
            }),
            Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
        },
        "native-cli" if recipe.id == "openclaw" => check_gateway_status(app, app_id),
        other => Ok(RuntimeActionResult::fail(
            &recipe.id,
            format!("Runtime status is not implemented for {other}."),
            Some(current_status(app, &recipe)),
        )),
    }
}

pub fn get_runtime_error(app: &AppHandle, app_id: &str) -> Result<Option<RuntimeActionError>, String> {
    Ok(current_status_by_id(app, app_id).runtime_error)
}

pub fn retry_runtime_error(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let Some(error) = get_runtime_error(app, app_id)? else {
        return Ok(RuntimeActionResult::ok(
            app_id,
            "No persisted runtime error exists for this app.",
            Some(current_status_by_id(app, app_id)),
        ));
    };

    match error.code.as_str() {
        "OPENCLAW_GATEWAY_START_FAILED" => start(app, app_id),
        "OPENCLAW_DASHBOARD_FAILED" => open_dashboard(app, app_id),
        "OPENCLAW_GATEWAY_STATUS_FAILED" => check_gateway_status(app, app_id),
        "OPENCLAW_GATEWAY_PROBE_FAILED" | "OPENCLAW_PORT_UNREACHABLE" | "LOCAL_ENDPOINT_UNREACHABLE" => check_health(app, app_id),
        "DASHBOARD_READINESS_FAILED" => check_readiness(app, app_id),
        _ => Ok(RuntimeActionResult::fail(
            app_id,
            format!("No retry mapping is implemented for error code {}.", error.code),
            Some(current_status_by_id(app, app_id)),
        )),
    }
}

pub fn repair_runtime_error(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let Some(error) = get_runtime_error(app, app_id)? else {
        return Ok(RuntimeActionResult::ok(
            app_id,
            "No persisted runtime error exists for this app.",
            Some(current_status_by_id(app, app_id)),
        ));
    };

    match error.repair_action.as_deref() {
        Some("recheck-environment") => check_environment(app, app_id),
        Some("rerun-onboarding") => run_onboarding(app, app_id),
        Some("rerun-probe") => check_health(app, app_id),
        Some("reopen-dashboard") => open_dashboard(app, app_id),
        Some("recheck-docker") => check_environment(app, app_id),
        Some("resolve-ports") => resolve_ports(app, app_id).map(|result| {
            if result.ok {
                RuntimeActionResult::ok(app_id, result.message.clone(), Some(current_status_by_id(app, app_id)))
            } else {
                RuntimeActionResult::fail(app_id, result.message, Some(current_status_by_id(app, app_id)))
            }
        }),
        Some("rewrite-compose") => repair(app, app_id),
        Some(other) => Ok(RuntimeActionResult::fail(
            app_id,
            format!("Repair action {} is not implemented.", other),
            Some(current_status_by_id(app, app_id)),
        )),
        None => Ok(RuntimeActionResult::fail(
            app_id,
            format!("Error {} does not have a safe one-click repair action.", error.code),
            Some(current_status_by_id(app, app_id)),
        )),
    }
}

pub fn configure_openclaw(app: &AppHandle, input: OpenClawSetupInput) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe("openclaw") {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    if recipe.runtime != "native-cli" {
        return Ok(RuntimeActionResult::fail(
            &recipe.id,
            "OpenClaw desktop setup is only implemented for the native CLI runtime.",
            Some(current_status(app, &recipe)),
        ));
    }

    let provider = match native_cli::OpenClawProvider::parse(&input.provider) {
        Ok(provider) => provider,
        Err(error) => {
            return Ok(RuntimeActionResult::fail_with_detail(
                &recipe.id,
                "OPENCLAW_PROVIDER_UNSUPPORTED",
                "The selected model provider is not supported.",
                Some(error),
                Some("Choose DeepSeek, OpenAI, OpenRouter, or Anthropic.".to_string()),
                Some(current_status(app, &recipe)),
            ));
        }
    };

    let api_key = input.api_key.trim();
    if api_key.is_empty() {
        return Ok(RuntimeActionResult::fail_with_detail(
            &recipe.id,
            "OPENCLAW_API_KEY_REQUIRED",
            "The API key cannot be empty.",
            None,
            Some("Paste a provider API key, then try again.".to_string()),
            Some(current_status(app, &recipe)),
        ));
    }

    let secret_ids = vec![
        "deepseekApiToken".to_string(),
        "openaiApiToken".to_string(),
        "openrouterApiToken".to_string(),
        "anthropicApiToken".to_string(),
        "geminiApiToken".to_string(),
    ];
    let _ = token_store::delete_many(&recipe.id, &secret_ids);
    token_store::save(
        &recipe.id,
        vec![RecipeSecretInput {
            id: provider.secret_id().to_string(),
            value: api_key.to_string(),
        }],
    )?;

    if let Err(error) = native_cli::configure_openclaw_provider(app, provider, api_key) {
        return Ok(fail_with_persisted_status(app, &recipe.id, error));
    }

    let _ = status_store::mark_installed(app, &recipe.id);
    if let Err(error) = native_cli::start_openclaw(app, 18789) {
        return Ok(fail_with_persisted_status(app, &recipe.id, error));
    }

    if input.open_chat {
        return open_dashboard(app, &recipe.id);
    }

    Ok(RuntimeActionResult::ok(
        &recipe.id,
        "OpenClaw provider configured and gateway started.",
        Some(current_status(app, &recipe)),
    ))
}


pub fn rollback_failed_install(app: &AppHandle, app_id: &str) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    match rollback_adapter::rollback_failed_install(app, &recipe) {
        Ok(status) => Ok(RuntimeActionResult::ok(
            &recipe.id,
            "Rollback completed. Partial install artifacts were removed while logs/status were preserved.",
            Some(status),
        )),
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}

pub fn uninstall(app: &AppHandle, app_id: &str, remove_data: bool) -> Result<RuntimeActionResult, String> {
    let recipe = match validate_recipe(app_id) {
        Ok(recipe) => recipe,
        Err(result) => return Ok(result),
    };

    let mode = if remove_data {
        rollback_adapter::UninstallMode::RemoveData
    } else {
        rollback_adapter::UninstallMode::KeepData
    };

    match rollback_adapter::uninstall(app, &recipe, mode) {
        Ok(status) => Ok(RuntimeActionResult::ok(
            &recipe.id,
            if remove_data {
                "Uninstall completed and OpenNest-managed app data was removed."
            } else {
                "Uninstall completed while keeping app data and secrets."
            },
            Some(status),
        )),
        Err(error) => Ok(fail_with_persisted_status(app, &recipe.id, error)),
    }
}