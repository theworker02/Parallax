//! CLI command implementations.

use crate::emit::{emit_javascript, emit_python};
use crate::{AdapterCommand, AdaptersCommand, Cli, Commands, VersionFmt};
use chrono::Utc;
use parallax_core::{
    ComponentVersions, ConversionPolicy, ErrorCode, ExecutionLimits, ExecutionRequest,
    ExecutionState, ParallaxError, Remediation, RuntimeKind, RuntimeStatus, PARALLAX_VERSION,
};
use parallax_diagnostics::{DoctorReport, RuntimeHealth};
use parallax_ir::PirDocument;
use parallax_migrate::{migrate_document, MigrationReport};
use parallax_runtime::RuntimeManager;
use parallax_security::SandboxPolicy;
use parallax_snapshot::Snapshot;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub async fn run(cli: Cli) -> Result<(), ParallaxError> {
    let json = cli.json;
    match cli.command {
        Commands::Run {
            file,
            runtime,
            timeout_ms,
            entry,
            capture,
        } => cmd_run(json, file, runtime, timeout_ms, entry, capture).await,
        Commands::Inspect { snapshot } => cmd_inspect(json, snapshot),
        Commands::Snapshot {
            file,
            output,
            runtime,
            capture,
            label,
        } => cmd_snapshot(json, file, output, runtime, capture, label).await,
        Commands::Restore { snapshot, target } => cmd_restore(json, snapshot, target).await,
        Commands::Migrate {
            file,
            to,
            from,
            capture,
            allow_lossy,
            no_prefer_bigint,
            reject_unsupported,
            strict,
            output,
            snapshot,
            pir_input,
            mode,
            dry_run,
            verify,
            target_style,
            preserve_layout,
            require_build,
            require_tests,
            min_confidence,
            fail_on_unsupported,
            keep,
            update,
            project,
        } => {
            if mode == "continuation" {
                crate::continuum::cmd_migrate_continuation_mode(json, file, to, from, 30000).await
            } else if crate::transmute_cmd::should_use_transmute(&file, &mode, project, &to) {
                crate::transmute_cmd::cmd_transmute(
                    json,
                    file,
                    to,
                    from,
                    output,
                    dry_run,
                    verify,
                    strict,
                    target_style,
                    preserve_layout,
                    require_build,
                    require_tests,
                    min_confidence,
                    fail_on_unsupported,
                    keep,
                    update,
                )
                .await
            } else if mode == "value" || mode == "auto" {
                cmd_migrate(
                    json,
                    file,
                    to,
                    from,
                    capture,
                    allow_lossy,
                    no_prefer_bigint,
                    reject_unsupported,
                    strict,
                    output,
                    snapshot,
                    pir_input,
                )
                .await
            } else {
                Err(ParallaxError::new(
                    ErrorCode::InvalidArgument,
                    format!("unknown migrate --mode {mode} (expected auto|value|continuation|project)"),
                ))
            }
        }
        Commands::Origin { location, project } => crate::transmute_cmd::cmd_origin(json, location, project),
        Commands::Continuum {
            file,
            runtime,
            output,
            resume,
            to,
            analyze_only,
            inspect_ues,
            timeout_ms,
        } => {
            if inspect_ues {
                crate::continuum::cmd_inspect_ues(json, file)
            } else {
                crate::continuum::cmd_continuum(crate::continuum::ContinuumOpts {
                    json,
                    file,
                    runtime,
                    output,
                    resume,
                    to,
                    analyze_only,
                    timeout_ms,
                })
                .await
            }
        }
        Commands::Runtimes => cmd_runtimes(json).await,
        Commands::Connectors {
            id,
            pairs,
            family,
            maturity,
        } => crate::connectors_cmd::cmd_connectors(json, id, pairs, family, maturity),
        Commands::Adapters { command } => {
            let sub = match command {
                None => crate::atlas_cmd::AdaptersSub::List { query: None },
                Some(AdaptersCommand::List { query }) => {
                    crate::atlas_cmd::AdaptersSub::List { query }
                }
                Some(AdaptersCommand::Info { id }) => crate::atlas_cmd::AdaptersSub::Info { id },
                Some(AdaptersCommand::Capabilities { id }) => {
                    crate::atlas_cmd::AdaptersSub::Capabilities { id }
                }
                Some(AdaptersCommand::Health) => crate::atlas_cmd::AdaptersSub::Health,
                Some(AdaptersCommand::Update { check }) => {
                    crate::atlas_cmd::AdaptersSub::Update { check }
                }
                Some(AdaptersCommand::Report) => crate::atlas_cmd::AdaptersSub::Report,
            };
            crate::atlas_cmd::cmd_adapters(json, Some(sub))
        }
        Commands::Adapter { command } => {
            let action = match command {
                AdapterCommand::New { name } => crate::atlas_cmd::AdapterToolAction::New { name },
                AdapterCommand::Validate { path } => {
                    crate::atlas_cmd::AdapterToolAction::Validate { path }
                }
            };
            crate::atlas_cmd::cmd_adapter_tool(json, action)
        }
        Commands::Analyze {
            path,
            to,
            write_lock,
        } => crate::atlas_cmd::cmd_analyze(json, path, to, write_lock),
        Commands::Stacks => crate::atlas_cmd::cmd_stacks(json),
        Commands::Mappings { query } => crate::atlas_cmd::cmd_mappings(json, query),
        Commands::Compatibility { source, target } => {
            crate::atlas_cmd::cmd_compatibility(json, source, target)
        }
        Commands::Unsupported { path } => crate::atlas_cmd::cmd_unsupported(json, path),
        Commands::ExplainStack { path, to } => {
            crate::atlas_cmd::cmd_explain_stack(json, path, to)
        }
        Commands::Observe { path } => crate::horizon_cmd::cmd_observe(json, path),
        Commands::Impossible { path, to, strategy } => {
            crate::horizon_cmd::cmd_impossible(json, path, to, strategy)
        }
        Commands::Dissolve { path, to } => crate::horizon_cmd::cmd_dissolve(json, path, to),
        Commands::Debt { path, to } => crate::horizon_cmd::cmd_debt(json, path, to),
        Commands::Detach { path, to } => crate::horizon_cmd::cmd_detach(json, path, to),
        Commands::Reconstruct => crate::horizon_cmd::cmd_reconstruct(json),
        Commands::OptimizeMigration { path, to } => {
            crate::horizon_cmd::cmd_optimize_migration(json, path, to)
        }
        Commands::ExplainBarrier { id, path, to } => {
            crate::horizon_cmd::cmd_explain_barrier(json, path, id, to)
        }
        Commands::Blame { location } => crate::horizon_cmd::cmd_blame(json, location),
        Commands::CherryPick { commit } => crate::horizon_cmd::cmd_cherry_pick(json, commit),
        Commands::Patch { example } => {
            if example {
                crate::horizon_cmd::cmd_patch_example(json)
            } else {
                Err(ParallaxError::new(
                    ErrorCode::InvalidArgument,
                    "pass --example to print a sample .plxp semantic patch",
                ))
            }
        }
        Commands::Capabilities {
            runtime,
            continuations,
        } => {
            if continuations {
                crate::continuum::cmd_capabilities_continuations(json, runtime).await
            } else {
                cmd_capabilities(json, runtime).await
            }
        }
        Commands::Doctor => cmd_doctor(json).await,
        Commands::Bench {
            iterations,
            file,
            to,
        } => cmd_bench(json, iterations, file, to).await,
        Commands::Link {
            source,
            target,
            policy,
        } => crate::mirror_cmd::cmd_link(json, source, target, policy).await,
        Commands::Sync {
            path,
            check,
            reverse,
            lint,
            patch,
            no_verify,
            property,
            deterministic,
        } => {
            crate::mirror_cmd::cmd_sync(
                json,
                path,
                check,
                reverse,
                lint,
                patch,
                no_verify,
                property,
                deterministic,
            )
            .await
        }
        Commands::Status { path } => crate::mirror_cmd::cmd_status(json, path).await,
        Commands::Ci { path } => crate::mirror_cmd::cmd_ci(json, path).await,
        Commands::History { path } => crate::mirror_cmd::cmd_history(json, path),
        Commands::Rollback { path } => crate::mirror_cmd::cmd_rollback(json, path),
        Commands::Explain { location, path } => crate::mirror_cmd::cmd_explain(json, path, location),
        Commands::Why { file, path } => crate::mirror_cmd::cmd_why(json, path, file),
        Commands::Verify {
            path,
            property,
            deterministic,
            performance,
        } => crate::mirror_cmd::cmd_verify(json, path, property, deterministic, performance).await,
        Commands::Version { format } => {
            cmd_version(json || matches!(format, VersionFmt::Json));
            Ok(())
        }
    }
}

