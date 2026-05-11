use serde::{Deserialize, Serialize};

use super::secret_redaction_registry;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub runtime: String,
    pub ports: Vec<u16>,
    pub featured: bool,
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
    pub dashboard_url: Option<String>,
    pub last_started_at: Option<String>,
    pub last_stopped_at: Option<String>,
    pub last_error: Option<String>,
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
}

impl RecipeStatus {
    pub fn default_for(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            installed: false,
            install_state: "not_installed".to_string(),
            run_state: "unknown".to_string(),
            dashboard_url: None,
            last_started_at: None,
            last_stopped_at: None,
            last_error: None,
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActionResult {
    pub ok: bool,
    pub app_id: String,
    pub status: Option<RecipeStatus>,
    pub message: Option<String>,
    pub logs: Option<Vec<String>>,
    pub error: Option<String>,
}

impl RuntimeActionResult {
    pub fn ok(app_id: &str, message: impl Into<String>, status: Option<RecipeStatus>) -> Self {
        Self { ok: true, app_id: app_id.to_string(), status, message: Some(message.into()), logs: None, error: None }
    }

    pub fn fail(app_id: &str, error: impl Into<String>, status: Option<RecipeStatus>) -> Self {
        let safe_error = secret_redaction_registry::redact(&error.into());
        Self { ok: false, app_id: app_id.to_string(), status, message: None, logs: None, error: Some(safe_error) }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSecretInput {
    pub id: String,
    pub value: String,
}
