//! Continuum CLI helpers — safepoint capture, contract analysis, honest reports.

use parallax_core::{
    ErrorCode, ExecutionLimits, ExecutionRequest, ParallaxError, Remediation, RuntimeKind,
};
use parallax_migrate::{
    analyze_contract, analyze_ues_contract, require_contract, MigrationContract,
};
use parallax_pcir::PcirProgram;
use parallax_runtime::RuntimeManager;
use parallax_security::SandboxPolicy;
use parallax_ues::{continuation_matrix, from_json_bytes, to_json_bytes, UniversalExecutionState};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub async fn cmd_capabilities_continuations(
    json: bool,
    runtime: Option<String>,
) -> Result<(), ParallaxError> {
    let kinds: Vec<RuntimeKind> = if let Some(r) = runtime {
        vec![parse_runtime(&r)?]
    } else {
        vec![
            RuntimeKind::Python,
            RuntimeKind::JavaScript,
            RuntimeKind::Wasm,
        ]
    };
    let matrices: Vec<_> = kinds.into_iter().map(continuation_matrix).collect();
    if json {
        println!("{}", serde_json::to_string_pretty(&matrices)?);
    } else {
        for m in matrices {
            print!("{}", m.format_human());
            println!();
        }
    }
    Ok(())
}

/// Options for `plx continuum`.
pub struct ContinuumOpts {
    /// Machine-readable output.
    pub json: bool,
    /// Program path.
    pub file: PathBuf,
    /// Runtime override.
    pub runtime: Option<String>,
    /// Optional UES output path.
    pub output: Option<PathBuf>,
    /// Resume after capture.
    pub resume: bool,
    /// Contract target runtime.
    pub to: Option<String>,
    /// Contract-only.
    pub analyze_only: bool,
    /// Timeout ms.
    pub timeout_ms: u64,
}

