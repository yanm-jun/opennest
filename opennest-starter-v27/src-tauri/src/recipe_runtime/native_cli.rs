use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;
use rand::{distributions::Alphanumeric, Rng};

use super::status::RuntimeActionError;
use super::{command_timeout, healthcheck, logs, node_runtime, paths, process_manager, token_store};

#[derive(Debug, Clone)]
pub struct OpenClawEnvironmentReport {
    pub node_source: Option<String>,
    pub node_version: Option<String>,
    pub node_path: Option<String>,
    pub npm_path: Option<String>,
    pub cli_installed: bool,
    pub cli_path: PathBuf,
    pub cli_version: Option<String>,
    pub message: String,
}

pub fn openclaw_binary(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let prefix = paths::app_dir(app, "openclaw")?.join("cli");
    #[cfg(target_os = "windows")]
    {
        Ok(prefix.join("openclaw.cmd"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(prefix.join("bin").join("openclaw"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpenClawProvider {
    DeepSeek,
    OpenAI,
    OpenRouter,
    Anthropic,
}

impl OpenClawProvider {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "deepseek" => Ok(Self::DeepSeek),
            "openai" => Ok(Self::OpenAI),
            "openrouter" => Ok(Self::OpenRouter),
            "anthropic" => Ok(Self::Anthropic),
            other => Err(format!("Unsupported provider `{other}`. Expected DeepSeek, OpenAI, OpenRouter, or Anthropic.")),
        }
    }

    pub fn secret_id(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseekApiToken",
            Self::OpenAI => "openaiApiToken",
            Self::OpenRouter => "openrouterApiToken",
            Self::Anthropic => "anthropicApiToken",
        }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::OpenAI => "openai",
            Self::OpenRouter => "openrouter",
            Self::Anthropic => "anthropic",
        }
    }

    fn default_model_ref(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek/deepseek-chat",
            Self::OpenAI => "openai/gpt-5.5",
            Self::OpenRouter => "openrouter/openai/gpt-4.1-mini",
            Self::Anthropic => "anthropic/claude-sonnet-4-6",
        }
    }

    fn auth_choice(&self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek-api-key",
            Self::OpenAI => "openai-api-key",
            Self::OpenRouter => "openrouter-api-key",
            Self::Anthropic => "apiKey",
        }
    }

    fn inline_flag(&self) -> &'static str {
        match self {
            Self::DeepSeek => "--deepseek-api-key",
            Self::OpenAI => "--openai-api-key",
            Self::OpenRouter => "--openrouter-api-key",
            Self::Anthropic => "--anthropic-api-key",
        }
    }

    fn token(&self, app: &AppHandle) -> Result<String, String> {
        token_store::get("openclaw", self.secret_id())
            .filter(|value| !value.trim().is_empty())
            .or_else(|| load_provider_token_from_auth_profiles(app, *self).ok().flatten())
            .ok_or_else(|| {
                format!(
                    "OpenClaw Desktop needs a saved {} API key before it can repair startup automatically.",
                    self.id()
                )
            })
    }
}

