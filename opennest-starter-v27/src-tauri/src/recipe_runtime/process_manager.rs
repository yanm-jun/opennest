use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;

use super::{healthcheck, logs, paths};
use super::status::RecipeStatus;
use super::status_store;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagedProcessRecord {
    pub app_id: String,
    pub pid: u32,
    pub command: String,
    pub port: Option<u16>,
    pub started_at: String,
    pub last_seen_at: Option<String>,
}

fn process_file(app: &AppHandle, app_id: &str) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, app_id)?.join("process.json"))
}

pub fn load_record(app: &AppHandle, app_id: &str) -> Result<Option<ManagedProcessRecord>, String> {
    let path = process_file(app, app_id)?;
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(&path).map_err(|e| format!("failed to read process.json: {e}"))?;
    let record = serde_json::from_str::<ManagedProcessRecord>(&text)
        .map_err(|e| format!("process.json is corrupted: {e}"))?;
    Ok(Some(record))
}

pub fn save_record(app: &AppHandle, record: &ManagedProcessRecord) -> Result<(), String> {
    let path = process_file(app, &record.app_id)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create process dir: {e}"))?;
    }
    let content = serde_json::to_string_pretty(record).map_err(|e| format!("failed to serialize process record: {e}"))?;
    fs::write(path, content).map_err(|e| format!("failed to write process.json: {e}"))
}

pub fn clear_record(app: &AppHandle, app_id: &str) -> Result<(), String> {
    let path = process_file(app, app_id)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("failed to remove process.json: {e}"))?;
    }
    Ok(())
}

pub fn spawn_managed(
    app: &AppHandle,
    app_id: &str,
    mut command: Command,
    command_label: impl Into<String>,
    port: Option<u16>,
) -> Result<u32, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to start managed process: {e}"))?;

    let pid = child.id();
    let label = command_label.into();
    let record = ManagedProcessRecord {
        app_id: app_id.to_string(),
        pid,
        command: label.clone(),
        port,
        started_at: Utc::now().to_rfc3339(),
        last_seen_at: Some(Utc::now().to_rfc3339()),
    };
    if let Err(error) = save_record(app, &record) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("failed to save process record; killed unmanaged child pid={pid}: {error}"));
    }

    let _ = logs::append(app, app_id, "process", &format!("managed process started pid={pid} command={label}"));

    if let Some(stdout) = child.stdout.take() {
        stream_output(app.clone(), app_id.to_string(), "stdout".to_string(), stdout);
    }
    if let Some(stderr) = child.stderr.take() {
        stream_output(app.clone(), app_id.to_string(), "stderr".to_string(), stderr);
    }

    monitor_process_exit(app.clone(), app_id.to_string(), child, pid);
    thread::sleep(Duration::from_millis(350));

    if !is_pid_running(pid) {
        let _ = clear_record(app, app_id);
        return Err(format!("managed process pid={pid} exited immediately. Check Logs for details."));
    }

    Ok(pid)
}

fn stream_output<R: Read + Send + 'static>(app: AppHandle, app_id: String, category: String, reader: R) {
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let _ = logs::append(&app, &app_id, &category, &line);
                }
                Err(error) => {
                    let _ = logs::append(&app, &app_id, &category, &format!("failed to read process output: {error}"));
                    break;
                }
            }
        }
    });
}

fn monitor_process_exit(app: AppHandle, app_id: String, mut child: Child, pid: u32) {
    thread::spawn(move || match child.wait() {
        Ok(status) => {
            let _ = logs::append(&app, &app_id, "process", &format!("managed process exited pid={pid} status={status}"));

            // If stop_managed cleared process.json before killing the process, this was an intentional stop.
            // Do not race against the command path and overwrite `stopped` with `error`.
            match load_record(&app, &app_id) {
                Ok(Some(record)) if record.pid == pid => {
                    let _ = clear_record(&app, &app_id);
                    if status.success() {
                        let _ = status_store::mark_stopped(&app, &app_id);
                    } else {
                        let _ = status_store::mark_error(&app, &app_id, format!("Managed process pid={pid} exited with status {status}"));
                    }
                }
                _ => {
                    let _ = logs::append(&app, &app_id, "process", &format!("managed process pid={pid} exit already reconciled"));
                }
            }
        }
        Err(error) => {
            let _ = logs::append(&app, &app_id, "process", &format!("failed to wait on managed process pid={pid}: {error}"));
        }
    });
}