fn build_manager() -> RuntimeManager {
    let manager = RuntimeManager::new(4);
    parallax_adapter_python::register_lenient(&manager);
    parallax_adapter_js::register_lenient(&manager);
    parallax_adapter_wasm::register_lenient(&manager);
    // Broad language catalog: scaffolds for Go, Java, Ruby, C#, … (honest Unsupported).
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

fn policy_from_timeout(timeout_ms: u64) -> SandboxPolicy {
    let mut p = SandboxPolicy::default();
    p.limits.timeout = Duration::from_millis(timeout_ms);
    p
}

async fn cmd_run(
    json: bool,
    file: PathBuf,
    runtime: Option<String>,
    timeout_ms: u64,
    entry: Option<String>,
    capture: Option<String>,
) -> Result<(), ParallaxError> {
    let rt = infer_runtime(&file, runtime.as_deref())?;
    let manager = build_manager();
    let capture_names: Vec<String> = capture
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut req = ExecutionRequest::file(rt.clone(), file.to_string_lossy().to_string());
    req.limits = ExecutionLimits {
        timeout: Duration::from_millis(timeout_ms),
        ..ExecutionLimits::default()
    };
    req.entry = entry;
    req.capture_state = !capture_names.is_empty();

    let policy = policy_from_timeout(timeout_ms);
    let result = if capture_names.is_empty() {
        manager.execute(req, &policy).await?
    } else {
        manager
            .execute_and_capture(req, &capture_names, &policy)
            .await?
            .execution
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "runtime: {}  success: {}  duration: {} µs",
            result.runtime, result.success, result.duration_us
        );
        if !result.stdout.is_empty() {
            println!("--- stdout ---\n{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            println!("--- stderr ---\n{}", result.stderr);
        }
        if let Some(ex) = &result.exception {
            println!("exception: {}: {}", ex.type_name, ex.message);
        }
        if let Some(v) = &result.value {
            println!("value: {v}");
        }
        if let Some(state) = &result.state {
            println!("captured bindings: {}", state.heap);
        }
    }
    if result.success {
        Ok(())
    } else {
        Err(
            ParallaxError::new(ErrorCode::ExecutionFailure, "guest execution failed")
                .with_runtime(rt),
        )
    }
}

fn cmd_inspect(json: bool, path: PathBuf) -> Result<(), ParallaxError> {
    let snap = Snapshot::read_from_path(&path)?;
    let summary = snap.inspect_summary();
    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("snapshot: {}", summary.id);
        println!("runtime: {}", summary.runtime);
        println!("created: {}", summary.created_at);
        println!("hash: {}", summary.content_hash);
        println!("bindings: {}", summary.binding_names.join(", "));
        println!("values: {}", summary.value_count);
        for (name, val) in &snap.pir.bindings {
            println!("  {name}: {}", val.summary());
        }
    }
    Ok(())
}