fn provider_has_token(app: &AppHandle, provider: OpenClawProvider) -> bool {
    token_store::get("openclaw", provider.secret_id())
        .or_else(|| load_provider_token_from_auth_profiles(app, provider).ok().flatten())
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn first_available_provider(app: &AppHandle) -> Option<OpenClawProvider> {
    [
        OpenClawProvider::DeepSeek,
        OpenClawProvider::OpenAI,
        OpenClawProvider::OpenRouter,
        OpenClawProvider::Anthropic,
    ]
    .into_iter()
    .find(|provider| provider_has_token(app, *provider))
}

fn provider_from_model_ref(model_ref: &str) -> Option<OpenClawProvider> {
    let normalized = model_ref.trim().to_ascii_lowercase();
    if normalized.starts_with("deepseek/") {
        Some(OpenClawProvider::DeepSeek)
    } else if normalized.starts_with("openai/") {
        Some(OpenClawProvider::OpenAI)
    } else if normalized.starts_with("openrouter/") {
        Some(OpenClawProvider::OpenRouter)
    } else if normalized.starts_with("anthropic/") {
        Some(OpenClawProvider::Anthropic)
    } else {
        None
    }
}

fn provider_from_config(root: &serde_json::Value) -> Option<OpenClawProvider> {
    root.get("opennestDesktop")
        .and_then(|value| value.get("selectedProvider"))
        .and_then(|value| value.as_str())
        .and_then(|value| OpenClawProvider::parse(value).ok())
        .or_else(|| {
            root.get("agents")
                .and_then(|value| value.get("defaults"))
                .and_then(|value| value.get("model"))
                .and_then(|value| {
                    value
                        .as_str()
                        .map(ToOwned::to_owned)
                        .or_else(|| value.get("primary").and_then(|primary| primary.as_str()).map(ToOwned::to_owned))
                })
                .and_then(|value| provider_from_model_ref(&value))
        })
}

fn resolve_provider_for_runtime(app: &AppHandle, preferred: Option<OpenClawProvider>, root: &serde_json::Value) -> Option<OpenClawProvider> {
    preferred
        .filter(|provider| provider_has_token(app, *provider))
        .or_else(|| provider_from_config(root).filter(|provider| provider_has_token(app, *provider)))
        .or_else(|| first_available_provider(app))
}

fn prepare_openclaw_command(app: &AppHandle) -> Result<Command, String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet. Run Install first.".to_string());
    }
    let runtime = node_runtime::ensure_node_runtime(app, "openclaw")?;
    let dir = paths::app_dir(app, "openclaw")?;
    let state_dir = dir.join("state");
    let config_dir = dir.join("config");
    std::fs::create_dir_all(&state_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(binary);
    cmd.env("PATH", runtime.path_env())
        .env("OPENCLAW_HOME", &dir)
        .env("OPENCLAW_STATE_DIR", &state_dir)
        .env("OPENCLAW_CONFIG_PATH", config_dir.join("openclaw.json"));

    if let Some(token) = token_store::get("openclaw", "openrouterApiToken") {
        cmd.env("OPENROUTER_API_KEY", token);
    }
    if let Some(token) = token_store::get("openclaw", "deepseekApiToken") {
        cmd.env("DEEPSEEK_API_KEY", token);
    }
    if let Some(token) = token_store::get("openclaw", "openaiApiToken") {
        cmd.env("OPENAI_API_KEY", token);
    }
    if let Some(token) = token_store::get("openclaw", "anthropicApiToken") {
        cmd.env("ANTHROPIC_API_KEY", token);
    }
    if let Some(token) = token_store::get("openclaw", "geminiApiToken") {
        cmd.env("GEMINI_API_KEY", token);
    }

    Ok(cmd)
}

fn openclaw_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, "openclaw")?.join("config").join("openclaw.json"))
}

fn openclaw_state_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, "openclaw")?.join("state"))
}

fn openclaw_workspace_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, "openclaw")?.join(".openclaw").join("workspace"))
}

fn ensure_openclaw_dirs(app: &AppHandle) -> Result<(), String> {
    let config_dir = paths::app_dir(app, "openclaw")?.join("config");
    let state_dir = openclaw_state_dir(app)?;
    let workspace_dir = openclaw_workspace_dir(app)?;
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&state_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&workspace_dir).map_err(|e| e.to_string())?;
    Ok(())
}

fn random_gateway_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect::<String>()
        .to_ascii_lowercase()
}

fn openclaw_agent_models_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, "openclaw")?
        .join("state")
        .join("agents")
        .join("main")
        .join("agent")
        .join("models.json"))
}

fn openclaw_agent_auth_profiles_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(paths::app_dir(app, "openclaw")?
        .join("state")
        .join("agents")
        .join("main")
        .join("agent")
        .join("auth-profiles.json"))
}