pub async fn cmd_continuum(opts: ContinuumOpts) -> Result<(), ParallaxError> {
    let ContinuumOpts {
        json,
        file,
        runtime,
        output,
        resume,
        to,
        analyze_only,
        timeout_ms,
    } = opts;
    let source_rt = infer_runtime(&file, runtime.as_deref())?;
    let target_rt = if let Some(t) = to.as_deref() {
        parse_runtime(t)?
    } else {
        source_rt.clone()
    };

    let contract = MigrationContract::continuation_checkpoint(source_rt.clone(), target_rt.clone());
    let analysis = analyze_contract(&contract);

    if analyze_only {
        if json {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        } else {
            println!("status: EXPERIMENTAL / contract analysis only");
            print!("{}", analysis.format_human());
        }
        return if analysis.satisfied {
            Ok(())
        } else {
            Err(ParallaxError::new(
                ErrorCode::MigrationRejected,
                "continuum contract not satisfied",
            )
            .context("report", analysis.format_human()))
        };
    }

    // Cross-runtime continuation: always run contract and reject honestly.
    if source_rt != target_rt {
        let _ = require_contract(&contract).map_err(|e| {
            e.remediate(Remediation::with_detail(
                "Use same-runtime continuum capture/resume, or value migrate via `plx migrate`",
                "Cross-runtime continuation resume is Unsupported in this milestone",
            ))
        })?;
    }

    let manager = build_manager();
    let mut req = ExecutionRequest::file(source_rt.clone(), file.to_string_lossy().to_string());
    req.continuum = true;
    req.limits = ExecutionLimits {
        timeout: Duration::from_millis(timeout_ms),
        ..ExecutionLimits::default()
    };
    let policy = {
        let mut p = SandboxPolicy::default();
        p.limits.timeout = Duration::from_millis(timeout_ms);
        p
    };

    let result = manager.execute(req, &policy).await?;
    if !result.suspended {
        return Err(ParallaxError::new(
            ErrorCode::CaptureFailure,
            "program completed without hitting parallax.checkpoint(); no UES captured",
        )
        .with_runtime(source_rt)
        .remediate(Remediation::new(
            "Insert parallax.checkpoint(\"label\") at the desired safepoint",
        )));
    }

    let mut ues_value = result.ues.clone().ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::CaptureFailure,
            "suspended but worker returned no UES",
        )
        .with_runtime(source_rt.clone())
    })?;

    // Enrich with validated PCIR stub when worker omitted it.
    if ues_value.get("pcir").map(|v| v.is_null()).unwrap_or(true) {
        let label = ues_value
            .pointer("/control_state/safepoint_label")
            .and_then(|v| v.as_str())
            .unwrap_or("checkpoint");
        let stub = PcirProgram::checkpoint_stub(label);
        ues_value["pcir"] = serde_json::to_value(&stub)?;
    }

    let ues: UniversalExecutionState = serde_json::from_value(ues_value.clone()).map_err(|e| {
        ParallaxError::new(
            ErrorCode::SerializationFailure,
            format!("invalid UES from worker: {e}"),
        )
        .with_source("parallax-cli")
    })?;
    ues.validate()?;

    let ues_analysis = analyze_ues_contract(&contract, &ues);
    if !ues_analysis.satisfied && source_rt != target_rt {
        return Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "UES failed continuation contract",
        )
        .context("report", ues_analysis.format_human()));
    }

    if let Some(path) = &output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, to_json_bytes(&ues)?)?;
    }

    let mut resume_payload = None;
    if resume {
        if source_rt != target_rt {
            return Err(ParallaxError::new(
                ErrorCode::CapabilityViolation,
                "cross-runtime continuation resume is Unsupported",
            )
            .with_runtime(target_rt)
            .context("level", "NO"));
        }
        // Same-runtime resume only.
        resume_payload = Some(resume_same_runtime(&source_rt, ues_value.clone(), &policy).await?);
    }

    if json {
        let out = serde_json::json!({
            "status": "EXPERIMENTAL",
            "suspended": true,
            "source_runtime": source_rt,
            "target_runtime": target_rt,
            "contract": ues_analysis,
            "safepoint": result.safepoint,
            "ues": ues,
            "stdout": result.stdout,
            "duration_us": result.duration_us,
            "resume": resume_payload,
            "notes": [
                "Arbitrary live stack migration is NOT implemented",
                "Same-runtime checkpoint resume executes post-checkpoint source only",
                "Deterministic replay engine is Unsupported",
            ],
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Continuum status: EXPERIMENTAL");
        println!(
            "runtime: {}  suspended: true  duration: {} µs",
            source_rt, result.duration_us
        );
        if let Some(sp) = &result.safepoint {
            println!("safepoint: {}", serde_json::to_string_pretty(sp)?);
        }
        print!("{}", ues_analysis.format_human());
        if let Some(path) = &output {
            println!("wrote UES {}", path.display());
        }
        if let Some(r) = &resume_payload {
            println!(
                "resume: success={} duration={} µs",
                r["success"], r["duration_us"]
            );
            if let Some(stdout) = r.get("stdout").and_then(|v| v.as_str()) {
                if !stdout.is_empty() {
                    println!("--- resume stdout ---\n{stdout}");
                }
            }
            if let Some(warnings) = r.get("warnings").and_then(|v| v.as_array()) {
                for w in warnings {
                    println!("warning: {w}");
                }
            }
        }
        println!("notes:");
        println!("  - Arbitrary live stack migration is NOT implemented");
        println!("  - Same-runtime resume runs post-checkpoint source only (not a full restart)");
        println!("  - Cross-runtime continuation resume: Unsupported");
    }
    Ok(())
}

pub async fn cmd_migrate_continuation_mode(
    json: bool,
    file: PathBuf,
    to: String,
    from: Option<String>,
    timeout_ms: u64,
) -> Result<(), ParallaxError> {
    // Honest path: analyze contract; do not fake cross-runtime resume.
    let source_rt = infer_runtime(&file, from.as_deref())?;
    let target_rt = parse_runtime(&to)?;
    let contract = MigrationContract::continuation_checkpoint(source_rt.clone(), target_rt.clone());
    let analysis = analyze_contract(&contract);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mode": "continuation",
                "status": if analysis.satisfied { "EXPERIMENTAL" } else { "UNSUPPORTED" },
                "contract": analysis,
                "note": "Use `plx continuum` for explicit checkpoint capture; value migration remains `plx migrate` without --mode continuation",
            }))?
        );
    } else {
        println!("migrate --mode continuation");
        println!(
            "status: {}",
            if analysis.satisfied {
                "EXPERIMENTAL"
            } else {
                "UNSUPPORTED"
            }
        );
        print!("{}", analysis.format_human());
        println!(
            "hint: use `plx continuum {}` for safepoint capture on the source runtime",
            file.display()
        );
    }
    if analysis.satisfied && source_rt == target_rt {
        // Delegate to continuum capture+resume for same-runtime experimental path.
        return cmd_continuum(ContinuumOpts {
            json,
            file,
            runtime: Some(source_rt.as_str().to_string()),
            output: None,
            resume: true,
            to: Some(target_rt.as_str().to_string()),
            analyze_only: false,
            timeout_ms,
        })
        .await;
    }
    Err(ParallaxError::new(
        ErrorCode::MigrationRejected,
        "continuation migration mode not satisfied (see contract report)",
    )
    .context("report", analysis.format_human())
    .remediate(Remediation::new(
        "For value/state PIR migration omit --mode continuation; for Continuum use `plx continuum`",
    )))
}