async fn cmd_snapshot(
    json: bool,
    file: PathBuf,
    output: PathBuf,
    runtime: Option<String>,
    capture: String,
    label: Option<String>,
) -> Result<(), ParallaxError> {
    let rt = infer_runtime(&file, runtime.as_deref())?;
    let names: Vec<String> = capture
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let manager = build_manager();
    let mut req = ExecutionRequest::file(rt.clone(), file.to_string_lossy().to_string());
    req.capture_state = true;
    let policy = SandboxPolicy::default();
    let outcome = manager.execute_and_capture(req, &names, &policy).await?;
    if !outcome.execution.success {
        return Err(ParallaxError::new(
            ErrorCode::CaptureFailure,
            "execution failed during snapshot capture",
        )
        .with_runtime(rt)
        .with_diagnostic(
            outcome
                .execution
                .exception
                .map(|e| format!("{}: {}", e.type_name, e.message))
                .unwrap_or_default(),
        ));
    }
    let snap = Snapshot::create(rt, outcome.state, outcome.pir, label)?;
    snap.write_to_path(&output)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&snap.inspect_summary())?);
    } else {
        println!(
            "wrote {} ({} bindings, hash {})",
            output.display(),
            snap.pir.bindings.len(),
            snap.content_hash
        );
    }
    Ok(())
}

