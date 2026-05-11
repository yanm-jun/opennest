use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use super::{command_timeout, healthcheck, logs, node_runtime, paths, port_resolver, recipe_loader, status_store};
use super::recipe_loader::OpenNestRecipe;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePreflightCheck {
    pub id: String,
    pub label: String,
    pub status: String,
    pub required: bool,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePreflightReport {
    pub app_id: String,
    pub checked_at: String,
    pub ok: bool,
    pub blocking_count: u32,
    pub warning_count: u32,
    pub summary: String,
    pub checks: Vec<ResourcePreflightCheck>,
}

fn pass(id: impl Into<String>, label: impl Into<String>, required: bool, message: impl Into<String>, details: Option<String>) -> ResourcePreflightCheck {
    ResourcePreflightCheck { id: id.into(), label: label.into(), status: "pass".to_string(), required, message: message.into(), details }
}

fn warn(id: impl Into<String>, label: impl Into<String>, required: bool, message: impl Into<String>, details: Option<String>) -> ResourcePreflightCheck {
    ResourcePreflightCheck { id: id.into(), label: label.into(), status: "warning".to_string(), required, message: message.into(), details }
}

fn fail(id: impl Into<String>, label: impl Into<String>, required: bool, message: impl Into<String>, details: Option<String>) -> ResourcePreflightCheck {
    ResourcePreflightCheck { id: id.into(), label: label.into(), status: "error".to_string(), required, message: message.into(), details }
}

fn bool_requirement(recipe: &OpenNestRecipe, key: &str) -> bool {
    recipe.requirements.as_ref()
        .and_then(|requirements| requirements.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn u64_requirement(recipe: &OpenNestRecipe, key: &str) -> Option<u64> {
    recipe.requirements.as_ref()
        .and_then(|requirements| requirements.get(key))
        .and_then(Value::as_u64)
}

fn os_requirement(recipe: &OpenNestRecipe) -> Vec<String> {
    recipe.requirements.as_ref()
        .and_then(|requirements| requirements.get("os"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn app_disk_estimate_gb(recipe: &OpenNestRecipe) -> u64 {
    match recipe.id.as_str() {
        "openclaw" => 2,
        "open-webui" => 10,
        "flowise" => 5,
        "dify" => 25,
        _ => 5,
    }
}

fn required_network_hosts(recipe: &OpenNestRecipe) -> Vec<(&'static str, u16)> {
    match recipe.id.as_str() {
        "openclaw" => vec![("registry.npmjs.org", 443), ("nodejs.org", 443)],
        "open-webui" => vec![("ghcr.io", 443)],
        "flowise" => vec![("registry-1.docker.io", 443), ("auth.docker.io", 443)],
        "dify" => vec![("github.com", 443), ("registry-1.docker.io", 443), ("auth.docker.io", 443)],
        _ => vec![("github.com", 443)],
    }
}

fn can_connect(host: &str, port: u16, timeout_ms: u64) -> Result<(), String> {
    let address = format!("{host}:{port}");
    let mut addrs = address.to_socket_addrs().map_err(|error| format!("DNS lookup failed for {address}: {error}"))?;
    let socket = addrs.next().ok_or_else(|| format!("DNS lookup returned no addresses for {address}"))?;
    TcpStream::connect_timeout(&socket, Duration::from_millis(timeout_ms))
        .map(|_| ())
        .map_err(|error| format!("cannot connect to {address}: {error}"))
}

fn timed_command(mut command: Command, label: &str, timeout_ms: u64) -> Result<String, String> {
    let output = command_timeout::run_with_timeout(command, timeout_ms)
        .map_err(|error| format!("{label} failed to spawn or wait: {error}"))?;
    if output.success {
        let text = format!("{}{}", output.stdout.trim(), output.stderr.trim());
        Ok(text.chars().take(500).collect())
    } else {
        Err(output.failure_message(label))
    }
}

fn check_os(recipe: &OpenNestRecipe) -> ResourcePreflightCheck {
    let required = os_requirement(recipe);
    if required.is_empty() {
        return pass("os", "Operating system", false, "No strict OS requirement declared by recipe.", None);
    }

    let current = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    if required.iter().any(|item| item == current) {
        pass("os", "Operating system", true, format!("Current OS satisfies recipe requirement: {current}."), Some(format!("required={}", required.join(", "))))
    } else {
        fail("os", "Operating system", true, format!("Recipe requires {}, but current OS is {current}.", required.join(", ")), None)
    }
}

fn check_workspace(app: &AppHandle, recipe: &OpenNestRecipe) -> ResourcePreflightCheck {
    match paths::app_dir(app, &recipe.id) {
        Ok(dir) => {
            let probe = dir.join(".opennest-preflight-write-test");
            match fs::write(&probe, b"ok") {
                Ok(_) => {
                    let _ = fs::remove_file(&probe);
                    pass("workspace", "Workspace write access", true, "OpenNest app workspace is writable.", Some(dir.display().to_string()))
                }
                Err(error) => fail("workspace", "Workspace write access", true, format!("Cannot write to app workspace: {error}"), Some(dir.display().to_string())),
            }
        }
        Err(error) => fail("workspace", "Workspace write access", true, format!("Cannot resolve app workspace: {error}"), None),
    }
}

#[cfg(target_os = "windows")]
fn free_disk_bytes_for(path: &Path) -> Result<u64, String> {
    let path_text = path.display().to_string();
    let drive = path_text.chars().take(2).collect::<String>();
    if !drive.ends_with(':') {
        return Err(format!("cannot resolve Windows drive from path {path_text}"));
    }
    let drive_name = drive.trim_end_matches(':');
    let script = format!("(Get-PSDrive -Name '{drive_name}').Free");
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script]);
    let text = timed_command(command, "powershell disk free check", command_timeout::CHECK_TIMEOUT_MS)?;
    text.trim().parse::<u64>().map_err(|error| format!("failed to parse free disk bytes from PowerShell output {text:?}: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn free_disk_bytes_for(_path: &Path) -> Result<u64, String> {
    Err("disk free check is currently implemented for Windows only in this adapter".to_string())
}

fn check_disk(app: &AppHandle, recipe: &OpenNestRecipe) -> ResourcePreflightCheck {
    let app_dir = match paths::app_dir(app, &recipe.id) {
        Ok(dir) => dir,
        Err(error) => return warn("disk", "Disk space", true, format!("Cannot resolve app dir for disk check: {error}"), None),
    };
    let required_gb = app_disk_estimate_gb(recipe);
    match free_disk_bytes_for(&app_dir) {
        Ok(bytes) => {
            let free_gb = bytes / 1024 / 1024 / 1024;
            if free_gb >= required_gb {
                pass("disk", "Disk space", true, format!("Free disk looks sufficient: {free_gb} GB available."), Some(format!("estimated required: {required_gb} GB")))
            } else {
                fail("disk", "Disk space", true, format!("Only {free_gb} GB free; estimated requirement is {required_gb} GB."), Some(app_dir.display().to_string()))
            }
        }
        Err(error) => warn("disk", "Disk space", true, format!("Could not verify free disk space: {error}"), Some(format!("estimated required: {required_gb} GB"))),
    }
}

#[cfg(target_os = "windows")]
fn total_memory_gb() -> Result<u64, String> {
    let script = "[math]::Floor((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB)";
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script]);
    let text = timed_command(command, "powershell memory check", command_timeout::CHECK_TIMEOUT_MS)?;
    text.trim().parse::<u64>().map_err(|error| format!("failed to parse total memory GB from PowerShell output {text:?}: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn total_memory_gb() -> Result<u64, String> {
    Err("memory check is currently implemented for Windows only in this adapter".to_string())
}

#[cfg(target_os = "windows")]
fn cpu_logical_count() -> Result<u64, String> {
    let script = "(Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors";
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script]);
    let text = timed_command(command, "powershell cpu check", command_timeout::CHECK_TIMEOUT_MS)?;
    text.trim().parse::<u64>().map_err(|error| format!("failed to parse logical CPU count from PowerShell output {text:?}: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn cpu_logical_count() -> Result<u64, String> {
    Err("CPU check is currently implemented for Windows only in this adapter".to_string())
}

fn check_memory(recipe: &OpenNestRecipe) -> ResourcePreflightCheck {
    let recommended = u64_requirement(recipe, "memoryGbRecommended");
    if recommended.is_none() {
        return pass("memory", "Memory", false, "No recommended memory requirement declared by recipe.", None);
    }
    let recommended = recommended.unwrap();
    match total_memory_gb() {
        Ok(total) if total >= recommended => pass("memory", "Memory", false, format!("System memory looks sufficient: {total} GB available."), Some(format!("recommended: {recommended} GB"))),
        Ok(total) => warn("memory", "Memory", false, format!("System memory is below recommendation: {total} GB available, {recommended} GB recommended."), None),
        Err(error) => warn("memory", "Memory", false, format!("Could not verify system memory: {error}"), Some(format!("recommended: {recommended} GB"))),
    }
}

fn check_cpu(recipe: &OpenNestRecipe) -> ResourcePreflightCheck {
    let recommended = u64_requirement(recipe, "cpuRecommended");
    if recommended.is_none() {
        return pass("cpu", "CPU", false, "No recommended CPU requirement declared by recipe.", None);
    }
    let recommended = recommended.unwrap();
    match cpu_logical_count() {
        Ok(count) if count >= recommended => pass("cpu", "CPU", false, format!("Logical CPU count looks sufficient: {count}."), Some(format!("recommended: {recommended}"))),
        Ok(count) => warn("cpu", "CPU", false, format!("Logical CPU count is below recommendation: {count}; recommended {recommended}."), None),
        Err(error) => warn("cpu", "CPU", false, format!("Could not verify CPU count: {error}"), Some(format!("recommended: {recommended}"))),
    }
}

fn check_node(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<ResourcePreflightCheck> {
    let requires_node = recipe.runtime == "native-cli" || recipe.requirements.as_ref().and_then(|requirements| requirements.get("node")).is_some();
    if !requires_node {
        return None;
    }

    let report = node_runtime::inspect_runtime(app);
    if report.usable {
        Some(pass(
            "node",
            "Node runtime",
            true,
            report.message,
            Some(format!("source={:?}; version={:?}; node={:?}; npm={:?}", report.source, report.version, report.node_path, report.npm_path)),
        ))
    } else if cfg!(target_os = "windows") {
        Some(warn(
            "node",
            "Node runtime",
            true,
            "No supported Node runtime is ready yet. OpenNest can prepare a managed Node runtime during Repair/Install if network access works.",
            Some(report.message),
        ))
    } else {
        Some(fail(
            "node",
            "Node runtime",
            true,
            "No supported Node runtime found, and managed Node preparation is currently Windows-only.",
            Some(report.message),
        ))
    }
}

fn check_docker(recipe: &OpenNestRecipe) -> Option<ResourcePreflightCheck> {
    let requires = recipe.runtime == "docker-compose" || recipe.runtime == "external-compose" || bool_requirement(recipe, "docker");
    if !requires {
        return None;
    }

    let mut compose = Command::new("docker");
    compose.args(["compose", "version"]);
    if let Err(error) = timed_command(compose, "docker compose version", command_timeout::CHECK_TIMEOUT_MS) {
        return Some(fail("docker", "Docker Compose", true, format!("Docker Compose is not available: {error}"), None));
    }

    let mut info = Command::new("docker");
    info.arg("info");
    match timed_command(info, "docker info", command_timeout::CHECK_TIMEOUT_MS) {
        Ok(details) => Some(pass("docker", "Docker Desktop / daemon", true, "Docker Compose is available and Docker daemon responded.", Some(details))),
        Err(error) => Some(fail("docker", "Docker Desktop / daemon", true, format!("Docker daemon is not ready: {error}"), Some("Start Docker Desktop, wait until it is running, then retry.".to_string()))),
    }
}

fn check_git(recipe: &OpenNestRecipe) -> Option<ResourcePreflightCheck> {
    let requires = recipe.runtime == "external-compose" || recipe.install.as_ref().and_then(|install| install.repo.as_ref()).is_some();
    if !requires {
        return None;
    }
    let mut command = Command::new("git");
    command.arg("--version");
    match timed_command(command, "git --version", command_timeout::CHECK_TIMEOUT_MS) {
        Ok(details) => Some(pass("git", "Git", true, "Git is available.", Some(details))),
        Err(error) => Some(fail("git", "Git", true, format!("Git is required but not available: {error}"), None)),
    }
}

fn check_ports(app: &AppHandle, recipe: &OpenNestRecipe) -> Vec<ResourcePreflightCheck> {
    if recipe.ports.is_empty() {
        return Vec::new();
    }

    match port_resolver::resolve_ports(app, recipe, false) {
        Ok(resolution) => {
            let mut checks = Vec::new();
            for mapping in &resolution.mappings {
                if mapping.changed {
                    checks.push(warn(
                        format!("port-{}", mapping.requested_port),
                        format!("Port {}", mapping.requested_port),
                        true,
                        format!(
                            "127.0.0.1:{} is occupied, but OpenNest can remap {} to 127.0.0.1:{}.",
                            mapping.requested_port, recipe.name, mapping.resolved_port
                        ),
                        Some("The final port mapping will be written during Install/Start and reflected in Dashboard/Readiness URLs.".to_string()),
                    ));
                } else {
                    checks.push(pass(
                        format!("port-{}", mapping.requested_port),
                        format!("Port {}", mapping.requested_port),
                        true,
                        format!("127.0.0.1:{} appears available as configured.", mapping.requested_port),
                        None,
                    ));
                }
            }
            if !resolution.ok {
                checks.push(fail(
                    "port-resolution",
                    "Port auto-resolution",
                    true,
                    resolution.message,
                    Some(resolution.warnings.join(" | ")),
                ));
            }
            checks
        }
        Err(error) => vec![fail(
            "port-resolution",
            "Port auto-resolution",
            true,
            format!("Could not evaluate port resolution: {error}"),
            None,
        )],
    }
}

fn check_network(recipe: &OpenNestRecipe) -> Vec<ResourcePreflightCheck> {
    let requires = bool_requirement(recipe, "network") || recipe.version_source.is_some() || recipe.install.as_ref().map(|install| install.package.is_some() || install.repo.is_some() || install.source.is_some()).unwrap_or(false);
    if !requires {
        return vec![pass("network", "Network", false, "No network dependency declared by recipe.", None)];
    }
    required_network_hosts(recipe).into_iter().map(|(host, port)| {
        match can_connect(host, port, 2_500) {
            Ok(_) => pass(format!("network-{host}"), format!("Network: {host}"), true, format!("Can connect to {host}:{port}."), None),
            Err(error) => fail(format!("network-{host}"), format!("Network: {host}"), true, error, Some("Network, DNS, proxy, VPN, firewall, or regional access may block installation.".to_string())),
        }
    }).collect()
}

pub fn run(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<ResourcePreflightReport, String> {
    let mut checks = Vec::new();
    checks.push(check_os(recipe));
    checks.push(check_workspace(app, recipe));
    checks.push(check_disk(app, recipe));
    checks.push(check_memory(recipe));
    checks.push(check_cpu(recipe));
    if let Some(check) = check_node(app, recipe) { checks.push(check); }
    if let Some(check) = check_docker(recipe) { checks.push(check); }
    if let Some(check) = check_git(recipe) { checks.push(check); }
    checks.extend(check_ports(app, recipe));
    checks.extend(check_network(recipe));

    let blocking_count = checks.iter().filter(|check| check.required && check.status == "error").count() as u32;
    let warning_count = checks.iter().filter(|check| check.status == "warning" || (!check.required && check.status == "error")).count() as u32;
    let ok = blocking_count == 0;
    let checked_at = Utc::now().to_rfc3339();
    let summary = if ok {
        if warning_count > 0 {
            format!("Resource preflight passed with {warning_count} warning(s).")
        } else {
            "Resource preflight passed.".to_string()
        }
    } else {
        format!("Resource preflight blocked install/start with {blocking_count} required failure(s) and {warning_count} warning(s).")
    };

    logs::append(app, &recipe.id, "resource-preflight", &summary)?;
    for check in &checks {
        logs::append(
            app,
            &recipe.id,
            "resource-preflight",
            &format!("{} [{} required={}] {}{}", check.label, check.status, check.required, check.message, check.details.as_ref().map(|value| format!(" details={value}")).unwrap_or_default()),
        )?;
    }

    let _ = status_store::mark_resource_preflight(app, &recipe.id, if ok { "passed" } else { "blocked" }, blocking_count, warning_count, checked_at.clone());

    Ok(ResourcePreflightReport {
        app_id: recipe.id.clone(),
        checked_at,
        ok,
        blocking_count,
        warning_count,
        summary,
        checks,
    })
}

pub fn summarize_blockers(report: &ResourcePreflightReport) -> String {
    let blockers: Vec<String> = report.checks.iter()
        .filter(|check| check.required && check.status == "error")
        .map(|check| format!("{}: {}", check.label, check.message))
        .collect();
    if blockers.is_empty() {
        report.summary.clone()
    } else {
        format!("{}\n{}", report.summary, blockers.join("\n"))
    }
}
