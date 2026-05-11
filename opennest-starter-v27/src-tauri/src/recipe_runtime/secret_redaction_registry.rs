use std::sync::{Mutex, OnceLock};

/// Process-local registry of exact secret values that must never be written to logs,
/// status.json, or command results returned to the UI.
///
/// Important security boundary:
/// - This registry is intentionally in-memory only.
/// - Do NOT persist these raw values to disk.
/// - Re-register values whenever they are saved to / read from Credential Store.
static SECRET_REGISTRY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn registry() -> &'static Mutex<Vec<String>> {
    SECRET_REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

fn normalize(value: &str) -> Option<String> {
    let trimmed = value.trim();
    // Avoid redacting tiny/common values that would destroy useful logs.
    if trimmed.len() < 6 {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn register_secret(value: &str) {
    let Some(secret) = normalize(value) else {
        return;
    };

    if let Ok(mut guard) = registry().lock() {
        if !guard.iter().any(|existing| existing == &secret) {
            guard.push(secret);
            // Longest first avoids partial replacement leaks when one token contains another.
            guard.sort_by(|a, b| b.len().cmp(&a.len()));
        }
    }
}

pub fn register_secret_values<'a>(values: impl IntoIterator<Item = &'a str>) {
    for value in values {
        register_secret(value);
    }
}

pub fn registered_count() -> usize {
    registry().lock().map(|guard| guard.len()).unwrap_or(0)
}

fn is_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | ':' | '/' | '=')
}

fn redact_marker(mut input: String, marker: &str) -> String {
    let mut search_from = 0;
    while let Some(relative_pos) = input[search_from..].find(marker) {
        let start = search_from + relative_pos;
        let mut end = start + marker.len();
        for (offset, ch) in input[end..].char_indices() {
            if is_token_char(ch) {
                end = start + marker.len() + offset + ch.len_utf8();
            } else {
                break;
            }
        }
        // If only the marker was present, still redact a short segment to avoid leaking prefixes.
        if end <= start + marker.len() {
            end = (start + marker.len()).min(input.len());
        }
        input.replace_range(start..end, "[REDACTED]");
        search_from = start + "[REDACTED]".len();
    }
    input
}

pub fn redact(input: &str) -> String {
    let mut out = input.to_string();

    // Exact-value dynamic redaction first. This is the main protection: it catches
    // custom provider tokens and arbitrary user-entered secrets, not only known prefixes.
    let secrets = registry().lock().map(|guard| guard.clone()).unwrap_or_default();
    for secret in secrets {
        if !secret.is_empty() {
            out = out.replace(&secret, "[REDACTED]");
        }
    }

    // Prefix-based fallback catches secrets that appeared before registration or came from tools.
    // Keep markers specific enough to avoid wrecking normal prose.
    for marker in [
        "sk-ant-",
        "sk-proj-",
        "sk-or-v1-",
        "sk-",
        "AIza",
        "xoxb-",
        "xoxp-",
        "ghp_",
        "github_pat_",
        "hf_",
        "OPENAI_API_KEY=",
        "ANTHROPIC_API_KEY=",
        "GEMINI_API_KEY=",
        "OPENROUTER_API_KEY=",
    ] {
        out = redact_marker(out, marker);
    }

    out
}