async fn cmd_restore(json: bool, path: PathBuf, target: String) -> Result<(), ParallaxError> {
    let snap = Snapshot::read_from_path(&path)?;
    let rt = parse_runtime(&target)?;
    let manager = build_manager();
    let policy = SandboxPolicy::default();
    let result = manager.restore_bindings(rt, &snap.pir, &policy).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!(
            "restore {}  duration: {} µs",
            if result.success { "OK" } else { "FAILED" },
            result.duration_us
        );
        for (k, v) in &result.restored_bindings {
            println!("  {k}: {v}");
        }
        for w in &result.warnings {
            println!("warning: {w}");
        }
    }
    if result.success {
        Ok(())
    } else {
        Err(ParallaxError::new(
            ErrorCode::RestoreFailure,
            "restore failed",
        ))
    }
}

#[allow(clippy::too_many_arguments)]
async fn cmd_migrate(
    json: bool,
    file: PathBuf,
    to: String,
    from: Option<String>,
    capture: String,
    allow_lossy: bool,
    no_prefer_bigint: bool,
    reject_unsupported: bool,
    strict: bool,
    output: Option<PathBuf>,
    snapshot_out: Option<PathBuf>,
    pir_input: bool,
) -> Result<(), ParallaxError> {
    let target = parse_runtime(&to)?;
    if matches!(target, RuntimeKind::Wasm) {
        return Err(ParallaxError::new(
            ErrorCode::CapabilityViolation,
            "WASM state migration is unsupported",
        )
        .with_runtime(target)
        .with_operation("migrate")
        .context("capability", "globals")
        .context("level", "NO")
        .remediate(Remediation::new(
            "Migrate between python and javascript; WASM supports execution only",
        )));
    }
    let names: Vec<String> = capture
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut policy = if strict {
        ConversionPolicy::strict()
    } else {
        ConversionPolicy::default()
    };
    policy.allow_lossy = allow_lossy || policy.allow_lossy;
    policy.prefer_bigint = !no_prefer_bigint;
    if reject_unsupported || strict {
        policy.reject_unsupported = true;
    }

    let manager = build_manager();
    let sandbox = SandboxPolicy::default();

    let mut capture_us = None;
    let (source_rt, pir_source) = if pir_input {
        let bytes = std::fs::read(&file)?;
        let doc = parallax_ir::from_json_bytes(&bytes)?;
        let rt = from
            .as_deref()
            .map(parse_runtime)
            .transpose()?
            .unwrap_or(RuntimeKind::Other("pir".into()));
        (rt, doc)
    } else {
        let source_rt = infer_runtime(&file, from.as_deref())?;
        let t0 = Instant::now();
        let mut req = ExecutionRequest::file(source_rt.clone(), file.to_string_lossy().to_string());
        req.capture_state = true;
        let outcome = manager.execute_and_capture(req, &names, &sandbox).await?;
        capture_us = Some(t0.elapsed().as_micros() as u64);
        if !outcome.execution.success {
            return Err(ParallaxError::new(
                ErrorCode::CaptureFailure,
                "source execution/capture failed",
            )
            .with_runtime(source_rt)
            .with_diagnostic(
                outcome
                    .execution
                    .exception
                    .map(|e| format!("{}: {}", e.type_name, e.message))
                    .unwrap_or_else(|| outcome.execution.stderr.clone()),
            )
            .remediate(Remediation::new(
                "Ensure the program defines the captured bindings (default: state)",
            )));
        }
        if outcome.pir.bindings.is_empty() {
            return Err(
                ParallaxError::new(ErrorCode::CaptureFailure, "no bindings captured")
                    .with_runtime(source_rt)
                    .remediate(Remediation::new(
                        "Define `state = {...}` or pass --capture name1,name2",
                    )),
            );
        }
        (source_rt, outcome.pir)
    };

    let t_total = Instant::now();
    let (migrated, mut report) =
        migrate_document(source_rt.clone(), target.clone(), &pir_source, &policy)?;
    report.timings.capture_us = capture_us;

    let t_restore = Instant::now();
    let restore = manager
        .restore_bindings(target.clone(), &migrated, &sandbox)
        .await?;
    let restore_us = t_restore.elapsed().as_micros() as u64;
    report.timings.restore_us = Some(restore_us);
    report.timings.total_us = t_total.elapsed().as_micros() as u64 + capture_us.unwrap_or(0);
    // Prefer wall-clock sum of measured phases.
    report.timings.total_us = capture_us.unwrap_or(0)
        + report.timings.analyze_us
        + report.timings.convert_us
        + restore_us;

    if !restore.success {
        report.success = false;
        report.notes.push("target restore reported failure".into());
    }
    report
        .notes
        .push(format!("restored: {:?}", restore.restored_bindings));

    if let Some(path) = snapshot_out {
        let state = ExecutionState::empty(
            target.clone(),
            match target {
                RuntimeKind::Python => parallax_core::RuntimeCapabilities::python(),
                RuntimeKind::JavaScript => parallax_core::RuntimeCapabilities::javascript(),
                RuntimeKind::Wasm => parallax_core::RuntimeCapabilities::wasm(),
                RuntimeKind::Other(_) => parallax_core::RuntimeCapabilities::none(),
            },
        );
        let snap = Snapshot::create(
            target.clone(),
            state,
            migrated.clone(),
            Some(format!("migrate-{}", Utc::now().timestamp())),
        )?;
        snap.write_to_path(&path)?;
        report
            .notes
            .push(format!("wrote snapshot {}", path.display()));
    }

    if let Some(path) = output {
        let preview = match target {
            RuntimeKind::JavaScript => emit_javascript(&migrated)?,
            RuntimeKind::Python => emit_python(&migrated)?,
            other => {
                return Err(ParallaxError::new(
                    ErrorCode::UnsupportedValue,
                    format!("no source emitter for {other}"),
                ));
            }
        };
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(&path, preview)?;
        report.notes.push(format!("wrote {}", path.display()));
    }

    print_migration(json, &report, &migrated)?;
    if report.success {
        Ok(())
    } else {
        Err(ParallaxError::new(
            ErrorCode::MigrationRejected,
            "migration completed with failures",
        ))
    }
}

