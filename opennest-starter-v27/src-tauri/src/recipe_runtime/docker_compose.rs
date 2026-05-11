use std::process::Command;
use tauri::AppHandle;

use super::{command_timeout, logs, paths};

#[derive(Debug, Clone)]
pub struct DockerEnvironmentReport {
    pub docker_version: String,
    pub compose_version: String,
    pub daemon_info: String,
}

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

fn run_logged_command(
    app: &AppHandle,
    app_id: &str,
    category: &str,
    action: &str,
    timeout_ms: u64,
    args: &[String],
) -> Result<command_timeout::TimedCommandOutput, String> {
    let mut command = Command::new("docker");
    command.args(args);
    let output = command_timeout::run_with_timeout(command, timeout_ms)
        .map_err(|error| format!("{action} failed to start or wait: {error}"))?;
    log_timed_output(app, app_id, category, action, &output)?;
    if output.success {
        Ok(output)
    } else {
        Err(output.failure_message(action))
    }
}

fn compose_args(compose_path: &std::path::Path, tail: &[&str]) -> Vec<String> {
    let mut args = vec![
        "compose".to_string(),
        "-f".to_string(),
        compose_path.to_string_lossy().to_string(),
    ];
    args.extend(tail.iter().map(|value| (*value).to_string()));
    args
}

pub fn check_docker(app: &AppHandle, app_id: &str) -> Result<DockerEnvironmentReport, String> {
    let docker_version = run_logged_command(
        app,
        app_id,
        "docker",
        "docker --version",
        command_timeout::CHECK_TIMEOUT_MS,
        &["--version".to_string()],
    )?;

    let compose_version = run_logged_command(
        app,
        app_id,
        "docker",
        "docker compose version",
        command_timeout::CHECK_TIMEOUT_MS,
        &["compose".to_string(), "version".to_string()],
    )?;

    let daemon_info = run_logged_command(
        app,
        app_id,
        "docker",
        "docker info",
        command_timeout::CHECK_TIMEOUT_MS,
        &["info".to_string()],
    )
    .map_err(|error| format!("Docker is installed, but Docker Desktop / daemon is not ready. Start Docker Desktop and wait until it is running. {error}"))?;

    Ok(DockerEnvironmentReport {
        docker_version: docker_version.stdout.trim().to_string(),
        compose_version: compose_version.stdout.trim().to_string(),
        daemon_info: daemon_info.stdout.trim().to_string(),
    })
}

pub fn write_compose(app: &AppHandle, app_id: &str, compose_content: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    std::fs::write(&compose_path, compose_content).map_err(|e| e.to_string())?;
    logs::append(app, app_id, "install", &format!("compose written: {}", compose_path.display()))
}

pub fn compose_pull(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["pull"]);
    let output = run_logged_command(app, app_id, "start", "docker compose pull", command_timeout::COMPOSE_UP_TIMEOUT_MS, &args)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose pull"))
    }
}

pub fn compose_up(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["up", "-d"]);
    let output = run_logged_command(app, app_id, "start", "docker compose up", command_timeout::COMPOSE_UP_TIMEOUT_MS, &args)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose up"))
    }
}

pub fn compose_ps(app: &AppHandle, app_id: &str) -> Result<String, String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["ps"]);
    let output = run_logged_command(app, app_id, "docker-status", "docker compose ps", command_timeout::CHECK_TIMEOUT_MS, &args)?;
    Ok(format!("{}{}", output.stdout, output.stderr).trim().to_string())
}

pub fn compose_ps_running_services(app: &AppHandle, app_id: &str) -> Result<Vec<String>, String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["ps", "--status", "running", "--services"]);
    let output = run_logged_command(
        app,
        app_id,
        "docker-status",
        "docker compose ps --status running --services",
        command_timeout::CHECK_TIMEOUT_MS,
        &args,
    )?;
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

pub fn compose_stop(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["stop"]);
    let output = run_logged_command(app, app_id, "stop", "docker compose stop", command_timeout::STOP_TIMEOUT_MS, &args)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose stop"))
    }
}

pub fn compose_logs(app: &AppHandle, app_id: &str) -> Result<Vec<String>, String> {
    let compose_path = paths::compose_file(app, app_id)?;
    let args = compose_args(&compose_path, &["logs", "--tail", "200"]);
    let output = run_logged_command(app, app_id, "logs", "docker compose logs", command_timeout::LOGS_TIMEOUT_MS, &args)?;
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

    let mut tail = vec!["down", "--remove-orphans"];
    if remove_volumes {
        tail.push("-v");
    }
    let args = compose_args(&compose_path, &tail);
    let output = run_logged_command(app, app_id, "uninstall", "docker compose down", command_timeout::STOP_TIMEOUT_MS, &args)?;
    if output.success {
        Ok(())
    } else {
        Err(output.failure_message("docker compose down"))
    }
}
