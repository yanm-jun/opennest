use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::status::RecipeSummary;

const REGISTRY_JSON: &str = include_str!("../../../registry/apps.json");
const OPENCLAW_RECIPE_JSON: &str = include_str!("../../../recipes/openclaw/recipe.opennest.json");
const OPEN_WEBUI_RECIPE_JSON: &str = include_str!("../../../recipes/open-webui/recipe.opennest.json");
const FLOWISE_RECIPE_JSON: &str = include_str!("../../../recipes/flowise/recipe.opennest.json");
const DIFY_RECIPE_JSON: &str = include_str!("../../../recipes/dify/recipe.opennest.json");
const OPEN_WEBUI_COMPOSE: &str = include_str!("../../../recipes/open-webui/docker-compose.yml");
const FLOWISE_COMPOSE: &str = include_str!("../../../recipes/flowise/docker-compose.yml");

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
    pub recipe: String,
    pub runtime: String,
    #[serde(default)]
    pub featured: bool,
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
    pub license: Option<String>,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub requirements: Option<Value>,
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
            dashboard
                .url
                .clone()
                .or_else(|| dashboard.fallback_url.clone())
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
            category: self.category.clone(),
            runtime: self.runtime.clone(),
            ports: self.ports.clone(),
            featured,
        }
    }
}

pub fn load_registry() -> Result<RecipeRegistry, String> {
    serde_json::from_str(REGISTRY_JSON).map_err(|error| format!("Invalid embedded registry/apps.json: {error}"))
}

pub fn list_recipes() -> Result<Vec<RecipeSummary>, String> {
    let registry = load_registry()?;
    let mut result = Vec::new();

    for entry in registry.apps {
        let recipe = load_recipe(&entry.id)?;
        if recipe.runtime != entry.runtime {
            return Err(format!(
                "Registry/runtime mismatch for {}: registry={} recipe={}",
                entry.id, entry.runtime, recipe.runtime
            ));
        }
        result.push(recipe.to_summary(entry.featured));
    }

    Ok(result)
}

pub fn load_recipe(app_id: &str) -> Result<OpenNestRecipe, String> {
    let registry = load_registry()?;
    let entry = registry
        .apps
        .iter()
        .find(|entry| entry.id == app_id)
        .ok_or_else(|| format!("Unknown recipe app: {app_id}"))?;

    let json = embedded_recipe_json(&entry.id)
        .ok_or_else(|| format!("Recipe {} is declared in registry but not embedded in this build.", entry.id))?;
    let recipe: OpenNestRecipe = serde_json::from_str(json)
        .map_err(|error| format!("Invalid recipe JSON for {}: {error}", entry.id))?;

    if recipe.id != entry.id {
        return Err(format!(
            "Recipe id mismatch: registry id={} but recipe id={}",
            entry.id, recipe.id
        ));
    }
    if recipe.runtime != entry.runtime {
        return Err(format!(
            "Recipe runtime mismatch for {}: registry={} recipe={}",
            entry.id, entry.runtime, recipe.runtime
        ));
    }

    Ok(recipe)
}

fn embedded_recipe_json(app_id: &str) -> Option<&'static str> {
    match app_id {
        "openclaw" => Some(OPENCLAW_RECIPE_JSON),
        "open-webui" => Some(OPEN_WEBUI_RECIPE_JSON),
        "flowise" => Some(FLOWISE_RECIPE_JSON),
        "dify" => Some(DIFY_RECIPE_JSON),
        _ => None,
    }
}

pub fn compose_content_for(recipe: &OpenNestRecipe) -> Result<&'static str, String> {
    let source = recipe
        .install
        .as_ref()
        .and_then(|install| install.source.as_deref())
        .ok_or_else(|| format!("Recipe {} does not define install.source", recipe.id))?;

    match source {
        "recipes/open-webui/docker-compose.yml" => Ok(OPEN_WEBUI_COMPOSE),
        "recipes/flowise/docker-compose.yml" => Ok(FLOWISE_COMPOSE),
        _ => Err(format!(
            "Recipe {} references unsupported compose source: {}. Add it to recipe_loader.rs before enabling install.",
            recipe.id, source
        )),
    }
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