async fn resume_same_runtime(
    runtime: &RuntimeKind,
    ues: serde_json::Value,
    policy: &SandboxPolicy,
) -> Result<serde_json::Value, ParallaxError> {
    match runtime {
        RuntimeKind::Python => {
            let adapter = parallax_adapter_python::PythonAdapter::new()?;
            let resp = adapter.resume_checkpoint(ues, policy).await?;
            Ok(serde_json::to_value(resp)?)
        }
        RuntimeKind::JavaScript => {
            let adapter = parallax_adapter_js::JsAdapter::new()?;
            let resp = adapter.resume_checkpoint(ues, policy).await?;
            Ok(serde_json::to_value(resp)?)
        }
        other => Err(ParallaxError::new(
            ErrorCode::CapabilityViolation,
            format!("continuation resume unsupported on {other}"),
        )
        .with_runtime(other.clone())),
    }
}

pub fn cmd_inspect_ues(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let bytes = std::fs::read(&path)?;
    let ues = if bytes.starts_with(parallax_ues::UES_MAGIC) {
        parallax_ues::from_binary(&bytes)?
    } else {
        from_json_bytes(&bytes)?
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&ues)?);
    } else {
        println!("UES format_version: {}", ues.format_version);
        println!("execution_id: {}", ues.execution_id);
        println!("source_runtime: {}", ues.source_runtime);
        println!("program: {}", ues.source_program.as_deref().unwrap_or("?"));
        println!(
            "safepoint: {:?} ({:?})",
            ues.control_state.safepoint_label, ues.control_state.safepoint_kind
        );
        println!("frames: {}", ues.call_stack.len());
        println!(
            "continuum_status: {:?}",
            ues.migration_metadata.continuum_status
        );
        println!("replay: {:?}", ues.deterministic_context.engine_status);
        if let Some(reason) = &ues.deterministic_context.unsupported_reason {
            println!("replay note: {reason}");
        }
    }
    Ok(())
}

fn build_manager() -> RuntimeManager {
    let manager = RuntimeManager::new(4);
    parallax_adapter_python::register_lenient(&manager);
    parallax_adapter_js::register_lenient(&manager);
    parallax_adapter_wasm::register_lenient(&manager);
    parallax_connectors::register_all_lenient(&manager);
    manager
}

fn parse_runtime(s: &str) -> Result<RuntimeKind, ParallaxError> {
    RuntimeKind::parse(s).ok_or_else(|| {
        ParallaxError::new(ErrorCode::InvalidArgument, format!("unknown runtime: {s}"))
            .with_operation("parse_runtime")
    })
}

fn infer_runtime(file: &Path, override_rt: Option<&str>) -> Result<RuntimeKind, ParallaxError> {
    if let Some(r) = override_rt {
        return parse_runtime(r);
    }
    RuntimeKind::from_path(file).ok_or_else(|| {
        ParallaxError::new(
            ErrorCode::InvalidArgument,
            format!(
                "cannot infer runtime from {}; pass --runtime",
                file.display()
            ),
        )
    })
}
