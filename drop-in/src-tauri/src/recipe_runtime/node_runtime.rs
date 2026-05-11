use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::AppHandle;

use super::{command_timeout, logs, paths};

const MIN_NODE_MAJOR_LTS: u64 = 22;
const MIN_NODE_MINOR_LTS: u64 = 16;
const PREFERRED_NODE_MAJOR: u64 = 24;

#[derive(Debug, Clone)]
pub struct NodeRuntime {
    pub source: String,
    pub version: String,
    pub bin_dir: PathBuf,
    pub node_path: PathBuf,
    pub npm_path: PathBuf,
}

impl NodeRuntime {
    pub fn path_env(&self) -> OsString {
        let existing = std::env::var_os("PATH").unwrap_or_default();
        if self.source == "system" {
            return existing;
        }
        std::env::join_paths(std::iter::once(self.bin_dir.clone()).chain(std::env::split_paths(&existing)))
            .unwrap_or_else(|_| existing)
    }

    pub fn describe(&self) -> String {
        format!(
            "Node runtime source={} version={} node={} npm={}",
            self.source,
            self.version,
            self.node_path.display(),
            self.npm_path.display()
        )
    }
}

#[derive(Debug, Clone)]
pub struct NodeRuntimeInspection {
    pub usable: bool,
    pub source: Option<String>,
    pub version: Option<String>,
    pub node_path: Option<String>,
    pub npm_path: Option<String>,
    pub message: String,
}

pub fn runtime_root(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = paths::root_dir(app)?.join("_runtime").join("node");
    fs::create_dir_all(&dir).map_err(|error| format!("failed to create node runtime dir: {error}"))?;
    Ok(dir)
}