fn load_provider_token_from_auth_profiles(app: &AppHandle, provider: OpenClawProvider) -> Result<Option<String>, String> {
    let auth_profiles_path = openclaw_agent_auth_profiles_path(app)?;
    if !auth_profiles_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&auth_profiles_path)
        .map_err(|error| format!("failed to read OpenClaw auth profiles at {}: {error}", auth_profiles_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse OpenClaw auth profiles at {}: {error}", auth_profiles_path.display()))?;
    let profiles = match parsed.get("profiles").and_then(|value| value.as_object()) {
        Some(profiles) => profiles,
        None => return Ok(None),
    };

    let token = profiles
        .values()
        .find(|entry| {
            entry.get("provider")
                .and_then(|value| value.as_str())
                .map(|value| value.eq_ignore_ascii_case(provider.id()))
                .unwrap_or(false)
        })
        .and_then(|entry| entry.get("key"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Ok(token)
}

fn ensure_openclaw_config(
    app: &AppHandle,
    preferred_provider: Option<OpenClawProvider>,
) -> Result<(String, Option<OpenClawProvider>), String> {
    ensure_openclaw_dirs(app)?;
    let config_path = openclaw_config_path(app)?;
    let mut root = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|error| format!("failed to read OpenClaw config at {}: {error}", config_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&raw)
            .map_err(|error| format!("failed to parse OpenClaw config at {}: {error}", config_path.display()))?
    } else {
        serde_json::json!({})
    };

    let existing = root
        .get("gateway")
        .and_then(|gateway| gateway.get("auth"))
        .and_then(|auth| auth.get("token"))
        .and_then(|token| token.as_str())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned);

    let token = existing.unwrap_or_else(random_gateway_token);
    let selected_provider = resolve_provider_for_runtime(app, preferred_provider, &root);

    if let Some(object) = root.as_object_mut() {
        object.remove("provider");
        object.remove("defaultModel");
        object.remove("authConfigured");
        object.remove("gatewayToken");
        object.remove("systemGatewayToken");
        object.remove("opennestDesktop");
    }

    root["gateway"]["mode"] = serde_json::json!("local");
    root["gateway"]["bind"] = serde_json::json!("loopback");
    root["gateway"]["port"] = serde_json::json!(18789);
    root["gateway"]["auth"]["mode"] = serde_json::json!("token");
    root["gateway"]["auth"]["token"] = serde_json::json!(token.clone());
    if let Some(provider) = selected_provider {
        root["agents"]["defaults"]["model"] = serde_json::json!(provider.default_model_ref());
        if !root["agents"]["defaults"]["models"][provider.default_model_ref()].is_object() {
            root["agents"]["defaults"]["models"][provider.default_model_ref()] = serde_json::json!({});
        }
    }

    let serialized = serde_json::to_string_pretty(&root).map_err(|error| error.to_string())?;
    fs::write(&config_path, serialized)
        .map_err(|error| format!("failed to write OpenClaw config at {}: {error}", config_path.display()))?;
    Ok((token, selected_provider))
}

fn ensure_gateway_token_in_config(app: &AppHandle) -> Result<String, String> {
    ensure_openclaw_config(app, None).map(|(token, _)| token)
}

fn ensure_agent_models_file(app: &AppHandle) -> Result<(), String> {
    ensure_openclaw_dirs(app)?;
    let config_path = openclaw_config_path(app)?;
    let raw = fs::read_to_string(&config_path)
        .map_err(|error| format!("failed to read OpenClaw config at {}: {error}", config_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse OpenClaw config at {}: {error}", config_path.display()))?;
    let agent_models_path = openclaw_agent_models_path(app)?;
    if let Some(parent) = agent_models_path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("failed to create OpenClaw agent config dir: {error}"))?;
    }
    let payload = serde_json::json!({
        "providers": parsed
            .get("models")
            .and_then(|value| value.get("providers"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}))
    });
    let serialized = serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())?;
    fs::write(&agent_models_path, serialized)
        .map_err(|error| format!("failed to write OpenClaw agent models at {}: {error}", agent_models_path.display()))?;
    Ok(())
}

