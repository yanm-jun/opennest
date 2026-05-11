use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::status::RecipeSummary;

const REGISTRY_JSON: &str = include_str!("../../../registry/apps.json");
static RECIPES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../recipes");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeRegistry {
    pub schema_version: String,
    pub apps: Vec<RecipeRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeRegistryEntry {
    pub id: String,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecipeTemplateApp {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub category: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecipeTemplateRequirements {
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub memory: Option<String>,
    #[serde(default)]
    pub disk: Option<String>,
    #[serde(default)]
    pub docker_required: bool,
    #[serde(default)]
    pub node_required: bool,
    #[serde(default)]
    pub git_required: bool,
    #[serde(default)]
    pub gpu_required: bool,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub node: Option<Value>,
    #[serde(default)]
    pub network: Option<bool>,
    #[serde(default)]
    pub memory_gb_recommended: Option<u64>,
    #[serde(default)]
    pub cpu_recommended: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecipeTemplateRecipe {
    pub runtime_type: String,
    #[serde(default)]
    pub version_source: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub paths: Option<Value>,
    #[serde(default)]
    pub install: Option<RecipeInstallSpec>,
    #[serde(default)]
    pub start: Option<RecipeActionSpec>,
    #[serde(default)]
    pub stop: Option<RecipeActionSpec>,
    #[serde(default)]
    pub dashboard: Option<RecipeDashboardSpec>,
    #[serde(default)]
    pub logs: Option<RecipeLogsSpec>,
    #[serde(default)]
    pub onboarding: Option<RecipeActionSpec>,
    #[serde(default)]
    pub doctor: Option<RecipeActionSpec>,
    #[serde(default)]
    pub secrets: Vec<Value>,
    #[serde(default)]
    pub permissions: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenNestRecipe {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    pub runtime: String,
    pub category: String,
    #[serde(default)]
    pub version_source: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
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
    pub ports: Vec<u16>,
    #[serde(default)]
    pub requirements: Option<Value>,
    #[serde(default)]
    pub install_plan_template: Option<Value>,
    #[serde(default)]
    pub runtime_defaults: Option<Value>,
    #[serde(default)]
    pub paths: Option<Value>,
    #[serde(default)]
    pub install: Option<RecipeInstallSpec>,
    #[serde(default)]
    pub start: Option<RecipeActionSpec>,
    #[serde(default)]
    pub stop: Option<RecipeActionSpec>,
    #[serde(default)]
    pub dashboard: Option<RecipeDashboardSpec>,
    #[serde(default)]
    pub logs: Option<RecipeLogsSpec>,
    #[serde(default)]
    pub onboarding: Option<RecipeActionSpec>,
    #[serde(default)]
    pub doctor: Option<RecipeActionSpec>,
    #[serde(default)]
    pub secrets: Vec<Value>,
    #[serde(default)]
    pub permissions: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeInstallSpec {
    pub strategy: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub binary_windows: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default, rename = "ref")]
    pub git_ref: Option<String>,
    #[serde(default)]
    pub compose_dir: Option<String>,
    #[serde(default)]
    pub env_example: Option<String>,
    #[serde(default)]
    pub env_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeActionSpec {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub healthcheck: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeDashboardSpec {
    pub strategy: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub fallback_url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeLogsSpec {
    #[serde(default)]
    pub strategy: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub tail: Option<u32>,
}

impl OpenNestRecipe {
    pub fn dashboard_url(&self) -> Option<String> {
        self.dashboard.as_ref().and_then(|dashboard| {
            dashboard.url.clone().or_else(|| dashboard.fallback_url.clone())
        })
    }

    pub fn primary_port(&self) -> Option<u16> {
        self.ports.first().copied()
    }

    pub fn to_summary(&self, featured: bool) -> RecipeSummary {
        RecipeSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            runtime: self.runtime.clone(),
            ports: self.ports.clone(),
            featured,
            icon: self.icon.clone(),
            screenshots: self.screenshots.clone(),
            tags: self.tags.clone(),
            difficulty: self.difficulty.clone(),
            priority: self.priority.clone(),
            homepage: self.homepage.clone(),
            source_url: self.source_url.clone(),
        }
    }
}

pub fn load_registry() -> Result<RecipeRegistry, String> {
    serde_json::from_str(REGISTRY_JSON).map_err(|error| format!("Invalid embedded registry/apps.json: {error}"))
}

pub fn list_recipes() -> Result<Vec<RecipeSummary>, String> {
    load_user_recipes();
    let registry = load_registry()?;
    let mut result = Vec::new();

    let user_recipes = load_user_recipes();
    for user in &user_recipes {
        result.push(user.to_summary(false));
    }

    for entry in registry.apps {
        let recipe = load_recipe(&entry.id)?;
        result.push(recipe.to_summary(entry.featured));
    }

    Ok(result)
}

pub fn load_recipe(app_id: &str) -> Result<OpenNestRecipe, String> {
    if let Some(recipe) = user_recipes_map().as_ref().and_then(|m| m.get(app_id).cloned()) {
        return Ok(recipe);
    }
    let registry = load_registry()?;
    let entry = registry
        .apps
        .iter()
        .find(|entry| entry.id == app_id)
        .ok_or_else(|| format!("Unknown recipe app: {app_id}"))?;

    let app: RecipeTemplateApp = recipe_json(app_id, "app.json")?;
    let requirements: RecipeTemplateRequirements = recipe_json(app_id, "requirements.json")?;
    let recipe: RecipeTemplateRecipe = recipe_json(app_id, "recipe.json")?;
    let install_plan_template: Value = recipe_json(app_id, "install-plan.json")?;
    let runtime_defaults: Value = recipe_json(app_id, "runtime.json")?;

    if app.id != entry.id {
        return Err(format!("Recipe id mismatch: registry id={} but app.json id={}", entry.id, app.id));
    }

    Ok(OpenNestRecipe {
        schema_version: "2.0.0".to_string(),
        id: app.id,
        name: app.name,
        summary: app.summary.unwrap_or_else(|| app.description.clone()),
        description: Some(app.description),
        runtime: recipe.runtime_type,
        category: app.category,
        version_source: recipe.version_source,
        homepage: app.homepage,
        source_url: app.source_url,
        license: recipe.license,
        icon: app.icon,
        screenshots: app.screenshots,
        tags: app.tags,
        difficulty: app.difficulty,
        priority: app.priority,
        ports: requirements.ports.clone(),
        requirements: Some(serde_json::to_value(requirements).map_err(|error| format!("failed to serialize requirements.json for {app_id}: {error}"))?),
        install_plan_template: Some(install_plan_template),
        runtime_defaults: Some(runtime_defaults),
        paths: recipe.paths,
        install: recipe.install,
        start: recipe.start,
        stop: recipe.stop,
        dashboard: recipe.dashboard,
        logs: recipe.logs,
        onboarding: recipe.onboarding,
        doctor: recipe.doctor,
        secrets: recipe.secrets,
        permissions: recipe.permissions,
    })
}

fn recipe_json<T: for<'de> Deserialize<'de>>(app_id: &str, file_name: &str) -> Result<T, String> {
    let file = RECIPES_DIR
        .get_file(format!("{app_id}/{file_name}"))
        .ok_or_else(|| format!("Missing recipe template file: recipes/{app_id}/{file_name}"))?;
    serde_json::from_slice(file.contents())
        .map_err(|error| format!("Invalid recipes/{app_id}/{file_name}: {error}"))
}

pub fn compose_content_for(recipe: &OpenNestRecipe) -> Result<&'static str, String> {
    let source = recipe
        .install
        .as_ref()
        .and_then(|install| install.source.as_deref())
        .ok_or_else(|| format!("Recipe {} does not define install.source", recipe.id))?;

    let normalized = source.replace('\\', "/").trim_start_matches("./").to_string();
    let relative = normalized.strip_prefix("recipes/").unwrap_or(&normalized);
    let file = RECIPES_DIR
        .get_file(relative)
        .ok_or_else(|| format!("Recipe {} references missing compose source: {}", recipe.id, source))?;
    file.contents_utf8()
        .ok_or_else(|| format!("Compose source for {} is not valid UTF-8: {}", recipe.id, source))
}

pub fn health_host_port(recipe: &OpenNestRecipe) -> Option<(String, u16)> {
    let from_start = recipe
        .start
        .as_ref()
        .and_then(|start| start.healthcheck.as_deref())
        .and_then(parse_localhost_port);
    if from_start.is_some() {
        return from_start;
    }

    recipe.primary_port().map(|port| ("127.0.0.1".to_string(), port))
}

pub fn parse_localhost_port(url: &str) -> Option<(String, u16)> {
    let stripped = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let host_port = stripped.split('/').next().unwrap_or(stripped);
    let mut pieces = host_port.split(':');
    let host = pieces.next()?.to_string();
    let port = pieces.next()?.parse::<u16>().ok()?;

    let normalized_host = match host.as_str() {
        "localhost" => "127.0.0.1".to_string(),
        _ => host,
    };

    Some((normalized_host, port))
}

// --- User recipe support ---

static USER_RECIPES: std::sync::Mutex<Option<std::collections::HashMap<String, OpenNestRecipe>>> = std::sync::Mutex::new(None);

fn user_recipes_map() -> std::sync::MutexGuard<'static, Option<std::collections::HashMap<String, OpenNestRecipe>>> {
    let mut guard = USER_RECIPES.lock().unwrap();
    if guard.is_none() {
        *guard = Some(std::collections::HashMap::new());
    }
    guard
}

fn user_recipes_dir() -> Result<std::path::PathBuf, String> {
    let dir = match std::env::var_os("APPDATA") {
        Some(appdata) => std::path::PathBuf::from(appdata).join("OpenNest").join("user-recipes"),
        None => return Err("APPDATA not available on this platform".to_string()),

    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create user-recipes dir: {e}"))?;
    Ok(dir)
}

pub fn import_user_recipe(recipe_json: &str) -> Result<OpenNestRecipe, String> {
    let recipe: OpenNestRecipe = serde_json::from_str(recipe_json)
        .map_err(|e| format!("invalid recipe JSON: {e}"))?;

    if recipe.id.is_empty() || recipe.name.is_empty() || recipe.runtime.is_empty() {
        return Err("recipe must have id, name, and runtime fields".to_string());
    }

    let valid_runtimes = ["native-cli", "docker-compose", "external-compose", "webview", "mcp-server", "agent-container"];
    if !valid_runtimes.contains(&recipe.runtime.as_str()) {
        return Err(format!("unsupported runtime '{}'. valid runtimes: {}", recipe.runtime, valid_runtimes.join(", ")));
    }

    let dir = user_recipes_dir()?;
    let recipe_file = dir.join(format!("{}.json", recipe.id));
    let json = serde_json::to_string_pretty(&recipe).map_err(|e| format!("failed to serialize recipe: {e}"))?;
    std::fs::write(&recipe_file, json).map_err(|e| format!("failed to write user recipe: {e}"))?;

    let mut map = user_recipes_map();
    map.as_mut().unwrap().insert(recipe.id.clone(), recipe.clone());

    Ok(recipe)
}

pub fn load_user_recipes() -> Vec<OpenNestRecipe> {
    let dir = match user_recipes_dir() {
        Ok(dir) => dir,
        Err(_) => return vec![],
    };

    let mut recipes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(recipe) = serde_json::from_str::<OpenNestRecipe>(&content) {
                        recipes.push(recipe.clone());
                        let mut map = user_recipes_map();
                        map.as_mut().unwrap().insert(recipe.id.clone(), recipe);
                    }
                }
            }
        }
    }
    recipes
}

pub fn remove_user_recipe(app_id: &str) -> Result<(), String> {
    let dir = user_recipes_dir()?;
    let recipe_file = dir.join(format!("{app_id}.json"));
    if recipe_file.exists() {
        std::fs::remove_file(&recipe_file).map_err(|e| format!("failed to remove user recipe: {e}"))?;
    }
    let mut map = user_recipes_map();
    map.as_mut().unwrap().remove(app_id);
    Ok(())
}
static MARKETPLACE_JSON: &str = include_str!("../../../registry/marketplace/index.json");

pub fn fetch_marketplace_recipes() -> Result<Vec<OpenNestRecipe>, String> {
    let recipes: Vec<OpenNestRecipe> = serde_json::from_str(MARKETPLACE_JSON)
        .map_err(|e| format!("invalid marketplace JSON: {e}"))?;

    Ok(recipes)
}
