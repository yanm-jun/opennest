use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use super::recipe_loader::OpenNestRecipe;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlanItem {
    pub label: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeInstallPlan {
    pub app_id: String,
    pub name: String,
    pub plan_version: String,
    pub plan_digest: String,
    pub runtime: String,
    pub summary: String,
    pub install_strategy: Option<String>,
    pub start_strategy: Option<String>,
    pub dashboard_url: Option<String>,
    pub risk_level: String,
    pub estimated_time: String,
    pub estimated_disk: String,
    pub requires_network: bool,
    pub requires_docker: bool,
    pub requires_node: bool,
    pub requires_git: bool,
    pub recommended_memory_gb: Option<u64>,
    pub recommended_cpu: Option<u64>,
    pub ports: Vec<u16>,
    pub downloads: Vec<InstallPlanItem>,
    pub directories: Vec<InstallPlanItem>,
    pub commands: Vec<InstallPlanItem>,
    pub secrets: Vec<InstallPlanItem>,
    pub permissions: Vec<InstallPlanItem>,
    pub checks: Vec<InstallPlanItem>,
    pub rollback: Vec<InstallPlanItem>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

fn item(
    label: impl Into<String>,
    value: Option<impl Into<String>>,
    description: Option<impl Into<String>>,
    required: bool,
) -> InstallPlanItem {
    InstallPlanItem {
        label: label.into(),
        value: value.map(Into::into),
        description: description.map(Into::into),
        required,
    }
}

fn bool_requirement(recipe: &OpenNestRecipe, key: &str) -> bool {
    recipe
        .requirements
        .as_ref()
        .and_then(|requirements| requirements.get(key))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn u64_requirement(recipe: &OpenNestRecipe, key: &str) -> Option<u64> {
    recipe
        .requirements
        .as_ref()
        .and_then(|requirements| requirements.get(key))
        .and_then(Value::as_u64)
}

fn has_requirement(recipe: &OpenNestRecipe, key: &str) -> bool {
    recipe
        .requirements
        .as_ref()
        .and_then(|requirements| requirements.get(key))
        .is_some()
}

fn value_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn value_field_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(value_string)
}

fn substitute_known_paths(template: &str, app_dir: &str) -> String {
    let mut value = template.replace("${openNestAppData}/apps", "<OpenNest app data>/apps");
    value = value.replace("${appDir}", app_dir);
    value = value.replace("${composeDir}", &format!("{app_dir}/source/docker"));
    value = value.replace("${binary}", "<resolved OpenClaw binary>");
    value
}

fn risk_level(recipe: &OpenNestRecipe) -> String {
    match recipe.runtime.as_str() {
        "external-compose" => "high".to_string(),
        "docker-compose" => "medium".to_string(),
        "native-cli" => "medium".to_string(),
        _ => "unknown".to_string(),
    }
}

fn estimated_time(recipe: &OpenNestRecipe) -> String {
    match recipe.id.as_str() {
        "openclaw" => "5-15 minutes, depending on npm/network speed".to_string(),
        "open-webui" => "5-20 minutes, depending on Docker image download speed".to_string(),
        "flowise" => "3-10 minutes, depending on Docker image download speed".to_string(),
        "dify" => "10-30+ minutes, because Dify syncs an official multi-service Docker stack".to_string(),
        _ => "Unknown; depends on recipe downloads and runtime".to_string(),
    }
}

fn estimated_disk(recipe: &OpenNestRecipe) -> String {
    match recipe.id.as_str() {
        "openclaw" => "~500 MB to 1.5 GB including managed Node/npm packages".to_string(),
        "open-webui" => "~5 GB to 10 GB including Docker image and local data".to_string(),
        "flowise" => "~2 GB to 5 GB including Docker image and local data".to_string(),
        "dify" => "~10 GB to 25 GB+ including official source, images, volumes, and databases".to_string(),
        _ => "Unknown; check recipe runtime and upstream images".to_string(),
    }
}

fn downloads(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    let mut result = Vec::new();

    if let Some(version_source) = recipe.version_source.as_ref() {
        result.push(item(
            "Version source",
            Some(version_source.clone()),
            Some("Upstream package/image/source reference used by this recipe."),
            true,
        ));
    }

    if let Some(install) = recipe.install.as_ref() {
        if let Some(package) = install.package.as_ref() {
            result.push(item(
                "npm package",
                Some(package.clone()),
                Some("Downloaded by the resolved Node/npm runtime into the OpenNest app data directory."),
                true,
            ));
        }
        if let Some(source) = install.source.as_ref() {
            result.push(item(
                "Compose source",
                Some(source.clone()),
                Some("Embedded compose template copied into the app workspace."),
                true,
            ));
        }
        if let Some(repo) = install.repo.as_ref() {
            result.push(item(
                "Git repository",
                Some(repo.clone()),
                Some("Official upstream source cloned into the OpenNest app workspace."),
                true,
            ));
        }
        if let Some(git_ref) = install.git_ref.as_ref() {
            result.push(item(
                "Git ref",
                Some(git_ref.clone()),
                Some("Branch/tag/ref checked out for the external compose source."),
                true,
            ));
        }
    }

    result
}

fn directories(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<Vec<InstallPlanItem>, String> {
    let app_data = app.path().app_data_dir().map_err(|error| format!("failed to resolve app data dir: {error}"))?;
    let app_dir = app_data.join("apps").join(&recipe.id).display().to_string();
    let mut result = vec![
        item(
            "App workspace",
            Some(app_dir.clone()),
            Some("OpenNest-managed directory for this app."),
            true,
        ),
        item(
            "Logs directory",
            Some(format!("{app_dir}/logs")),
            Some("OpenNest writes install/start/stop/runtime logs here."),
            true,
        ),
    ];

    if let Some(paths_value) = recipe.paths.as_ref().and_then(Value::as_object) {
        for (label, value) in paths_value {
            if let Some(template) = value.as_str() {
                result.push(item(
                    label.clone(),
                    Some(substitute_known_paths(template, &app_dir)),
                    Some("Declared by recipe.paths."),
                    true,
                ));
            }
        }
    }

    Ok(result)
}

fn commands(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    if recipe.id == "openclaw" {
        return vec![
            item(
                "Install CLI",
                Some("npm install -g openclaw@latest --prefix <app workspace>/cli"),
                Some("OpenNest installs OpenClaw into its managed app directory."),
                true,
            ),
            item(
                "Desktop setup",
                Some("Select provider + paste API key"),
                Some("OpenNest runs a non-interactive local quickstart, completes local sign-in wiring, and switches the default model automatically."),
                true,
            ),
            item(
                "Start gateway",
                Some("<resolved OpenClaw binary> gateway start"),
                Some("OpenNest starts the local gateway after setup is complete."),
                true,
            ),
            item(
                "Open chat",
                Some("OpenClaw Desktop embedded chat window"),
                Some("OpenNest opens the embedded chat window with local auth already injected."),
                true,
            ),
        ];
    }

    let mut result = Vec::new();

    result.push(item(
        "Check environment",
        Some(match recipe.runtime.as_str() {
            "native-cli" => "Check Windows + Node/npm runtime".to_string(),
            "docker-compose" => "docker compose version".to_string(),
            "external-compose" => "docker compose version + git --version".to_string(),
            other => format!("runtime {other} check is not implemented"),
        }),
        Some("Preflight command(s) run before install/start."),
        true,
    ));

    if let Some(install) = recipe.install.as_ref() {
        let value = match install.strategy.as_str() {
            "npm-global-prefix" => format!(
                "npm install -g {} --prefix <app workspace>/cli",
                install.package.clone().unwrap_or_else(|| "<package>".to_string())
            ),
            "write-compose" => format!(
                "write embedded compose template from {}",
                install.source.clone().unwrap_or_else(|| "<recipe source>".to_string())
            ),
            "clone-official-compose" => format!(
                "git clone/fetch {} then use compose dir {}",
                install.repo.clone().unwrap_or_else(|| "<repo>".to_string()),
                install.compose_dir.clone().unwrap_or_else(|| "docker".to_string())
            ),
            other => format!("install strategy: {other}"),
        };
        result.push(item("Install", Some(value), Some("Install command/strategy OpenNest will run."), true));
    }

    if let Some(start) = recipe.start.as_ref() {
        result.push(item(
            "Start",
            Some(match recipe.runtime.as_str() {
                "native-cli" => format!(
                    "{} {}",
                    start.command.clone().unwrap_or_else(|| "<resolved binary>".to_string()),
                    start.args.join(" ")
                ),
                "docker-compose" | "external-compose" => format!("docker compose {}", start.args.join(" ")),
                _ => start.args.join(" "),
            }),
            Some("Start command/strategy OpenNest will run."),
            true,
        ));
    }

    if let Some(stop) = recipe.stop.as_ref() {
        result.push(item(
            "Stop",
            Some(match recipe.runtime.as_str() {
                "native-cli" => stop.strategy.clone().unwrap_or_else(|| "tracked-pid-only".to_string()),
                "docker-compose" | "external-compose" => format!("docker compose {}", stop.args.join(" ")),
                _ => stop.args.join(" "),
            }),
            Some("Stop command/strategy OpenNest will run."),
            true,
        ));
    }

    result
}

fn secrets(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    if recipe.id == "openclaw" {
        return vec![item(
            "Provider API key",
            Some("DeepSeek / OpenAI / OpenRouter / Anthropic"),
            Some("User picks one provider in the desktop setup card. OpenNest writes the auth profile, default model, and local runtime config in one step."),
            true,
        )];
    }

    recipe
        .secrets
        .iter()
        .map(|secret| {
            let label = value_field_string(secret, "label")
                .or_else(|| value_field_string(secret, "id"))
                .unwrap_or_else(|| "Secret".to_string());
            let store = value_field_string(secret, "store").unwrap_or_else(|| "unknown-store".to_string());
            let required = secret.get("required").and_then(Value::as_bool).unwrap_or(false);
            let redact = secret.get("redact").and_then(Value::as_bool).unwrap_or(true);
            item(
                label,
                Some(store),
                Some(format!("required={required}; redacted={redact}; value is not written to recipe/status/log files.")),
                required,
            )
        })
        .collect()
}

fn permissions(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    recipe
        .permissions
        .iter()
        .map(|permission| {
            let permission_type = value_field_string(permission, "type").unwrap_or_else(|| "permission".to_string());
            let level = value_field_string(permission, "level").unwrap_or_else(|| "unspecified".to_string());
            let description = value_field_string(permission, "description");
            item(permission_type, Some(level), description, true)
        })
        .collect()
}

fn checks(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    let mut result = Vec::new();

    if recipe.runtime == "native-cli" || has_requirement(recipe, "node") {
        result.push(item(
            "Node runtime",
            Some("Node 24.x recommended; Node 22.16+ minimum"),
            Some("OpenNest will prefer a valid system Node, then fall back to managed Node runtime."),
            true,
        ));
    }

    if recipe.runtime == "docker-compose" || recipe.runtime == "external-compose" || bool_requirement(recipe, "docker") {
        result.push(item(
            "Docker Compose",
            Some("docker compose version"),
            Some("Docker Desktop must be installed and running before starting this app."),
            true,
        ));
    }

    if recipe.runtime == "external-compose" || recipe.install.as_ref().and_then(|install| install.repo.as_ref()).is_some() {
        result.push(item(
            "Git",
            Some("git --version"),
            Some("Required to sync official upstream source for external-compose apps."),
            true,
        ));
    }

    if !recipe.ports.is_empty() {
        result.push(item(
            "Port availability",
            Some(recipe.ports.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")),
            Some("These local ports must not be occupied by unrelated processes."),
            true,
        ));
    }

    if let Some(url) = recipe
        .start
        .as_ref()
        .and_then(|start| start.healthcheck.clone())
        .or_else(|| recipe.dashboard_url())
    {
        result.push(item(
            "Readiness URL",
            Some(url),
            Some("OpenNest checks this URL before treating the app as dashboard-ready."),
            false,
        ));
    }

    result
}

fn rollback(recipe: &OpenNestRecipe) -> Vec<InstallPlanItem> {
    match recipe.runtime.as_str() {
        "native-cli" => vec![
            item("Stop managed process", Some("tracked PID only"), Some("OpenNest will only stop the process it started."), true),
            item("Remove app workspace", Some("apps/<appId>"), Some("Manual cleanup can remove CLI/state/log files from OpenNest app data."), false),
        ],
        "docker-compose" => vec![
            item("Stop compose services", Some("docker compose stop"), Some("Stops services defined in the OpenNest-managed compose file."), true),
            item("Remove app workspace", Some("apps/<appId>"), Some("Manual cleanup can remove compose/log files; Docker volumes/images may remain."), false),
        ],
        "external-compose" => vec![
            item("Stop official compose services", Some("docker compose stop"), Some("Runs inside the upstream official compose directory."), true),
            item("Preserve upstream source", Some("apps/<appId>/source"), Some("OpenNest does not delete official source/volumes automatically without an explicit uninstall adapter."), false),
        ],
        _ => vec![item("No rollback adapter", None::<String>, Some("This runtime has no rollback implementation yet."), false)],
    }
}

fn warnings(recipe: &OpenNestRecipe) -> Vec<String> {
    let mut warnings = Vec::new();

    if bool_requirement(recipe, "network") || recipe.install.as_ref().is_some() {
        warnings.push("Network access is required for upstream packages/images/source downloads.".to_string());
    }

    if recipe.runtime == "docker-compose" || recipe.runtime == "external-compose" {
        warnings.push("Docker Desktop must be running. Container startup can be slow on first install.".to_string());
        warnings.push("Docker images, containers, and volumes may consume significant disk space outside the OpenNest workspace.".to_string());
    }

    if recipe.runtime == "external-compose" {
        warnings.push("This app uses the official upstream compose directory. Do not manually move partial source files while install/start is running.".to_string());
    }

    if !recipe.ports.is_empty() {
        warnings.push(format!(
            "Port conflicts can prevent startup. Required local ports: {}.",
            recipe.ports.iter().map(u16::to_string).collect::<Vec<_>>().join(", ")
        ));
    }

    if recipe.id == "openclaw" {
        warnings.push("OpenClaw setup is desktop-managed. Users should not be asked for gateway URL, gateway token, password, or port details.".to_string());
    }

    warnings
}

fn notes(recipe: &OpenNestRecipe) -> Vec<String> {
    let mut notes = Vec::new();
    notes.push("Install plan is generated from embedded registry/recipe metadata plus OpenNest workspace paths.".to_string());
    notes.push("Install plan acceptance is required before installation. If recipe metadata changes, the plan digest changes and must be accepted again.".to_string());

    if let Some(license) = recipe.license.as_ref() {
        notes.push(format!("Upstream license field: {license}. Verify upstream licensing before redistribution."));
    }

    if let Some(homepage) = recipe.homepage.as_ref() {
        notes.push(format!("Upstream homepage: {homepage}"));
    }

    if recipe.id == "openclaw" {
        notes.push("OpenClaw first run is: Install CLI -> choose provider -> paste API key -> open chat.".to_string());
    }

    notes
}


fn plan_version(recipe: &OpenNestRecipe) -> String {
    let upstream = recipe.version_source.clone().unwrap_or_else(|| "local".to_string());
    let install_strategy = recipe.install.as_ref().map(|install| install.strategy.clone()).unwrap_or_else(|| "none".to_string());
    format!("schema:{}|runtime:{}|source:{}|install:{}", recipe.schema_version, recipe.runtime, upstream, install_strategy)
}

pub fn digest(plan: &RecipeInstallPlan) -> Result<String, String> {
    let mut clone = plan.clone();
    clone.plan_digest.clear();
    let canonical = serde_json::to_string(&clone).map_err(|error| format!("failed to serialize install plan for digest: {error}"))?;
    // Stable FNV-1a digest. This is not a cryptographic hash; it is a
    // deterministic plan-change fingerprint for install gating.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in canonical.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{:016x}", hash))
}

pub fn build(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeInstallPlan, String> {
    let requires_docker = recipe.runtime == "docker-compose"
        || recipe.runtime == "external-compose"
        || bool_requirement(recipe, "docker");
    let requires_node = recipe.runtime == "native-cli" || has_requirement(recipe, "node");
    let requires_git = recipe.runtime == "external-compose"
        || recipe.install.as_ref().and_then(|install| install.repo.as_ref()).is_some();
    let requires_network = bool_requirement(recipe, "network")
        || recipe.install.as_ref().and_then(|install| install.package.as_ref()).is_some()
        || recipe.install.as_ref().and_then(|install| install.repo.as_ref()).is_some()
        || recipe.version_source.is_some();

    let mut plan = RecipeInstallPlan {
        app_id: recipe.id.clone(),
        name: recipe.name.clone(),
        plan_version: plan_version(recipe),
        plan_digest: String::new(),
        runtime: recipe.runtime.clone(),
        summary: recipe.summary.clone(),
        install_strategy: recipe.install.as_ref().map(|install| install.strategy.clone()),
        start_strategy: recipe.start.as_ref().and_then(|start| start.strategy.clone()),
        dashboard_url: recipe.dashboard_url(),
        risk_level: risk_level(recipe),
        estimated_time: estimated_time(recipe),
        estimated_disk: estimated_disk(recipe),
        requires_network,
        requires_docker,
        requires_node,
        requires_git,
        recommended_memory_gb: u64_requirement(recipe, "memoryGbRecommended"),
        recommended_cpu: u64_requirement(recipe, "cpuRecommended"),
        ports: recipe.ports.clone(),
        downloads: downloads(recipe),
        directories: directories(app, recipe)?,
        commands: commands(recipe),
        secrets: secrets(recipe),
        permissions: permissions(recipe),
        checks: checks(recipe),
        rollback: rollback(recipe),
        warnings: warnings(recipe),
        notes: notes(recipe),
    };
    plan.plan_digest = digest(&plan)?;
    Ok(plan)
}
