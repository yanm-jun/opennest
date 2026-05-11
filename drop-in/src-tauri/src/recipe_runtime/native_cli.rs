use std::process::Command;
use tauri::AppHandle;

use super::{command_timeout, healthcheck, logs, node_runtime, paths, process_manager, token_store};

pub fn app_binary(app: &AppHandle, app_id: &str) -> Result<std::path::PathBuf, String> {
    Ok(paths::app_dir(app, app_id)?.join("cli").join("openclaw.cmd"))
}

pub fn install_app(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let dir = paths::app_dir(app, app_id)?;
    let prefix = dir.join("cli");
    std::fs::create_dir_all(&prefix).map_err(|e| e.to_string())?;
    let runtime = node_runtime::ensure_node_runtime(app, app_id)?;
    logs::append(app, app_id, "install", &format!("installing openclaw@latest with npm prefix using {}", runtime.describe()))?;

    let mut command = Command::new(&runtime.npm_path);
    command
        .arg("install")
        .arg("-g")
        .arg("openclaw@latest")
        .arg("--prefix")
        .arg(prefix.to_string_lossy().to_string())
        .env("PATH", runtime.path_env());

    let output = command_timeout::run_with_timeout(command, command_timeout::INSTALL_TIMEOUT_MS)
        .map_err(|error| format!("npm install failed to start or wait: {error}"))?;

    logs::append(app, app_id, "install", &format!("npm install finished duration_ms={} timed_out={} exit_code={:?}", output.duration_ms, output.timed_out, output.exit_code))?;
    logs::append(app, app_id, "install", &output.stdout)?;
    logs::append(app, app_id, "install", &output.stderr)?;

    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("npm install openclaw@latest"))
    }
}

pub fn start_app(app: &AppHandle, app_id: &str, port: u16) -> Result<u32, String> {
    // Repeated Start must be idempotent. If OpenNest already manages a healthy
    // OpenClaw process, return that PID instead of spawning a duplicate.
    if let Some(record) = process_manager::load_record(app, app_id)? {
        if process_manager::is_pid_running(record.pid) {
            let report = healthcheck::check_tcp("127.0.0.1", port, 750);
            if report.ok {
                logs::append(app, app_id, "process", &format!("managed OpenClaw is already running pid={}", record.pid))?;
                return Ok(record.pid);
            }

            logs::append( app, app_id, "process",
                &format!("managed pid={} exists but healthcheck failed; stopping before restart", record.pid),
            )?;
            let _ = process_manager::stop_managed(app, app_id);
        } else {
            process_manager::clear_record(app, app_id)?;
            logs::append(app, app_id, "process", &format!("cleared stale process record pid={}", record.pid))?;
        }
    }

    // If something is already listening on the default port but OpenNest does not
    // own it, do not spawn a second gateway and create port conflicts.
    let external_port = healthcheck::check_tcp("127.0.0.1", port, 500);
    if external_port.ok {
        return Err("OpenClaw port is already reachable, but OpenNest has no managed process record for it. OpenClaw may already be running outside OpenNest; stop it or resolve to another port before starting here.".to_string());
    }

    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet.".to_string());
    }
    let dir = paths::app_dir(app, app_id)?;
    let state_dir = dir.join("state");
    let config_dir = dir.join("config");
    std::fs::create_dir_all(&state_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let runtime = node_runtime::ensure_node_runtime(app, app_id)?;
    let mut cmd = Command::new(binary);
    cmd.args(["gateway", "--port", &port.to_string(), "--verbose"])
        .env("PATH", runtime.path_env())
        .env("OPENCLAW_HOME", &dir)
        .env("OPENCLAW_STATE_DIR", &state_dir)
        .env("OPENCLAW_CONFIG_PATH", config_dir.join("openclaw.json"));

    if let Some(token) = token_store::get(app_id, "openrouterApiToken") {
        cmd.env("OPENROUTER_API_KEY", token);
    }
    if let Some(token) = token_store::get(app_id, "openaiApiToken") {
        cmd.env("OPENAI_API_KEY", token);
    }
    if let Some(token) = token_store::get(app_id, "anthropicApiToken") {
        cmd.env("ANTHROPIC_API_KEY", token);
    }
    if let Some(token) = token_store::get(app_id, "geminiApiToken") {
        cmd.env("GEMINI_API_KEY", token);
    }

    let pid = process_manager::spawn_managed(app, app_id,
        cmd,
        &format!("openclaw gateway --port {port} --verbose"),
        Some(port),
    )?;

    if let Err(error) = healthcheck::wait_for_tcp(app, app_id, "127.0.0.1", port, 20_000, 500) {
        let _ = logs::append(app, app_id, "healthcheck", &format!("gateway did not become healthy; stopping managed pid={pid}: {error}"));
        let _ = process_manager::stop_managed(app, app_id);
        return Err(format!("OpenClaw process started pid={pid}, but gateway healthcheck failed on 127.0.0.1:{port}: {error}"));
    }

    Ok(pid)
}

pub fn stop_app(app: &AppHandle, app_id: &str) -> Result<String, String> {
    process_manager::stop_managed(app, app_id)
}

pub fn app_onboarding(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet.".to_string());
    }
    let runtime = node_runtime::ensure_node_runtime(app, app_id)?;
    logs::append(app, app_id, "onboarding", "opening official onboarding")?;
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .env("PATH", runtime.path_env())
            .args(["/C", "start", "OpenClaw Onboarding", binary.to_string_lossy().as_ref(), "onboard", "--install-daemon"])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Official onboarding launcher is currently implemented for Windows only.".to_string())
    }
}

pub fn app_doctor(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet.".to_string());
    }
    let runtime = node_runtime::ensure_node_runtime(app, app_id)?;
    logs::append(app, app_id, "doctor", "running openclaw doctor")?;

    let mut command = Command::new(&binary);
    command.arg("doctor").env("PATH", runtime.path_env());
    let output = command_timeout::run_with_timeout(command, command_timeout::CHECK_TIMEOUT_MS)
        .map_err(|error| format!("openclaw doctor failed to start or wait: {error}"))?;

    logs::append(app, app_id, "doctor", &format!("doctor finished duration_ms={} timed_out={} exit_code={:?}", output.duration_ms, output.timed_out, output.exit_code))?;
    logs::append(app, app_id, "doctor", &output.stdout)?;
    logs::append(app, app_id, "doctor", &output.stderr)?;

    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("openclaw doctor"))
    }
}

pub fn app_dashboard(app: &AppHandle, app_id: &str, port: u16) -> Result<(), String> {
    let report = healthcheck::check_tcp("127.0.0.1", port, 750);
    if !report.ok {
        return Err(format!(
            "OpenClaw Gateway is not reachable on 127.0.0.1:{port}. Start Gateway first. {}",
            report.error.unwrap_or_default()
        ));
    }

    let binary = openclaw_binary(app)?;
    if binary.exists() {
        let runtime = node_runtime::ensure_node_runtime(app, app_id)?;
        let _ = Command::new(&binary).env("PATH", runtime.path_env()).arg("dashboard").spawn();
    }
    tauri_plugin_opener::open_url(format!("http://127.0.0.1:{port}/"), None::<&str>).map_err(|e| e.to_string())?;
    logs::append(app, app_id, "dashboard", "opened dashboard")
}
