//! Subprocess worker speaking NDJSON on stdin/stdout.

use parallax_core::{ErrorCode, ParallaxError, Remediation, RuntimeKind};
use parallax_protocol::Envelope;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

/// Default max NDJSON line size when callers do not supply a tighter bound.
const DEFAULT_MAX_MESSAGE_BYTES: u64 = 16_777_216;

/// Managed worker subprocess.
pub struct WorkerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    runtime: RuntimeKind,
    /// Bounded stderr capture (drained on a background task to avoid pipe deadlock).
    stderr_buf: Arc<Mutex<String>>,
    stderr_task: Option<tokio::task::JoinHandle<()>>,
}

impl WorkerProcess {
    /// Spawn a worker.
    pub async fn spawn(
        program: PathBuf,
        args: &[String],
        runtime: RuntimeKind,
    ) -> parallax_core::Result<Self> {
        let mut cmd = Command::new(&program);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            // Prevent console window flashes when possible.
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = cmd.spawn().map_err(|e| {
            ParallaxError::new(
                ErrorCode::RuntimeUnavailable,
                format!("failed to spawn {:?} worker: {e}", runtime),
            )
            .with_runtime(runtime.clone())
            .with_source("parallax-runtime")
            .with_operation("WorkerProcess::spawn")
            .remediate(Remediation::new(
                "Install the runtime or ensure it is on PATH",
            ))
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            ParallaxError::new(ErrorCode::AdapterCrashed, "worker stdin missing")
                .with_runtime(runtime.clone())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            ParallaxError::new(ErrorCode::AdapterCrashed, "worker stdout missing")
                .with_runtime(runtime.clone())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            ParallaxError::new(ErrorCode::AdapterCrashed, "worker stderr missing")
                .with_runtime(runtime.clone())
        })?;

        // Drain stderr so a chatty/crashing worker cannot fill the OS pipe and deadlock.
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        let stderr_buf_task = Arc::clone(&stderr_buf);
        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut chunk = [0u8; 4096];
            const MAX_STDERR: usize = 64 * 1024;
            loop {
                match reader.read(&mut chunk).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut guard = stderr_buf_task.lock().await;
                        if guard.len() < MAX_STDERR {
                            let take = (MAX_STDERR - guard.len()).min(n);
                            guard.push_str(&String::from_utf8_lossy(&chunk[..take]));
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            runtime,
            stderr_buf,
            stderr_task: Some(stderr_task),
        })
    }

    /// Send a request and wait for a matching response.
    pub async fn request(
        &mut self,
        env: Envelope,
        wait: Duration,
    ) -> parallax_core::Result<Envelope> {
        self.request_bounded(env, wait, DEFAULT_MAX_MESSAGE_BYTES)
            .await
    }

    /// Send a request with an explicit max response line size.
    pub async fn request_bounded(
        &mut self,
        env: Envelope,
        wait: Duration,
        max_message_bytes: u64,
    ) -> parallax_core::Result<Envelope> {
        let id = env.id;
        let line = env.to_ndjson_line()?;
        if line.len() as u64 > max_message_bytes {
            return Err(ParallaxError::new(
                ErrorCode::ResourceLimitExceeded,
                format!(
                    "outbound protocol message exceeds max_message_bytes ({})",
                    max_message_bytes
                ),
            )
            .with_runtime(self.runtime.clone())
            .with_operation("WorkerProcess::request"));
        }
        self.stdin.write_all(line.as_bytes()).await.map_err(|e| {
            ParallaxError::new(ErrorCode::AdapterCrashed, format!("write failed: {e}"))
                .with_runtime(self.runtime.clone())
                .with_source("parallax-runtime")
                .with_operation("WorkerProcess::request")
        })?;
        self.stdin.flush().await.map_err(|e| {
            ParallaxError::new(ErrorCode::AdapterCrashed, format!("flush failed: {e}"))
                .with_runtime(self.runtime.clone())
        })?;

        let runtime = self.runtime.clone();
        let max_message_bytes = max_message_bytes as usize;
        let read_fut = async {
            loop {
                let mut buf = String::new();
                let n = self.stdout.read_line(&mut buf).await.map_err(|e| {
                    ParallaxError::new(ErrorCode::AdapterCrashed, format!("read failed: {e}"))
                        .with_runtime(runtime.clone())
                })?;
                if n == 0 {
                    let stderr = self.stderr_buf.lock().await.clone();
                    return Err(ParallaxError::new(
                        ErrorCode::AdapterCrashed,
                        "worker closed stdout",
                    )
                    .with_runtime(runtime.clone())
                    .with_diagnostic(stderr));
                }
                if buf.len() > max_message_bytes {
                    return Err(ParallaxError::new(
                        ErrorCode::ResourceLimitExceeded,
                        format!(
                            "inbound protocol message exceeds max_message_bytes ({max_message_bytes})"
                        ),
                    )
                    .with_runtime(runtime.clone())
                    .with_operation("WorkerProcess::request"));
                }
                let line = buf.trim();
                if line.is_empty() {
                    continue;
                }
                let resp = Envelope::from_ndjson_line(line)?;
                if resp.id == id {
                    return Ok(resp);
                }
                // Mismatched ids are ignored until timeout; a hostile/buggy worker
                // cannot permanently desync a single request beyond `wait`.
            }
        };

        match timeout(wait, read_fut).await {
            Ok(r) => r,
            Err(_) => {
                let _ = self.child.kill().await;
                let stderr = self.stderr_buf.lock().await.clone();
                Err(ParallaxError::new(
                    ErrorCode::ExecutionTimeout,
                    format!("worker request timed out after {:?}", wait),
                )
                .with_runtime(self.runtime.clone())
                .with_operation("WorkerProcess::request")
                .with_diagnostic(stderr))
            }
        }
    }

    /// Graceful shutdown.
    pub async fn shutdown(&mut self) {
        let env = Envelope::request("shutdown", serde_json::json!({}));
        let _ = self.request(env, Duration::from_secs(2)).await;
        let _ = self.child.kill().await;
        if let Some(task) = self.stderr_task.take() {
            let _ = task.await;
        }
    }
}