fn print_migration(
    json: bool,
    report: &MigrationReport,
    migrated: &PirDocument,
) -> Result<(), ParallaxError> {
    if json {
        let out = serde_json::json!({
            "report": report,
            "bindings": migrated.bindings_to_json(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print!("{}", report.format_human());
        println!("\nmigrated bindings:");
        for (k, v) in &migrated.bindings {
            println!("  {k}: {}", v.summary());
        }
    }
    Ok(())
}

async fn cmd_runtimes(json: bool) -> Result<(), ParallaxError> {
    let manager = build_manager();
    let list = manager.list().await;
    if json {
        let rows: Vec<_> = list
            .iter()
            .map(|h| {
                serde_json::json!({
                    "runtime": h.kind,
                    "name": h.metadata.name,
                    "status": h.status,
                    "host_version": h.metadata.host_version,
                    "adapter_version": h.metadata.adapter_version,
                    "interface_version": h.metadata.interface_version,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for h in list {
            let status = match &h.status {
                RuntimeStatus::Ready => "READY".into(),
                RuntimeStatus::Degraded { reason } => format!("DEGRADED ({reason})"),
                RuntimeStatus::Unavailable { reason } => format!("UNAVAILABLE ({reason})"),
            };
            println!(
                "{:<12} {:<10} host={} adapter={}",
                h.kind,
                status,
                h.metadata.host_version.as_deref().unwrap_or("?"),
                h.metadata.adapter_version
            );
        }
    }
    Ok(())
}

async fn cmd_capabilities(json: bool, runtime: Option<String>) -> Result<(), ParallaxError> {
    let manager = build_manager();
    let list = manager.list().await;
    let filtered: Vec<_> = if let Some(r) = runtime {
        let kind = parse_runtime(&r)?;
        list.into_iter().filter(|h| h.kind == kind).collect()
    } else {
        list
    };
    if json {
        let rows: Vec<_> = filtered
            .iter()
            .map(|h| {
                serde_json::json!({
                    "runtime": h.kind,
                    "capabilities": h.capabilities,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else {
        for h in filtered {
            println!("[{}]", h.kind);
            for (name, level) in h.capabilities.entries() {
                println!("  {name:<22} {}", level.glyph());
            }
            println!();
        }
    }
    Ok(())
}

async fn cmd_doctor(json: bool) -> Result<(), ParallaxError> {
    let manager = build_manager();
    let list = manager.list().await;
    let runtimes: Vec<RuntimeHealth> =
        list.into_iter()
            .map(|h| {
                let binary = match &h.kind {
                    RuntimeKind::Python => {
                        parallax_runtime::discover_python().map(|d| d.path.display().to_string())
                    }
                    RuntimeKind::JavaScript => parallax_runtime::discover_javascript()
                        .map(|d| d.path.display().to_string()),
                    RuntimeKind::Wasm => Some("wasmtime (in-process)".into()),
                    RuntimeKind::Other(_) => None,
                };
                let detail = match &h.status {
                    RuntimeStatus::Ready => None,
                    RuntimeStatus::Degraded { reason } | RuntimeStatus::Unavailable { reason } => {
                        Some(reason.clone())
                    }
                };
                RuntimeHealth {
                    runtime: h.kind,
                    status: h.status,
                    binary,
                    version: h.metadata.host_version,
                    detail,
                }
            })
            .collect();
    let ok = runtimes
        .iter()
        .any(|r| matches!(r.status, RuntimeStatus::Ready));
    let report = DoctorReport {
        parallax_version: PARALLAX_VERSION.to_string(),
        versions: ComponentVersions::current(),
        host: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        runtimes,
        ok,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print!("{}", report.format_human());
    }
    if ok {
        Ok(())
    } else {
        Err(ParallaxError::new(
            ErrorCode::RuntimeUnavailable,
            "doctor found no ready runtimes",
        ))
    }
}

async fn cmd_bench(
    json: bool,
    iterations: u32,
    file: Option<PathBuf>,
    to: String,
) -> Result<(), ParallaxError> {
    let file = file.unwrap_or_else(|| PathBuf::from("examples/demo.py"));
    if !file.exists() {
        return Err(ParallaxError::new(
            ErrorCode::Io,
            format!("bench file not found: {}", file.display()),
        )
        .remediate(Remediation::new(
            "Pass --file path/to/demo.py or run from repo root",
        )));
    }
    let target = parse_runtime(&to)?;
    let source = infer_runtime(&file, None)?;
    let manager = build_manager();
    let sandbox = SandboxPolicy::default();
    let policy = ConversionPolicy::default();
    let names = vec!["state".into()];

    let mut samples = Vec::new();
    for i in 0..iterations {
        let t0 = Instant::now();
        let mut req = ExecutionRequest::file(source.clone(), file.to_string_lossy().to_string());
        req.capture_state = true;
        let outcome = manager.execute_and_capture(req, &names, &sandbox).await?;
        if !outcome.execution.success {
            return Err(ParallaxError::new(
                ErrorCode::ExecutionFailure,
                format!("bench iteration {i} capture failed"),
            ));
        }
        let (_doc, mut report) =
            migrate_document(source.clone(), target.clone(), &outcome.pir, &policy)?;
        let restore = manager
            .restore_bindings(target.clone(), &_doc, &sandbox)
            .await?;
        report.timings.capture_us = Some(outcome.execution.duration_us);
        report.timings.restore_us = Some(restore.duration_us);
        report.timings.total_us = t0.elapsed().as_micros() as u64;
        samples.push(report.timings);
    }

    let avg = |f: fn(&parallax_migrate::MigrationTimings) -> u64| -> u64 {
        if samples.is_empty() {
            0
        } else {
            samples.iter().map(f).sum::<u64>() / samples.len() as u64
        }
    };

    let summary = serde_json::json!({
        "iterations": iterations,
        "source": source,
        "target": target,
        "avg_capture_us": avg(|t| t.capture_us.unwrap_or(0)),
        "avg_analyze_us": avg(|t| t.analyze_us),
        "avg_convert_us": avg(|t| t.convert_us),
        "avg_restore_us": avg(|t| t.restore_us.unwrap_or(0)),
        "avg_total_us": avg(|t| t.total_us),
        "samples": samples,
    });

    if json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("bench {} → {} (n={})", source, target, iterations);
        println!("avg capture: {} µs", summary["avg_capture_us"]);
        println!("avg analyze: {} µs", summary["avg_analyze_us"]);
        println!("avg convert: {} µs", summary["avg_convert_us"]);
        println!("avg restore: {} µs", summary["avg_restore_us"]);
        println!("avg total:   {} µs", summary["avg_total_us"]);
    }
    Ok(())
}

fn cmd_version(json: bool) {
    let versions = ComponentVersions::current();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&versions).unwrap_or_else(|_| {
                serde_json::json!({
                    "parallax": PARALLAX_VERSION,
                    "pir_schema": versions.pir_schema,
                    "protocol": versions.protocol,
                    "snapshot": versions.snapshot,
                    "adapter_interface": versions.adapter_interface,
                    "ues_format": versions.ues_format,
                    "pcir_schema": versions.pcir_schema,
                })
                .to_string()
            })
        );
    } else {
        print!("{}", versions.format_human());
    }
}