fn openclaw_dashboard_token_url(app: &AppHandle, port: u16) -> Result<String, String> {
    let ensured_token = ensure_gateway_token_in_config(app)?;
    let config_path = openclaw_config_path(app)?;
    let raw = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("failed to read OpenClaw config at {}: {error}", config_path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse OpenClaw config at {}: {error}", config_path.display()))?;
    let token = parsed
        .get("gateway")
        .and_then(|gateway| gateway.get("auth"))
        .and_then(|auth| auth.get("token"))
        .and_then(|token| token.as_str())
        .filter(|token| !token.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or(ensured_token);
    Ok(format!("http://127.0.0.1:{port}/#token={token}"))
}

pub fn configure_openclaw_provider(app: &AppHandle, provider: OpenClawProvider, api_key: &str) -> Result<String, String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet. Run Install first.".to_string());
    }

    ensure_openclaw_dirs(app)?;
    let runtime = node_runtime::ensure_node_runtime(app, "openclaw")?;
    let state_dir = openclaw_state_dir(app)?;
    let workspace_dir = openclaw_workspace_dir(app)?;
    let config_path = openclaw_config_path(app)?;

    let mut command = Command::new(binary);
    command
        .env("PATH", runtime.path_env())
        .env("OPENCLAW_HOME", paths::app_dir(app, "openclaw")?)
        .env("OPENCLAW_STATE_DIR", &state_dir)
        .env("OPENCLAW_CONFIG_PATH", &config_path)
        .args([
            "onboard",
            "--non-interactive",
            "--accept-risk",
            "--flow",
            "quickstart",
            "--mode",
            "local",
            "--skip-health",
            "--skip-channels",
            "--skip-skills",
            "--skip-search",
            "--skip-ui",
            "--skip-bootstrap",
            "--no-install-daemon",
            "--workspace",
            workspace_dir.to_string_lossy().as_ref(),
            "--auth-choice",
            provider.auth_choice(),
            provider.inline_flag(),
            api_key,
        ]);

    logs::append(
        app,
        "openclaw",
        "setup",
        &format!("running desktop-managed quickstart onboarding for provider {}", provider.auth_choice()),
    )?;
    let output = command_timeout::run_with_timeout(command, command_timeout::INSTALL_TIMEOUT_MS)
        .map_err(|error| format!("OpenClaw setup failed to start or wait: {error}"))?;
    logs::append(app, "openclaw", "setup", &output.stdout)?;
    logs::append(app, "openclaw", "setup", &output.stderr)?;
    if !output.success {
        return Err(output.failure_message("openclaw onboard --non-interactive"));
    }

    let (token, _) = ensure_openclaw_config(app, Some(provider))?;
    ensure_agent_models_file(app)?;
    logs::append(app, "openclaw", "setup", &format!("desktop-managed setup completed; gateway token available locally ({})", token.len()))?;
    Ok(format!(
        "Provider configured for {}. OpenClaw Desktop will now default to {}.",
        provider.id(),
        provider.default_model_ref()
    ))
}

fn run_openclaw_command(
    app: &AppHandle,
    category: &str,
    args: &[&str],
    timeout_ms: u64,
) -> Result<command_timeout::TimedCommandOutput, String> {
    let mut command = prepare_openclaw_command(app)?;
    command.args(args);
    logs::append(app, "openclaw", category, &format!("running openclaw {}", args.join(" ")))?;
    let output = command_timeout::run_with_timeout(command, timeout_ms)
        .map_err(|error| format!("openclaw {} failed to start or wait: {error}", args.join(" ")))?;
    logs::append(app, "openclaw", category, &format!("openclaw {} finished duration_ms={} timed_out={} exit_code={:?}", args.join(" "), output.duration_ms, output.timed_out, output.exit_code))?;
    logs::append(app, "openclaw", category, &output.stdout)?;
    logs::append(app, "openclaw", category, &output.stderr)?;

    if output.success {
        Ok(output)
    } else {
        Err(output.failure_message(&format!("openclaw {}", args.join(" "))))
    }
}

fn run_openclaw_once(app: &AppHandle, category: &str, args: &[&str], timeout_ms: u64) -> Result<String, String> {
    let output = run_openclaw_command(app, category, args, timeout_ms)?;
    Ok(format!("{}{}", output.stdout, output.stderr).trim().to_string())
}