pub fn stop_managed(app: &AppHandle, app_id: &str) -> Result<String, String> {
    let Some(record) = load_record(app, app_id)? else {
        logs::append(app, app_id, "process", "no OpenNest-managed process record found; external processes were not touched")?;
        return Ok("No OpenNest-managed process is recorded for this app. External processes were not touched.".to_string());
    };

    if !is_pid_running(record.pid) {
        clear_record(app, app_id)?;
        logs::append(app, app_id, "process", &format!("recorded pid={} is not running; cleared stale process record", record.pid))?;
        return Ok(format!("Recorded process pid={} was already stopped.", record.pid));
    }

    // Clear first so the monitor thread knows this exit was intentional.
    clear_record(app, app_id)?;

    if let Err(error) = kill_pid(record.pid) {
        let _ = save_record(app, &record);
        return Err(error);
    }

    thread::sleep(Duration::from_millis(500));

    if is_pid_running(record.pid) {
        let _ = save_record(app, &record);
        return Err(format!("Failed to stop managed process pid={}. It still appears to be running.", record.pid));
    }

    logs::append(app, app_id, "process", &format!("managed process stopped pid={}", record.pid))?;
    Ok(format!("Stopped managed process pid={}", record.pid))
}

pub fn reconcile_status(app: &AppHandle, mut status: RecipeStatus) -> RecipeStatus {
    if let Ok(Some(record)) = load_record(app, &status.app_id) {
        status.pid = Some(record.pid);
        if is_pid_running(record.pid) {
            if status.app_id == "openclaw" {
                let port = record.port.unwrap_or(18789);
                let report = healthcheck::check_tcp("127.0.0.1", port, 750);
                status.health_checked_at = Some(report.checked_at.clone());
                if report.ok {
                    status.health_state = Some("healthy".to_string());
                    if status.run_state == "unknown" || status.run_state == "stopped" || status.run_state == "error" || status.run_state == "starting" {
                        status.run_state = "running".to_string();
                    }
                    status.last_error = None;
                    let _ = status_store::save(app, &status);
                    return status;
                }

                status.health_state = Some("unhealthy".to_string());
                status.run_state = "error".to_string();
                status.last_error = Some(format!(
                    "Managed process pid={} is running, but OpenClaw Gateway is not reachable on 127.0.0.1:{}. {}",
                    record.pid,
                    port,
                    report.error.unwrap_or_default()
                ));
                let _ = status_store::save(app, &status);
                return status;
            }

            if status.run_state == "unknown" || status.run_state == "stopped" || status.run_state == "error" {
                status.run_state = "running".to_string();
                status.last_error = None;
                let _ = status_store::save(app, &status);
            }
            return status;
        }

        let _ = clear_record(app, &status.app_id);
        if status.run_state == "running" || status.run_state == "starting" || status.run_state == "stopping" {
            status.pid = None;
            status.run_state = "error".to_string();
            status.last_error = Some(format!("Managed process pid={} is no longer running.", record.pid));
            let _ = status_store::save(app, &status);
        }
        return status;
    }

    if status.pid.is_some() && (status.run_state == "running" || status.run_state == "starting" || status.run_state == "stopping") {
        status.pid = None;
        status.run_state = "error".to_string();
        status.last_error = Some("No process.json record exists for the previously running app.".to_string());
        let _ = status_store::save(app, &status);
    }

    status
}

#[cfg(target_os = "windows")]
pub fn is_pid_running(pid: u32) -> bool {
    let filter = format!("PID eq {pid}");
    let Ok(output) = Command::new("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("No tasks") {
        return false;
    }

    let pid_text = pid.to_string();
    text.lines().any(|line| {
        let cols: Vec<String> = line
            .split(",")
            .map(|part| part.trim().trim_matches('\"').to_string())
            .collect();
        cols.get(1).map(|value| value == &pid_text).unwrap_or(false)
    })
}

#[cfg(not(target_os = "windows"))]
pub fn is_pid_running(pid: u32) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("kill -0 {pid} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn kill_pid(pid: u32) -> Result<(), String> {
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .map_err(|e| format!("failed to start taskkill: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        Err(format!("taskkill failed: {stderr}{stdout}"))
    }
}

#[cfg(not(target_os = "windows"))]
fn kill_pid(pid: u32) -> Result<(), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("kill {pid}"))
        .output()
        .map_err(|e| format!("failed to start kill: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
