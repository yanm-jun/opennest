use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

use super::logs;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub ok: bool,
    pub host: String,
    pub port: u16,
    pub checked_at: String,
    pub latency_ms: Option<u128>,
    pub error: Option<String>,
}

impl HealthReport {
    pub fn healthy(host: &str, port: u16, latency_ms: u128) -> Self {
        Self {
            ok: true,
            host: host.to_string(),
            port,
            checked_at: Utc::now().to_rfc3339(),
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    pub fn unhealthy(host: &str, port: u16, error: impl Into<String>) -> Self {
        Self {
            ok: false,
            host: host.to_string(),
            port,
            checked_at: Utc::now().to_rfc3339(),
            latency_ms: None,
            error: Some(error.into()),
        }
    }
}

pub fn check_tcp(host: &str, port: u16, timeout_ms: u64) -> HealthReport {
    let started = Instant::now();
    let addr_text = format!("{host}:{port}");

    let mut addrs = match addr_text.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(error) => return HealthReport::unhealthy(host, port, format!("failed to resolve {addr_text}: {error}")),
    };

    let Some(addr) = addrs.next() else {
        return HealthReport::unhealthy(host, port, format!("no socket address resolved for {addr_text}"));
    };

    match TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)) {
        Ok(_) => HealthReport::healthy(host, port, started.elapsed().as_millis()),
        Err(error) => HealthReport::unhealthy(host, port, format!("tcp connect failed for {addr_text}: {error}")),
    }
}

pub fn wait_for_tcp(
    app: &AppHandle,
    app_id: &str,
    host: &str,
    port: u16,
    total_timeout_ms: u64,
    interval_ms: u64,
) -> Result<HealthReport, String> {
    let start = Instant::now();
    let deadline = Duration::from_millis(total_timeout_ms);
    let interval = Duration::from_millis(interval_ms.max(100));
    let mut last_report = HealthReport::unhealthy(host, port, "healthcheck has not run yet");

    logs::append(
        app,
        app_id,
        "healthcheck",
        &format!("waiting for {host}:{port} for up to {total_timeout_ms}ms"),
    )?;

    while start.elapsed() <= deadline {
        let report = check_tcp(host, port, 750);
        if report.ok {
            logs::append(
                app,
                app_id,
                "healthcheck",
                &format!(
                    "healthy: {}:{} latency={}ms",
                    report.host,
                    report.port,
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
        .unwrap_or_else(|| format!("{host}:{port} did not become reachable before timeout"));
    logs::append(
        app,
        app_id,
        "healthcheck",
        &format!("unhealthy after {total_timeout_ms}ms: {error}"),
    )?;
    Err(error)
}
