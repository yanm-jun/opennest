use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use super::{logs, secret_redaction_registry, status_store};

pub const EVENT_NAME: &str = "opennest://recipe-progress";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeProgressEvent {
    pub app_id: String,
    pub operation_id: String,
    pub operation: String,
    pub phase: String,
    pub state: String,
    pub message: String,
    pub step: u32,
    pub total_steps: u32,
    pub percent: u8,
    pub timestamp: String,
    pub error: Option<String>,
}

impl RecipeProgressEvent {
    pub fn new(
        app_id: &str,
        operation_id: &str,
        operation: &str,
        phase: &str,
        state: &str,
        message: impl Into<String>,
        step: u32,
        total_steps: u32,
        error: Option<String>,
    ) -> Self {
        let safe_message = secret_redaction_registry::redact(&message.into());
        let safe_error = error.map(|value| secret_redaction_registry::redact(&value));
        let percent = percent_for(step, total_steps, state);
        Self {
            app_id: app_id.to_string(),
            operation_id: operation_id.to_string(),
            operation: operation.to_string(),
            phase: phase.to_string(),
            state: state.to_string(),
            message: safe_message,
            step,
            total_steps,
            percent,
            timestamp: Utc::now().to_rfc3339(),
            error: safe_error,
        }
    }
}

fn percent_for(step: u32, total_steps: u32, state: &str) -> u8 {
    if state == "succeeded" {
        return 100;
    }
    if state == "failed" {
        return 100;
    }
    if total_steps == 0 {
        return 0;
    }
    let clamped_step = step.min(total_steps);
    ((clamped_step as f32 / total_steps as f32) * 100.0).round() as u8
}

pub fn operation_id(app_id: &str, operation: &str) -> String {
    format!("{app_id}-{operation}-{}", Utc::now().timestamp_millis())
}

pub fn emit(app: &AppHandle, event: RecipeProgressEvent) {
    let _ = status_store::mark_progress_event(app, &event);
    let log_message = match &event.error {
        Some(error) => format!(
            "{} step {}/{} {} {}: {} | error={}",
            event.operation, event.step, event.total_steps, event.state, event.phase, event.message, error
        ),
        None => format!(
            "{} step {}/{} {} {}: {}",
            event.operation, event.step, event.total_steps, event.state, event.phase, event.message
        ),
    };
    let _ = logs::append(app, &event.app_id, "progress", &log_message);
    let _ = app.emit(EVENT_NAME, event);
}

pub fn begin(app: &AppHandle, app_id: &str, operation: &str, total_steps: u32, message: impl Into<String>) -> String {
    let operation_id = operation_id(app_id, operation);
    emit(
        app,
        RecipeProgressEvent::new(
            app_id,
            &operation_id,
            operation,
            "begin",
            "running",
            message,
            0,
            total_steps,
            None,
        ),
    );
    operation_id
}

pub fn step(
    app: &AppHandle,
    app_id: &str,
    operation_id: &str,
    operation: &str,
    phase: &str,
    step: u32,
    total_steps: u32,
    message: impl Into<String>,
) {
    emit(
        app,
        RecipeProgressEvent::new(
            app_id,
            operation_id,
            operation,
            phase,
            "running",
            message,
            step,
            total_steps,
            None,
        ),
    );
}

pub fn succeeded(
    app: &AppHandle,
    app_id: &str,
    operation_id: &str,
    operation: &str,
    total_steps: u32,
    message: impl Into<String>,
) {
    emit(
        app,
        RecipeProgressEvent::new(
            app_id,
            operation_id,
            operation,
            "complete",
            "succeeded",
            message,
            total_steps,
            total_steps,
            None,
        ),
    );
}

pub fn failed(
    app: &AppHandle,
    app_id: &str,
    operation_id: &str,
    operation: &str,
    total_steps: u32,
    message: impl Into<String>,
    error: impl Into<String>,
) {
    emit(
        app,
        RecipeProgressEvent::new(
            app_id,
            operation_id,
            operation,
            "failed",
            "failed",
            message,
            total_steps,
            total_steps,
            Some(error.into()),
        ),
    );
}

pub fn finish_from_result(
    app: &AppHandle,
    app_id: &str,
    operation_id: &str,
    operation: &str,
    total_steps: u32,
    result_ok: bool,
    message: Option<String>,
    error: Option<String>,
) {
    if result_ok {
        succeeded(
            app,
            app_id,
            operation_id,
            operation,
            total_steps,
            message.unwrap_or_else(|| format!("{operation} completed.")),
        );
    } else {
        failed(
            app,
            app_id,
            operation_id,
            operation,
            total_steps,
            format!("{operation} failed."),
            error.unwrap_or_else(|| "Unknown error".to_string()),
        );
    }
}
