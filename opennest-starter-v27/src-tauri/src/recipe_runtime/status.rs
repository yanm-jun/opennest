use serde::{Deserialize, Serialize};

use super::secret_redaction_registry;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    pub category: String,
    pub runtime: String,
    pub ports: Vec<u16>,
    pub featured: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipePortMapping {
    pub host: String,
    pub requested_port: u16,
    pub resolved_port: u16,
    pub changed: bool,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeStatus {
    pub app_id: String,
    pub installed: bool,
    pub install_state: String,
    pub run_state: String,
    #[serde(default)]
    pub install_dir: Option<String>,
    pub dashboard_url: Option<String>,
    #[serde(default)]
    pub effective_port: Option<u16>,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub last_error: Option<String>,
    #[serde(default)]
    pub logs_path: Option<String>,
    pub pid: Option<u32>,
    pub health_state: Option<String>,
    pub health_checked_at: Option<String>,
    pub readiness_state: Option<String>,
    pub readiness_checked_at: Option<String>,
    pub readiness_url: Option<String>,
    pub readiness_status_code: Option<u16>,
    pub readiness_latency_ms: Option<u128>,
    pub node_runtime_source: Option<String>,
    pub node_runtime_version: Option<String>,
    pub node_runtime_path: Option<String>,
    pub npm_path: Option<String>,
    #[serde(default)]
    pub plan_reviewed: bool,
    #[serde(default)]
    pub plan_accepted_at: Option<String>,
    #[serde(default)]
    pub plan_version: Option<String>,
    #[serde(default)]
    pub plan_digest: Option<String>,
    #[serde(default)]
    pub plan_risk_level: Option<String>,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub resource_preflight_state: Option<String>,
    #[serde(default)]
    pub resource_preflight_checked_at: Option<String>,
    #[serde(default)]
    pub resource_preflight_blocking_count: Option<u32>,
    #[serde(default)]
    pub resource_preflight_warning_count: Option<u32>,
    #[serde(default)]
    pub port_resolution_state: Option<String>,
    #[serde(default)]
    pub port_resolution_checked_at: Option<String>,
    #[serde(default)]
    pub port_resolution_message: Option<String>,
    #[serde(default)]
    pub port_mappings: Vec<RecipePortMapping>,
    #[serde(default)]
    pub effective_dashboard_url: Option<String>,
    #[serde(default)]
    pub effective_readiness_url: Option<String>,
    #[serde(default)]
    pub progress_state: Option<String>,
    #[serde(default)]
    pub progress_operation: Option<String>,
    #[serde(default)]
    pub progress_operation_id: Option<String>,
    #[serde(default)]
    pub progress_phase: Option<String>,
    #[serde(default)]
    pub progress_message: Option<String>,
    #[serde(default)]
    pub progress_percent: Option<u8>,
    #[serde(default)]
    pub progress_step: Option<u32>,
    #[serde(default)]
    pub progress_total_steps: Option<u32>,
    #[serde(default)]
    pub progress_started_at: Option<String>,
    #[serde(default)]
    pub progress_updated_at: Option<String>,
    #[serde(default)]
    pub progress_finished_at: Option<String>,
    #[serde(default)]
    pub progress_error: Option<String>,
    #[serde(default)]
    pub runtime_error: Option<RuntimeActionError>,
}

impl RecipeStatus {
    pub fn default_for(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            installed: false,
            install_state: "not_installed".to_string(),
            run_state: "unknown".to_string(),
            install_dir: None,
            dashboard_url: None,
            effective_port: None,
            last_started_at: None,
            last_stopped_at: None,
            last_error: None,
            logs_path: None,
            pid: None,
            health_state: None,
            health_checked_at: None,
            readiness_state: None,
            readiness_checked_at: None,
            readiness_url: None,
            readiness_status_code: None,
            readiness_latency_ms: None,
            node_runtime_source: None,
            node_runtime_version: None,
            node_runtime_path: None,
            npm_path: None,
            plan_reviewed: false,
            plan_accepted_at: None,
            plan_version: None,
            plan_digest: None,
            plan_risk_level: None,
            services: Vec::new(),
            resource_preflight_state: None,
            resource_preflight_checked_at: None,
            resource_preflight_blocking_count: None,
            resource_preflight_warning_count: None,
            port_resolution_state: None,
            port_resolution_checked_at: None,
            port_resolution_message: None,
            port_mappings: Vec::new(),
            effective_dashboard_url: None,
            effective_readiness_url: None,
            progress_state: None,
            progress_operation: None,
            progress_operation_id: None,
            progress_phase: None,
            progress_message: None,
            progress_percent: None,
            progress_step: None,
            progress_total_steps: None,
            progress_started_at: None,
            progress_updated_at: None,
            progress_finished_at: None,
            progress_error: None,
            runtime_error: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActionError {
    pub code: String,
    pub title: String,
    pub message: String,
    pub detail: Option<String>,
    pub likely_cause: Option<String>,
    pub next_action: Option<String>,
    pub repairable: bool,
    pub repair_action: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
}

impl RuntimeActionError {
    pub fn from_message(message: impl Into<String>) -> Self {
        let message = secret_redaction_registry::redact(&message.into());
        let code = infer_error_code(&message);
        let title = infer_title(&code);
        let likely_cause = infer_likely_cause(&code, &message);
        let next_action = infer_next_action(&code, &message);
        let repair_action = infer_repair_action(&code, &message);
        let diagnostics = parse_command_diagnostics(&message);
        Self {
            code: code.clone(),
            title,
            message: infer_user_message(&code, &message),
            detail: Some(message),
            likely_cause,
            next_action,
            repairable: repair_action.is_some(),
            repair_action,
            stdout: diagnostics.stdout,
            stderr: diagnostics.stderr,
            exit_code: diagnostics.exit_code,
        }
    }

    pub fn with_detail(
        code: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
        next_action: Option<String>,
    ) -> Self {
        let code = code.into();
        let raw_message = secret_redaction_registry::redact(&message.into());
        let redacted_detail = detail.map(|value| secret_redaction_registry::redact(&value));
        let combined = redacted_detail.clone().unwrap_or_else(|| raw_message.clone());
        let diagnostics = parse_command_diagnostics(&combined);
        let inferred_next_action = infer_next_action(&code, &combined);
        let repair_action = infer_repair_action(&code, &combined);
        Self {
            code: code.clone(),
            title: infer_title(&code),
            message: raw_message,
            detail: redacted_detail,
            likely_cause: infer_likely_cause(&code, &combined),
            next_action: next_action
                .map(|value| secret_redaction_registry::redact(&value))
                .or(inferred_next_action),
            repairable: repair_action.is_some(),
            repair_action,
            stdout: diagnostics.stdout,
            stderr: diagnostics.stderr,
            exit_code: diagnostics.exit_code,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ParsedCommandDiagnostics {
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: Option<i32>,
}

fn parse_command_diagnostics(message: &str) -> ParsedCommandDiagnostics {
    ParsedCommandDiagnostics {
        stdout: parse_labeled_block(message, "stdout"),
        stderr: parse_labeled_block(message, "stderr"),
        exit_code: parse_exit_code(message),
    }
}

fn parse_labeled_block(message: &str, label: &str) -> Option<String> {
    let marker = format!("\n {label}:");
    let alt_marker = format!("\n{label}:");
    let start = message.find(&marker)
        .map(|index| index + marker.len())
        .or_else(|| message.find(&alt_marker).map(|index| index + alt_marker.len()))?;
    let tail = &message[start..];
    let end = tail.find("\n stderr:")
        .or_else(|| tail.find("\n stdout:"))
        .unwrap_or(tail.len());
    let value = tail[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn parse_exit_code(message: &str) -> Option<i32> {
    let marker = "exit_code=Some(";
    let start = message.find(marker)? + marker.len();
    let rest = &message[start..];
    let end = rest.find(')')?;
    rest[..end].trim().parse::<i32>().ok()
}

fn infer_error_code(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("node") && lower.contains("not supported") {
        "NODE_UNSUPPORTED".to_string()
    } else if lower.contains("node") && lower.contains("not found") {
        "NODE_MISSING".to_string()
    } else if lower.contains("docker is installed, but docker desktop / daemon is not ready")
        || lower.contains("docker desktop and wait until it is running")
        || lower.contains("docker daemon")
        || lower.contains("cannot connect to the docker daemon")
    {
        "DOCKER_DAEMON_NOT_READY".to_string()
    } else if lower.contains("docker compose version") || (lower.contains("docker compose") && lower.contains("not found")) {
        "DOCKER_COMPOSE_MISSING".to_string()
    } else if lower.contains("docker --version") || (lower.contains("docker") && lower.contains("not found")) {
        "DOCKER_MISSING".to_string()
    } else if lower.contains("docker compose pull") {
        "DOCKER_PULL_FAILED".to_string()
    } else if lower.contains("docker compose up") {
        "DOCKER_START_FAILED".to_string()
    } else if lower.contains("docker compose ps") {
        "DOCKER_STATUS_FAILED".to_string()
    } else if lower.contains("docker compose stop") {
        "DOCKER_STOP_FAILED".to_string()
    } else if lower.contains("address already in use")
        || lower.contains("port 3000")
        || lower.contains("port 3001")
        || lower.contains("port 80")
        || lower.contains("requested ports are busy")
        || lower.contains("port conflict")
    {
        "PORT_CONFLICT".to_string()
    } else if lower.contains("binary was not found at") || lower.contains("openclaw binary was not found") {
        "OPENCLAW_CLI_NOT_FOUND".to_string()
    } else if lower.contains("run official onboarding") || lower.contains("onboarding first") {
        "OPENCLAW_ONBOARDING_REQUIRED".to_string()
    } else if lower.contains("npm install") {
        "OPENCLAW_INSTALL_FAILED".to_string()
    } else if lower.contains("onboard") {
        "OPENCLAW_ONBOARDING_FAILED".to_string()
    } else if lower.contains("gateway start") {
        "OPENCLAW_GATEWAY_START_FAILED".to_string()
    } else if lower.contains("gateway status") {
        "OPENCLAW_GATEWAY_STATUS_FAILED".to_string()
    } else if lower.contains("gateway probe") {
        "OPENCLAW_GATEWAY_PROBE_FAILED".to_string()
    } else if lower.contains("127.0.0.1:18789") || lower.contains("did not become reachable") {
        "OPENCLAW_PORT_UNREACHABLE".to_string()
    } else if lower.contains("dashboard") {
        "OPENCLAW_DASHBOARD_FAILED".to_string()
    } else if lower.contains("http-ready") || lower.contains("http-ready yet") || lower.contains("not fully ready yet") {
        "DASHBOARD_READINESS_FAILED".to_string()
    } else if lower.contains("timed out") {
        "COMMAND_TIMEOUT".to_string()
    } else if lower.contains("not reachable") || lower.contains("refused") {
        "LOCAL_ENDPOINT_UNREACHABLE".to_string()
    } else if lower.contains("install plan") {
        "INSTALL_PLAN_REQUIRED".to_string()
    } else {
        "RUNTIME_ERROR".to_string()
    }
}

fn infer_title(code: &str) -> String {
    match code {
        "NODE_UNSUPPORTED" => "Node 版本不符合要求".to_string(),
        "NODE_MISSING" => "缺少 Node 运行时".to_string(),
        "DOCKER_MISSING" => "未安装 Docker".to_string(),
        "DOCKER_DAEMON_NOT_READY" => "Docker Desktop 未启动".to_string(),
        "DOCKER_COMPOSE_MISSING" => "docker compose 不可用".to_string(),
        "DOCKER_PULL_FAILED" => "Docker 镜像拉取失败".to_string(),
        "DOCKER_START_FAILED" => "Docker 容器启动失败".to_string(),
        "DOCKER_STATUS_FAILED" => "Docker 状态检查失败".to_string(),
        "DOCKER_STOP_FAILED" => "Docker 停止失败".to_string(),
        "PORT_CONFLICT" => "端口被占用".to_string(),
        "OPENCLAW_CLI_NOT_FOUND" => "OpenClaw CLI 丢失".to_string(),
        "OPENCLAW_INSTALL_FAILED" => "OpenClaw 安装失败".to_string(),
        "OPENCLAW_ONBOARDING_REQUIRED" => "缺少 OpenClaw onboarding".to_string(),
        "OPENCLAW_ONBOARDING_FAILED" => "OpenClaw onboarding 失败".to_string(),
        "OPENCLAW_GATEWAY_START_FAILED" => "OpenClaw Gateway 启动失败".to_string(),
        "OPENCLAW_GATEWAY_STATUS_FAILED" => "OpenClaw Gateway 状态检查失败".to_string(),
        "OPENCLAW_GATEWAY_PROBE_FAILED" => "OpenClaw Gateway Probe 失败".to_string(),
        "OPENCLAW_PORT_UNREACHABLE" => "OpenClaw 本地聊天服务不可达".to_string(),
        "OPENCLAW_DASHBOARD_FAILED" => "OpenClaw Desktop 无法打开".to_string(),
        "DASHBOARD_READINESS_FAILED" => "应用已启动但未就绪".to_string(),
        "LOCAL_ENDPOINT_UNREACHABLE" => "本地服务不可达".to_string(),
        "INSTALL_PLAN_REQUIRED" => "未通过安装确认".to_string(),
        "COMMAND_TIMEOUT" => "命令执行超时".to_string(),
        _ => "运行时错误".to_string(),
    }
}

fn infer_user_message(code: &str, raw_message: &str) -> String {
    match code {
        "NODE_UNSUPPORTED" => "当前 Node 版本太低，无法满足 OpenNest 运行要求。".to_string(),
        "NODE_MISSING" => "当前机器没有检测到可用的 Node/npm。".to_string(),
        "DOCKER_MISSING" => "当前机器没有安装 Docker Desktop。".to_string(),
        "DOCKER_DAEMON_NOT_READY" => "Docker Desktop 已安装，但后端引擎没有准备好。".to_string(),
        "DOCKER_COMPOSE_MISSING" => "当前机器无法使用 `docker compose`。".to_string(),
        "DOCKER_PULL_FAILED" => "Docker 镜像拉取失败。".to_string(),
        "DOCKER_START_FAILED" => "Docker 容器启动命令执行失败。".to_string(),
        "DOCKER_STATUS_FAILED" => "Docker 容器状态读取失败。".to_string(),
        "DOCKER_STOP_FAILED" => "Docker 容器停止失败。".to_string(),
        "PORT_CONFLICT" => "应用所需端口已被占用。".to_string(),
        "OPENCLAW_CLI_NOT_FOUND" => "OpenClaw 安装完成后没有找到可执行入口。".to_string(),
        "OPENCLAW_INSTALL_FAILED" => "OpenClaw CLI 安装失败。".to_string(),
        "OPENCLAW_ONBOARDING_REQUIRED" => "OpenClaw 还没有完成官方 onboarding。".to_string(),
        "OPENCLAW_ONBOARDING_FAILED" => "OpenClaw 官方 onboarding 执行失败。".to_string(),
        "OPENCLAW_GATEWAY_START_FAILED" => "OpenClaw Gateway 启动失败。".to_string(),
        "OPENCLAW_GATEWAY_STATUS_FAILED" => "OpenClaw Gateway 状态检查失败。".to_string(),
        "OPENCLAW_GATEWAY_PROBE_FAILED" => "OpenClaw Gateway Probe 失败。".to_string(),
        "OPENCLAW_PORT_UNREACHABLE" => "OpenClaw 本地聊天服务当前不可达。".to_string(),
        "OPENCLAW_DASHBOARD_FAILED" => "OpenClaw Desktop 当前无法打开。".to_string(),
        "DASHBOARD_READINESS_FAILED" => "应用虽然启动了，但本地页面还没有达到可访问状态。".to_string(),
        "LOCAL_ENDPOINT_UNREACHABLE" => "本地服务端口当前不可达。".to_string(),
        "INSTALL_PLAN_REQUIRED" => "当前安装计划还没有被接受。".to_string(),
        "COMMAND_TIMEOUT" => "命令执行超过超时时间。".to_string(),
        _ => raw_message.to_string(),
    }
}

fn infer_likely_cause(code: &str, _message: &str) -> Option<String> {
    let cause = match code {
        "NODE_UNSUPPORTED" => "系统 Node 版本低于项目要求，或 npm 与 Node 版本不匹配。",
        "NODE_MISSING" => "系统没有 Node/npm，或 OpenNest 托管运行时还未准备好。",
        "DOCKER_MISSING" => "这台机器没有安装 Docker Desktop。",
        "DOCKER_DAEMON_NOT_READY" => "Docker Desktop UI 已打开，但 WSL2/虚拟化后端没有真正启动。",
        "DOCKER_COMPOSE_MISSING" => "Docker CLI 存在，但 compose 插件不可用或版本异常。",
        "DOCKER_PULL_FAILED" => "Docker daemon、网络、镜像仓库访问或镜像标签存在问题。",
        "DOCKER_START_FAILED" => "compose 文件、端口映射、镜像拉取结果或 Docker backend 有问题。",
        "DOCKER_STATUS_FAILED" => "容器状态查询失败，常见于 Docker backend 不稳定或 compose 工程损坏。",
        "DOCKER_STOP_FAILED" => "Docker backend 异常，或容器状态与 compose 记录不一致。",
        "PORT_CONFLICT" => "3000 / 3001 / 80 / 18789 等本地端口已被其他程序占用。",
        "OPENCLAW_CLI_NOT_FOUND" => "npm 安装完成了，但入口文件没有生成到预期位置，或文件路径已被移动。",
        "OPENCLAW_INSTALL_FAILED" => "npm 下载失败、网络受限、registry 异常，或 Node/npm 环境不稳定。",
        "OPENCLAW_ONBOARDING_REQUIRED" => "OpenClaw 已安装，但官方 onboarding 还没完成，所以 gateway 无法稳定工作。",
        "OPENCLAW_ONBOARDING_FAILED" => "官方 onboarding 中断、需要用户交互未完成，或环境依赖未准备好。",
        "OPENCLAW_GATEWAY_START_FAILED" => "onboarding 未完成、配置缺失、端口冲突或 gateway 自身报错。",
        "OPENCLAW_GATEWAY_STATUS_FAILED" => "gateway 没有正常启动，或者 CLI 无法读取当前状态。",
        "OPENCLAW_GATEWAY_PROBE_FAILED" => "gateway 已启动但本地服务没有成功响应 probe。",
        "OPENCLAW_PORT_UNREACHABLE" => "gateway 没有真正启动，或本地聊天服务没有成功监听。",
        "OPENCLAW_DASHBOARD_FAILED" => "gateway 未运行、本地聊天界面未就绪，或嵌入式应用窗口打开失败。",
        "DASHBOARD_READINESS_FAILED" => "容器或本地服务虽然起来了，但 HTTP 页面还没 ready。",
        "LOCAL_ENDPOINT_UNREACHABLE" => "本地 TCP 端口没有监听，或服务已经异常退出。",
        "INSTALL_PLAN_REQUIRED" => "用户还没有确认当前安装计划。",
        "COMMAND_TIMEOUT" => "命令执行时间过长，可能卡在网络、子进程等待或外部依赖。",
        _ => return None,
    };
    Some(cause.to_string())
}

fn infer_next_action(code: &str, message: &str) -> Option<String> {
    let action = match code {
        "NODE_UNSUPPORTED" | "NODE_MISSING" => "Run Repair or install Node 24+, then run Check Environment again.",
        "DOCKER_MISSING" => "Install Docker Desktop, then run Check Environment again.",
        "DOCKER_COMPOSE_MISSING" => "Update Docker Desktop so `docker compose` is available, then retry Check Environment.",
        "DOCKER_DAEMON_NOT_READY" => "Open Docker Desktop and wait for the engine to finish starting before retrying.",
        "DOCKER_PULL_FAILED" => "Review docker pull output in Logs, confirm Docker Desktop is healthy and registry access works, then retry Start.",
        "DOCKER_START_FAILED" => "Review docker compose up output in Logs, fix the Docker or port issue, then retry Start.",
        "DOCKER_STATUS_FAILED" => "Retry Container Status after Start. If it still fails, inspect Logs and Docker Desktop.",
        "DOCKER_STOP_FAILED" => "Review docker compose stop output in Logs, then retry Stop or stop the container in Docker Desktop.",
        "PORT_CONFLICT" => "Run Resolve Ports, review the remapped port in Status, then retry Start and Open App.",
        "OPENCLAW_CLI_NOT_FOUND" => "Run Check Environment, then Repair. If the CLI still cannot be found, reinstall OpenClaw.",
        "OPENCLAW_INSTALL_FAILED" => "Review npm output in Logs, confirm network access to registry.npmjs.org, then retry Install.",
        "OPENCLAW_ONBOARDING_REQUIRED" => "Run Official Onboarding, complete it, then retry Start.",
        "OPENCLAW_ONBOARDING_FAILED" => "Run Official Onboarding again and complete the terminal flow before starting the gateway.",
        "OPENCLAW_GATEWAY_START_FAILED" => "Run Official Onboarding first, then retry Start and inspect gateway logs.",
        "OPENCLAW_GATEWAY_STATUS_FAILED" => "Retry Gateway Status after Start. If it still fails, inspect Logs and run Doctor.",
        "OPENCLAW_GATEWAY_PROBE_FAILED" | "LOCAL_ENDPOINT_UNREACHABLE" | "OPENCLAW_PORT_UNREACHABLE" => "Retry Start, then run Check Health again. If it still fails, run Doctor and inspect Logs.",
        "OPENCLAW_DASHBOARD_FAILED" => "Start the gateway first, then retry Open App.",
        "DASHBOARD_READINESS_FAILED" => "Run Check Readiness again. If it still fails, inspect Logs, ports, and the resolved app URL.",
        "INSTALL_PLAN_REQUIRED" => "Accept the current install plan before running Install.",
        "COMMAND_TIMEOUT" => "Inspect Logs for partial output and retry the action after confirming prerequisites.",
        _ => {
            if message.is_empty() {
                return None;
            }
            "Inspect Logs for detail, then retry the action with prerequisites satisfied."
        }
    };
    Some(action.to_string())
}

fn infer_repair_action(code: &str, message: &str) -> Option<String> {
    let action = match code {
        "NODE_UNSUPPORTED" | "NODE_MISSING" => "recheck-environment",
        "OPENCLAW_CLI_NOT_FOUND" => "relocate-openclaw-cli",
        "OPENCLAW_ONBOARDING_REQUIRED" | "OPENCLAW_ONBOARDING_FAILED" => "rerun-onboarding",
        "OPENCLAW_GATEWAY_START_FAILED" => "rerun-onboarding",
        "OPENCLAW_GATEWAY_PROBE_FAILED" | "OPENCLAW_PORT_UNREACHABLE" | "LOCAL_ENDPOINT_UNREACHABLE" => "rerun-probe",
        "OPENCLAW_DASHBOARD_FAILED" => "reopen-dashboard",
        "DOCKER_DAEMON_NOT_READY" | "DOCKER_COMPOSE_MISSING" => "recheck-docker",
        "PORT_CONFLICT" => "resolve-ports",
        "DOCKER_PULL_FAILED" | "DOCKER_START_FAILED" => {
            if message.to_ascii_lowercase().contains("port") {
                "resolve-ports"
            } else {
                "rewrite-compose"
            }
        }
        "DASHBOARD_READINESS_FAILED" => "rewrite-compose",
        _ => return None,
    };
    Some(action.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActionResult {
    pub ok: bool,
    pub app_id: String,
    pub status: Option<RecipeStatus>,
    pub message: Option<String>,
    pub logs: Option<Vec<String>>,
    pub error: Option<RuntimeActionError>,
}

impl RuntimeActionResult {
    pub fn ok(app_id: &str, message: impl Into<String>, status: Option<RecipeStatus>) -> Self {
        Self { ok: true, app_id: app_id.to_string(), status, message: Some(message.into()), logs: None, error: None }
    }

    pub fn fail(app_id: &str, error: impl Into<String>, status: Option<RecipeStatus>) -> Self {
        Self {
            ok: false,
            app_id: app_id.to_string(),
            status,
            message: None,
            logs: None,
            error: Some(RuntimeActionError::from_message(error)),
        }
    }

    pub fn fail_with_detail(
        app_id: &str,
        code: impl Into<String>,
        message: impl Into<String>,
        detail: Option<String>,
        next_action: Option<String>,
        status: Option<RecipeStatus>,
    ) -> Self {
        Self {
            ok: false,
            app_id: app_id.to_string(),
            status,
            message: None,
            logs: None,
            error: Some(RuntimeActionError::with_detail(code, message, detail, next_action)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSecretInput {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawSetupInput {
    pub provider: String,
    pub api_key: String,
    #[serde(default)]
    pub open_chat: bool,
}
