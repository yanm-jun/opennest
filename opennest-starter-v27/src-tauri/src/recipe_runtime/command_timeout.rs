use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use super::secret_redaction_registry;

pub const CHECK_TIMEOUT_MS: u64 = 30_000;
pub const INSTALL_TIMEOUT_MS: u64 = 15 * 60_000;
pub const NODE_RUNTIME_TIMEOUT_MS: u64 = 10 * 60_000;
pub const COMPOSE_UP_TIMEOUT_MS: u64 = 10 * 60_000;
pub const START_TIMEOUT_MS: u64 = 120_000;
pub const STOP_TIMEOUT_MS: u64 = 120_000;
pub const LOGS_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone)]
pub struct TimedCommandOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub duration_ms: u128,
}

impl TimedCommandOutput {
    pub fn failure_message(&self, action: &str) -> String {
        if self.timed_out {
            return format!(
                "{action} timed out after {} ms. The child process was killed.{}{}",
                self.duration_ms,
                suffix(" stdout", &self.stdout),
                suffix(" stderr", &self.stderr)
            );
        }

        format!(
            "{action} failed with exit_code={:?}.{}{}",
            self.exit_code,
            suffix(" stdout", &self.stdout),
            suffix(" stderr", &self.stderr)
        )
    }
}

fn suffix(label: &str, text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        let clipped: String = trimmed.chars().take(4_000).collect();
        let safe = secret_redaction_registry::redact(&clipped);
        format!("\n{label}: {safe}")
    }
}

fn read_pipe<T: Read + Send + 'static>(mut pipe: T) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut text = String::new();
        let _ = pipe.read_to_string(&mut text);
        text
    })
}

pub fn run_with_timeout(mut command: Command, timeout_ms: u64) -> Result<TimedCommandOutput, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to spawn command: {error}"))?;

    let stdout_reader = child.stdout.take().map(read_pipe);
    let stderr_reader = child.stderr.take().map(read_pipe);
    let timeout = Duration::from_millis(timeout_ms);
    let mut timed_out = false;

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    timed_out = true;
                    let _ = child.kill();
                    break child
                        .wait()
                        .map_err(|error| format!("command timed out and failed to wait after kill: {error}"))?;
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed while waiting for command: {error}"));
            }
        }
    };

    let stdout = stdout_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let stderr = stderr_reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    Ok(TimedCommandOutput {
        success: !timed_out && status.success(),
        exit_code: status.code(),
        stdout,
        stderr,
        timed_out,
        duration_ms: start.elapsed().as_millis(),
    })
}