fn run_version(command: &Path) -> Result<String, String> {
    let output = Command::new(command)
        .arg("--version")
        .output()
        .map_err(|error| format!("failed to run {} --version: {error}", command.display()))?;
    if !output.status.success() {
        return Err(format!(
            "{} --version failed: {}{}",
            command.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_system_version(command: &str) -> Result<String, String> {
    let output = Command::new(command)
        .arg("--version")
        .output()
        .map_err(|error| format!("failed to run {command} --version: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command} --version failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_node_version(version: &str) -> Option<(u64, u64, u64)> {
    let cleaned = version.trim().trim_start_matches('v');
    let mut parts = cleaned.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    Some((major, minor, patch))
}

pub fn is_supported_node_version(version: &str) -> bool {
    match parse_node_version(version) {
        Some((major, minor, _)) if major >= PREFERRED_NODE_MAJOR => true,
        Some((major, minor, _)) if major == MIN_NODE_MAJOR_LTS && minor >= MIN_NODE_MINOR_LTS => true,
        _ => false,
    }
}

fn path_from_program(program: &str) -> PathBuf {
    PathBuf::from(program)
}

fn system_runtime() -> Result<NodeRuntime, String> {
    let node_version = run_system_version("node")?;
    if !is_supported_node_version(&node_version) {
        return Err(format!(
            "system Node version {node_version} is not supported. Required: Node 24+ or Node 22.16+."
        ));
    }

    let _npm_version = run_system_version("npm")?;
    Ok(NodeRuntime {
        source: "system".to_string(),
        version: node_version,
        bin_dir: PathBuf::from("."),
        node_path: path_from_program("node"),
        npm_path: path_from_program("npm"),
    })
}

fn managed_runtime(app: &AppHandle) -> Result<NodeRuntime, String> {
    let root = runtime_root(app)?;
    let entries = fs::read_dir(&root).map_err(|error| format!("failed to read node runtime dir: {error}"))?;
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with("node-v") && name.contains("win-x64")).unwrap_or(false))
        .collect();
    candidates.sort();
    candidates.reverse();

    for bin_dir in candidates {
        let node_path = bin_dir.join("node.exe");
        let npm_path = bin_dir.join("npm.cmd");
        if !node_path.exists() || !npm_path.exists() {
            continue;
        }
        let version = run_version(&node_path)?;
        if is_supported_node_version(&version) {
            return Ok(NodeRuntime {
                source: "managed".to_string(),
                version,
                bin_dir,
                node_path,
                npm_path,
            });
        }
    }

    Err("managed Node runtime was not found or is not supported".to_string())
}

pub fn inspect_runtime(app: &AppHandle) -> NodeRuntimeInspection {
    if let Ok(runtime) = system_runtime() {
        return NodeRuntimeInspection {
            usable: true,
            source: Some(runtime.source),
            version: Some(runtime.version),
            node_path: Some(runtime.node_path.display().to_string()),
            npm_path: Some(runtime.npm_path.display().to_string()),
            message: "Using supported system Node runtime.".to_string(),
        };
    }

    if let Ok(runtime) = managed_runtime(app) {
        return NodeRuntimeInspection {
            usable: true,
            source: Some(runtime.source),
            version: Some(runtime.version),
            node_path: Some(runtime.node_path.display().to_string()),
            npm_path: Some(runtime.npm_path.display().to_string()),
            message: "Using OpenNest managed Node runtime.".to_string(),
        };
    }

    NodeRuntimeInspection {
        usable: false,
        source: None,
        version: None,
        node_path: None,
        npm_path: None,
        message: "No supported Node runtime found. Run Repair to prepare managed Node.".to_string(),
    }
}

fn powershell_escape(value: &Path) -> String {
    value.to_string_lossy().replace('\'', "''")
}

#[cfg(target_os = "windows")]
pub fn prepare_managed_node(app: &AppHandle, app_id: &str) -> Result<NodeRuntime, String> {
    if let Ok(runtime) = managed_runtime(app) {
        logs::append(app, app_id, "node", &format!("managed Node already available: {}", runtime.describe()))?;
        return Ok(runtime);
    }

    let root = runtime_root(app)?;
    let root_escaped = powershell_escape(&root);
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$root = '{root_escaped}'
New-Item -ItemType Directory -Force -Path $root | Out-Null
$index = Invoke-RestMethod -Uri 'https://nodejs.org/dist/index.json'
$release = $index | Where-Object {{ $_.version -like 'v24.*' -and $_.files -contains 'win-x64-zip' }} | Select-Object -First 1
if (-not $release) {{ throw 'No Node 24 win-x64 zip release found in Node distribution index.' }}
$version = $release.version
$zip = Join-Path $root ("node-$version-win-x64.zip")
$url = "https://nodejs.org/dist/$version/node-$version-win-x64.zip"
Write-Output "Downloading $url"
Invoke-WebRequest -Uri $url -OutFile $zip
Write-Output "Extracting $zip"
Expand-Archive -Path $zip -DestinationPath $root -Force
Write-Output "Prepared $version"
"#
    );

    logs::append(app, app_id, "node", "preparing managed Node runtime from nodejs.org distribution index")?;
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script]);
    let output = command_timeout::run_with_timeout(command, command_timeout::NODE_RUNTIME_TIMEOUT_MS)
        .map_err(|error| format!("managed Node preparation failed to start or wait: {error}"))?;
    logs::append(app, app_id, "node", &format!("node runtime prepare finished duration_ms={} timed_out={} exit_code={:?}", output.duration_ms, output.timed_out, output.exit_code))?;
    logs::append(app, app_id, "node", &output.stdout)?;
    logs::append(app, app_id, "node", &output.stderr)?;

    if !output.success {
        return Err(output.failure_message("prepare managed Node runtime"));
    }

    managed_runtime(app)
}

#[cfg(not(target_os = "windows"))]
pub fn prepare_managed_node(_app: &AppHandle, _app_id: &str) -> Result<NodeRuntime, String> {
    Err("Managed portable Node runtime preparation is currently implemented for Windows only. Install Node 24+ or Node 22.16+ on this system.".to_string())
}

pub fn ensure_node_runtime(app: &AppHandle, app_id: &str) -> Result<NodeRuntime, String> {
    match system_runtime() {
        Ok(runtime) => {
            logs::append(app, app_id, "node", &format!("using system runtime: {}", runtime.describe()))?;
            return Ok(runtime);
        }
        Err(error) => {
            logs::append(app, app_id, "node", &format!("system Node unavailable or unsupported: {error}"))?;
        }
    }

    match managed_runtime(app) {
        Ok(runtime) => {
            logs::append(app, app_id, "node", &format!("using managed runtime: {}", runtime.describe()))?;
            Ok(runtime)
        }
        Err(error) => {
            logs::append(app, app_id, "node", &format!("managed Node unavailable: {error}; preparing runtime"))?;
            prepare_managed_node(app, app_id)
        }
    }
}