pub fn inspect_openclaw_environment(app: &AppHandle) -> Result<OpenClawEnvironmentReport, RuntimeActionError> {
    let inspection = node_runtime::inspect_runtime(app);
    let cli_path = openclaw_binary(app)
        .map_err(|error| RuntimeActionError::with_detail(
            "OPENCLAW_PATH_RESOLUTION_FAILED",
            "Failed to resolve the OpenClaw CLI path.",
            Some(error),
            Some("Inspect the OpenClaw recipe paths and retry Check Environment.".to_string()),
        ))?;

    let cli_installed = cli_path.exists();
    let cli_version = if cli_installed {
        let mut command = prepare_openclaw_command(app)
            .map_err(|error| RuntimeActionError::with_detail(
                "OPENCLAW_ENVIRONMENT_FAILED",
                "OpenClaw CLI is installed but could not be prepared for execution.",
                Some(error),
                Some("Run Check Environment again after fixing Node/npm availability.".to_string()),
            ))?;
        command.arg("--version");
        let output = command_timeout::run_with_timeout(command, command_timeout::CHECK_TIMEOUT_MS)
            .map_err(|error| RuntimeActionError::with_detail(
                "OPENCLAW_VERSION_CHECK_FAILED",
                "Failed to execute `openclaw --version`.",
                Some(error),
                Some("Review Logs, then retry Check Environment.".to_string()),
            ))?;
        let version_text = format!("{}{}", output.stdout, output.stderr).trim().to_string();
        if output.success {
            Some(version_text)
        } else {
            return Err(RuntimeActionError::with_detail(
                "OPENCLAW_VERSION_CHECK_FAILED",
                "OpenClaw CLI is present but `openclaw --version` failed.",
                Some(output.failure_message("openclaw --version")),
                Some("Re-run Install, then retry Check Environment.".to_string()),
            ));
        }
    } else {
        None
    };

    if !inspection.usable {
        return Err(RuntimeActionError::with_detail(
            "NODE_RUNTIME_UNAVAILABLE",
            "No supported Node/npm runtime is available for OpenClaw.",
            Some(inspection.message),
            Some("Run Repair to prepare managed Node, then run Check Environment again.".to_string()),
        ));
    }

    let cli_state = if cli_installed {
        format!(
            "OpenClaw CLI installed at {}{}",
            cli_path.display(),
            cli_version
                .as_ref()
                .map(|version| format!(" ({version})"))
                .unwrap_or_default()
        )
    } else {
        format!("OpenClaw CLI is not installed yet. Expected path: {}", cli_path.display())
    };

    Ok(OpenClawEnvironmentReport {
        node_source: inspection.source,
        node_version: inspection.version,
        node_path: inspection.node_path,
        npm_path: inspection.npm_path,
        cli_installed,
        cli_path,
        cli_version,
        message: format!("{} {cli_state}", inspection.message),
    })
}

pub fn install_openclaw(app: &AppHandle) -> Result<(), String> {
    let dir = paths::app_dir(app, "openclaw")?;
    let prefix = dir.join("cli");
    std::fs::create_dir_all(&prefix).map_err(|e| e.to_string())?;
    let runtime = node_runtime::ensure_node_runtime(app, "openclaw")?;
    logs::append(app, "openclaw", "install", &format!("installing openclaw@latest with npm prefix using {}", runtime.describe()))?;

    let mut command = Command::new(&runtime.npm_path);
    command
        .arg("install")
        .arg("-g")
        .arg("openclaw@latest")
        .arg("--prefix")
        .arg(prefix.to_string_lossy().to_string())
        .arg("--no-audit")
        .arg("--no-fund")
        .env("PATH", runtime.path_env());

    let output = command_timeout::run_with_timeout(command, command_timeout::INSTALL_TIMEOUT_MS)
        .map_err(|error| format!("npm install failed to start or wait: {error}"))?;

    logs::append(app, "openclaw", "install", &format!("npm install finished duration_ms={} timed_out={} exit_code={:?}", output.duration_ms, output.timed_out, output.exit_code))?;
    logs::append(app, "openclaw", "install", &output.stdout)?;
    logs::append(app, "openclaw", "install", &output.stderr)?;

    if !output.success {
        return Err(output.failure_message("npm install -g openclaw@latest"));
    }

    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err(format!("npm install completed, but OpenClaw binary was not found at {}", binary.display()));
    }

    let _ = run_openclaw_once(app, "install", &["--version"], command_timeout::CHECK_TIMEOUT_MS);
    Ok(())
}

