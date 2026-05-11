use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::thread;
use tauri::AppHandle;

use super::logs;
use super::recipe_loader::OpenNestRecipe;
use super::port_resolver;
use super::status::RecipeStatus;
use super::status_store;

const DEFAULT_HTTP_TIMEOUT_MS: u64 = 1_500;
const DEFAULT_READY_TOTAL_TIMEOUT_MS: u64 = 60_000;
const DEFAULT_READY_INTERVAL_MS: u64 = 1_500;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessUrl {
    pub raw: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpReadinessReport {
    pub ok: bool,
    pub url: String,
    pub checked_at: String,
    pub latency_ms: Option<u128>,
    pub status_code: Option<u16>,
    pub error: Option<String>,
}

impl HttpReadinessReport {
    fn ready(url: &str, latency_ms: u128, status_code: u16) -> Self {
        Self {
            ok: true,
            url: url.to_string(),
            checked_at: Utc::now().to_rfc3339(),
            latency_ms: Some(latency_ms),
            status_code: Some(status_code),
            error: None,
        }
    }

    fn not_ready(url: &str, error: impl Into<String>, status_code: Option<u16>) -> Self {
        Self {
            ok: false,
            url: url.to_string(),
            checked_at: Utc::now().to_rfc3339(),
            latency_ms: None,
            status_code,
            error: Some(error.into()),
        }
    }
}

pub fn readiness_url_for(app: &AppHandle, recipe: &OpenNestRecipe) -> Option<String> {
    port_resolver::effective_readiness_url(app, recipe)
}

pub fn parse_readiness_url(url: &str) -> Result<ReadinessUrl, String> {
    let (scheme, rest) = if let Some(rest) = url.strip_prefix("http://") {
        ("http".to_string(), rest)
    } else if let Some(rest) = url.strip_prefix("https://") {
        ("https".to_string(), rest)
    } else {
        return Err(format!("Readiness URL must start with http:// or https://: {url}"));
    };

    let (host_port, path_part) = match rest.split_once('/') {
        Some((host_port, path)) => (host_port, format!("/{path}")),
        None => (rest, "/".to_string()),
    };

    let default_port = if scheme == "https" { 443 } else { 80 };
    let (host, port) = match host_port.rsplit_once(':') {
        Some((host, port_text)) if !host.contains(']') => {
            let port = port_text
                .parse::<u16>()
                .map_err(|error| format!("Invalid readiness URL port in {url}: {error}"))?;
            (host.to_string(), port)
        }
        _ => (host_port.to_string(), default_port),
    };

    if host.trim().is_empty() {
        return Err(format!("Readiness URL is missing host: {url}"));
    }

    let normalized_host = match host.as_str() {
        "localhost" => "127.0.0.1".to_string(),
        _ => host,
    };

    Ok(ReadinessUrl {
        raw: url.to_string(),
        scheme,
        host: normalized_host,
        port,
        path: if path_part.is_empty() { "/".to_string() } else { path_part },
    })
}

fn status_code_is_ready(code: u16) -> bool {
    // 2xx/3xx mean the UI is reachable. 401/403 are also acceptable because
    // some apps protect dashboards behind a login wall. 404/5xx should not be
    // considered ready for a recipe dashboard URL.
    (200..400).contains(&code) || code == 401 || code == 403
}

pub fn check_url(url: &str, timeout_ms: u64) -> HttpReadinessReport {
    let parsed = match parse_readiness_url(url) {
        Ok(parsed) => parsed,
        Err(error) => return HttpReadinessReport::not_ready(url, error, None),
    };

    if parsed.scheme != "http" {
        // The current lightweight checker avoids TLS dependencies. HTTPS dashboard URLs still
        // need a future reqwest/native-tls implementation.
        return HttpReadinessReport::not_ready(
            url,
            format!("HTTPS readiness is not implemented in the lightweight checker: {url}"),
            None,
        );
    }

    let started = Instant::now();
    let addr_text = format!("{}:{}", parsed.host, parsed.port);
    let mut addrs = match addr_text.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(error) => return HttpReadinessReport::not_ready(url, format!("failed to resolve {addr_text}: {error}"), None),
    };

    let Some(addr) = addrs.next() else {
        return HttpReadinessReport::not_ready(url, format!("no socket address resolved for {addr_text}"), None);
    };

    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)) {
        Ok(stream) => stream,
        Err(error) => return HttpReadinessReport::not_ready(url, format!("tcp connect failed for {addr_text}: {error}"), None),
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(timeout_ms)));

    let host_header = if parsed.port == 80 {
        parsed.host.clone()
    } else {
        format!("{}:{}", parsed.host, parsed.port)
    };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: OpenNest-Readiness/1.0\r\nAccept: text/html,*/*\r\nConnection: close\r\n\r\n",
        parsed.path,
        host_header
    );

    if let Err(error) = stream.write_all(request.as_bytes()) {
        return HttpReadinessReport::not_ready(url, format!("failed to send HTTP request: {error}"), None);
    }

    let mut buffer = [0_u8; 2048];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes_read) if bytes_read > 0 => bytes_read,
        Ok(_) => return HttpReadinessReport::not_ready(url, "HTTP server closed connection without response", None),
        Err(error) => return HttpReadinessReport::not_ready(url, format!("failed to read HTTP response: {error}"), None),
    };

    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    let status_line = response.lines().next().unwrap_or_default();
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|part| part.parse::<u16>().ok());

    match status_code {
        Some(code) if status_code_is_ready(code) => HttpReadinessReport::ready(url, started.elapsed().as_millis(), code),
        Some(code) => HttpReadinessReport::not_ready(
            url,
            format!("HTTP readiness returned status {code}; expected 2xx/3xx/401/403"),
            Some(code),
        ),
        None => HttpReadinessReport::not_ready(url, format!("could not parse HTTP status line: {status_line}"), None),
    }
}

pub fn wait_for_url(
    app: &AppHandle,
    app_id: &str,
    url: &str,
    total_timeout_ms: u64,
    interval_ms: u64,
) -> Result<HttpReadinessReport, String> {
    let started = Instant::now();
    let deadline = Duration::from_millis(total_timeout_ms);
    let interval = Duration::from_millis(interval_ms.max(250));
    let mut last_report = HttpReadinessReport::not_ready(url, "readiness has not run yet", None);

    logs::append(app, app_id, "readiness", &format!("waiting for HTTP readiness url={url} timeout_ms={total_timeout_ms}"))?;

    while started.elapsed() <= deadline {
        let report = check_url(url, DEFAULT_HTTP_TIMEOUT_MS);
        if report.ok {
            logs::append(
                app,
                app_id,
                "readiness",
                &format!(
                    "ready url={} status={} latency_ms={}",
                    report.url,
                    report.status_code.unwrap_or_default(),
                    report.latency_ms.unwrap_or_default()
                ),
            )?;
            return Ok(report);
        }
        last_report = report;
        thread::sleep(interval);
    }

    let error = last_report
        .error
        .clone()
        .unwrap_or_else(|| format!("{url} did not become HTTP-ready before timeout"));
    logs::append(app, app_id, "readiness", &format!("not ready after {total_timeout_ms}ms url={url}: {error}"))?;
    Err(error)
}

pub fn ensure_ready(app: &AppHandle, recipe: &OpenNestRecipe) -> Result<RecipeStatus, String> {
    let Some(url) = readiness_url_for(app, recipe) else {
        return status_store::mark_readiness_unknown(app, &recipe.id, "Recipe does not define start.healthcheck or dashboard url.");
    };

    match wait_for_url(app, &recipe.id, &url, DEFAULT_READY_TOTAL_TIMEOUT_MS, DEFAULT_READY_INTERVAL_MS) {
        Ok(report) => status_store::mark_http_ready(app, &recipe.id, &url, report.status_code, report.latency_ms),
        Err(error) => {
            let _ = status_store::mark_http_not_ready(app, &recipe.id, &url, error.clone(), None);
            Err(error)
        }
    }
}

pub fn check_once(app: &AppHandle, recipe: &OpenNestRecipe, status: RecipeStatus) -> RecipeStatus {
    let Some(url) = readiness_url_for(app, recipe) else {
        return status;
    };

    let report = check_url(&url, DEFAULT_HTTP_TIMEOUT_MS);
    if report.ok {
        status_store::mark_http_ready(app, &recipe.id, &url, report.status_code, report.latency_ms)
            .unwrap_or(status)
    } else {
        let error = report.error.clone().unwrap_or_else(|| format!("{url} is not HTTP-ready."));
        status_store::mark_http_not_ready(app, &recipe.id, &url, error, report.status_code)
            .unwrap_or(status)
    }
}
