use std::process::Command;
use tauri::AppHandle;

use super::{command_timeout, logs, paths};

fn log_timed_output(app: &AppHandle, app_id: &str, category: &str, action: &str, output: &command_timeout::TimedCommandOutput) -> Result<(), String> {
    logs::append(
        app,
        app_id,
        category,
        &format!(
            "{action} finished duration_ms={} timed_out={} exit_code={:?}",
            output.duration_ms, output.timed_out, output.exit_code
        ),
    )?;
    logs::append(app, app_id, category, &output.stdout)?;
    logs::append(app, app_id, category, &output.stderr)
}

pub fn check_docker(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let mut command = Command::new("docker");
    command.args(["compose", "version"]);
    let output = command_timeout::run_with_timeout(command, command_timeout::CHECK_TIMEOUT_MS)
        .map_err(|error| format!("docker compose version failed to start or wait: {error}"))?;
    log_timed_output(app, app_id, "docker", "docker compose version", &output)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose version"))
    }
}

pub fn write_compose(app: &AppHandle, app_id: &str, compose_content: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    std::fs::write(&compose_path, compose_content).map_err(|e| e.to_string())?;
    logs::append(app, app_id, "install", &format!("compose written: {}", compose_path.display()))
}

pub fn compose_up(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let mut command = Command::new("docker");
    command
        .arg("compose")
        .arg("-f")
        .arg(compose_path.to_string_lossy().to_string())
        .arg("up")
        .arg("-d");
    let output = command_timeout::run_with_timeout(command, command_timeout::COMPOSE_UP_TIMEOUT_MS)
        .map_err(|error| format!("docker compose up failed to start or wait: {error}"))?;
    log_timed_output(app, app_id, "start", "docker compose up", &output)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose up"))
    }
}

pub fn compose_stop(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let mut command = Command::new("docker");
    command
        .arg("compose")
        .arg("-f")
        .arg(compose_path.to_string_lossy().to_string())
        .arg("stop");
    let output = command_timeout::run_with_timeout(command, command_timeout::STOP_TIMEOUT_MS)
        .map_err(|error| format!("docker compose stop failed to start or wait: {error}"))?;
    log_timed_output(app, app_id, "stop", "docker compose stop", &output)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose stop"))
    }
}

pub fn compose_logs(app: &AppHandle, app_id: &str) -> Result<Vec<String>, String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let mut command = Command::new("docker");
    command
        .arg("compose")
        .arg("-f")
        .arg(compose_path.to_string_lossy().to_string())
        .arg("logs")
        .arg("--tail")
        .arg("200");
    let output = command_timeout::run_with_timeout(command, command_timeout::LOGS_TIMEOUT_MS)
        .map_err(|error| format!("docker compose logs failed to start or wait: {error}"))?;
    if output.timed_out {
        return Err(output.failure_message("docker compose logs"));
    }
    let text = format!("{}{}", output.stdout, output.stderr);
    Ok(text.lines().map(logs::redact).collect())
}


pub fn compose_down(app: &AppHandle, app_id: &str, remove_volumes: bool) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    if !compose_path.exists() {
        logs::append(app, app_id, "uninstall", &format!("compose file not found; skipping docker compose down: {}", compose_path.display()))?;
        return Ok(());
    }

    let mut command = Command::new("docker");
    command
        .arg("compose")
        .arg("-f")
        .arg(compose_path.to_string_lossy().to_string())
        .arg("down")
        .arg("--remove-orphans");
    if remove_volumes {
        command.arg("-v");
    }

    let output = command_timeout::run_with_timeout(command, command_timeout::STOP_TIMEOUT_MS)
        .map_err(|error| format!("docker compose down failed to start or wait: {error}"))?;
    log_timed_output(app, app_id, "uninstall", "docker compose down", &output)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose down"))
    }
}