pub fn repair_openclaw_cli_path(app: &AppHandle) -> Result<String, String> {
    let expected = openclaw_binary(app)?;
    if expected.exists() {
        logs::append(app, "openclaw", "repair", &format!("OpenClaw CLI already exists at {}", expected.display()))?;
        return Ok(format!("OpenClaw CLI already exists at {}", expected.display()));
    }

    let app_dir = paths::app_dir(app, "openclaw")?;
    let candidates = [
        app_dir.join("cli").join("bin").join("openclaw.cmd"),
        app_dir.join("cli").join("node_modules").join(".bin").join("openclaw.cmd"),
        app_dir.join("cli").join("node_modules").join("openclaw").join("openclaw.cmd"),
    ];

    let Some(found) = candidates.into_iter().find(|path| path.exists()) else {
        return Err(format!(
            "OpenClaw CLI is still missing. Expected {}, and no fallback candidate was found under the managed CLI directory.",
            expected.display()
        ));
    };

    #[cfg(target_os = "windows")]
    {
        let shim = format!("@echo off\r\n\"{}\" %*\r\n", found.display());
        std::fs::write(&expected, shim).map_err(|error| format!("failed to recreate openclaw.cmd shim: {error}"))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::fs::copy(&found, &expected).map_err(|error| format!("failed to restore OpenClaw binary: {error}"))?;
    }

    logs::append(
        app,
        "openclaw",
        "repair",
        &format!("recreated OpenClaw CLI entrypoint at {} from {}", expected.display(), found.display()),
    )?;
    Ok(format!(
        "Recreated OpenClaw CLI entrypoint at {} from {}",
        expected.display(),
        found.display()
    ))
}

pub fn start_openclaw(app: &AppHandle, port: u16) -> Result<Option<u32>, String> {
    let _ = ensure_openclaw_config(app, None)?;
    ensure_agent_models_file(app)?;

    let already_reachable = healthcheck::check_tcp("127.0.0.1", port, 300);
    if already_reachable.ok {
        logs::append(app, "openclaw", "gateway", &format!("OpenClaw gateway already reachable on managed port {port}; skipping restart"))?;
        return Ok(None);
    }

    if let Some(record) = process_manager::load_record(app, "openclaw")? {
        if process_manager::is_pid_running(record.pid) {
            let report = healthcheck::check_tcp("127.0.0.1", record.port.unwrap_or(port), 750);
            if report.ok {
                logs::append(app, "openclaw", "process", &format!("managed OpenClaw is already reachable pid={}", record.pid))?;
                return Ok(Some(record.pid));
            }
            let _ = process_manager::stop_managed(app, "openclaw");
        } else {
            process_manager::clear_record(app, "openclaw")?;
        }
    }

    run_openclaw_once(app, "gateway", &["gateway", "start"], command_timeout::START_TIMEOUT_MS)?;

    if let Err(error) = healthcheck::wait_for_tcp(app, "openclaw", "127.0.0.1", port, 25_000, 750) {
        return Err(format!("OpenClaw gateway start command completed, but 127.0.0.1:{port} did not become reachable. Re-run desktop setup or inspect logs/doctor output. Details: {error}"));
    }

    logs::append(app, "openclaw", "gateway", "OpenClaw gateway is reachable after official gateway start")?;
    Ok(None)
}

pub fn auto_heal_openclaw_startup(app: &AppHandle, port: u16) -> Result<(), String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Ok(());
    }

    ensure_openclaw_dirs(app)?;
    let config_path = openclaw_config_path(app)?;
    let bootstrap_root = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .map_err(|error| format!("failed to read OpenClaw config at {}: {error}", config_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&raw)
            .map_err(|error| format!("failed to parse OpenClaw config at {}: {error}", config_path.display()))?
    } else {
        serde_json::json!({})
    };

    let provider = resolve_provider_for_runtime(app, None, &bootstrap_root).ok_or_else(|| {
        "OpenClaw Desktop needs one model provider key before it can start. Add a DeepSeek, OpenAI, OpenRouter, or Anthropic key in the app and reopen it.".to_string()
    })?;

    let provider_token = provider.token(app)?;
    let _ = ensure_openclaw_config(app, Some(provider))?;
    ensure_agent_models_file(app)?;
    let auth_profiles_path = openclaw_agent_auth_profiles_path(app)?;
    let auth_profiles_missing = !auth_profiles_path.exists()
        || fs::read_to_string(&auth_profiles_path)
            .map(|content| content.trim().is_empty())
            .unwrap_or(true);
    if auth_profiles_missing {
        logs::append(
            app,
            "openclaw",
            "startup",
            "auth profile file missing or empty; rerunning desktop-managed provider setup",
        )?;
        configure_openclaw_provider(app, provider, &provider_token)?;
    }

    let tcp_ready = healthcheck::check_tcp("127.0.0.1", port, 300).ok;
    if !tcp_ready {
        if healthcheck::wait_for_tcp(app, "openclaw", "127.0.0.1", port, 8_000, 500).is_ok() {
            logs::append(
                app,
                "openclaw",
                "startup",
                &format!("OpenClaw gateway became reachable during startup grace period on port {port}; skipping restart"),
            )?;
            return Ok(());
        }
        let _ = stop_openclaw(app);
        start_openclaw(app, port)?;
    }

    let final_report = healthcheck::check_tcp("127.0.0.1", port, 750);
    if !final_report.ok {
        return Err(
            "OpenClaw Desktop could not bring its local chat service online automatically. Reconnect your model key or run Repair, then try again."
                .to_string(),
        );
    }

    logs::append(
        app,
        "openclaw",
        "startup",
        &format!(
            "startup self-check completed: provider={} default_model={} port={port}",
            provider.id(),
            provider.default_model_ref()
        ),
    )?;
    Ok(())
}

pub fn gateway_status_openclaw(app: &AppHandle) -> Result<String, String> {
    let output = run_openclaw_command(app, "gateway-status", &["gateway", "status"], command_timeout::CHECK_TIMEOUT_MS)?;
    let text = format!("{}{}", output.stdout, output.stderr).trim().to_string();
    if text.is_empty() {
        Ok("OpenClaw gateway status completed without textual output.".to_string())
    } else {
        Ok(text)
    }
}

pub fn gateway_probe_openclaw(app: &AppHandle) -> Result<String, String> {
    let output = run_openclaw_command(app, "gateway-probe", &["gateway", "probe"], command_timeout::CHECK_TIMEOUT_MS)?;
    let text = format!("{}{}", output.stdout, output.stderr).trim().to_string();
    if text.is_empty() {
        Ok("OpenClaw gateway probe completed without textual output.".to_string())
    } else {
        Ok(text)
    }
}

pub fn stop_openclaw(app: &AppHandle) -> Result<String, String> {
    if let Some(record) = process_manager::load_record(app, "openclaw")? {
        if process_manager::is_pid_running(record.pid) {
            return process_manager::stop_managed(app, "openclaw");
        }
        let _ = process_manager::clear_record(app, "openclaw");
    }

    match run_openclaw_once(app, "gateway", &["gateway", "stop"], command_timeout::STOP_TIMEOUT_MS) {
        Ok(output) => Ok(if output.is_empty() { "OpenClaw gateway stop completed.".to_string() } else { output }),
        Err(error) => Err(format!("OpenClaw gateway stop failed. If OpenClaw was started outside OpenNest, stop it from the terminal. {error}")),
    }
}

pub fn openclaw_onboarding(app: &AppHandle) -> Result<(), String> {
    let binary = openclaw_binary(app)?;
    if !binary.exists() {
        return Err("OpenClaw CLI is not installed yet. Run Install first.".to_string());
    }
    let runtime = node_runtime::ensure_node_runtime(app, "openclaw")?;
    logs::append(app, "openclaw", "onboarding", "opening official onboarding: openclaw onboard --install-daemon")?;
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
        Command::new(binary)
            .env("PATH", runtime.path_env())
            .args(["onboard", "--install-daemon"])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn openclaw_doctor(app: &AppHandle) -> Result<(), String> {
    match run_openclaw_once(app, "doctor", &["doctor", "--non-interactive"], command_timeout::CHECK_TIMEOUT_MS) {
        Ok(_) => Ok(()),
        Err(first_error) => {
            logs::append(app, "openclaw", "doctor", &format!("doctor --non-interactive failed; retrying plain doctor: {first_error}"))?;
            run_openclaw_once(app, "doctor", &["doctor"], command_timeout::CHECK_TIMEOUT_MS).map(|_| ())
        }
    }
}

pub fn openclaw_dashboard(app: &AppHandle, port: u16) -> Result<String, String> {
    let report = healthcheck::check_tcp("127.0.0.1", port, 750);
    if !report.ok {
        return Err(format!(
            "OpenClaw Desktop could not reach its local chat service. Run Start or reconnect the provider key, then try again. {}",
            report.error.unwrap_or_default()
        ));
    }

    let url = openclaw_dashboard_token_url(app, port)?;
    logs::append(app, "openclaw", "dashboard", "desktop chat URL resolved from local config without extra CLI round-trip")?;
    Ok(url)
}
