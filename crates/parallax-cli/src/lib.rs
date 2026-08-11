//! Parallax CLI library (shared by `parallax` and `plx` binaries).

#![deny(unsafe_code)]

mod app;
mod atlas_cmd;
mod connectors_cmd;
mod continuum;
mod emit;
mod horizon_cmd;
mod mirror_cmd;
mod transmute_cmd;

use clap::{Parser, Subcommand, ValueEnum};
use parallax_core::{ComponentVersions, ParallaxError, PARALLAX_VERSION};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::OnceLock;

fn long_version() -> &'static str {
    static LONG: OnceLock<String> = OnceLock::new();
    LONG.get_or_init(|| {
        let v = ComponentVersions::current();
        format!(
            "{}\npir_schema {}\nprotocol {}\nsnapshot {}\nadapter_interface {}\nues_format {}\npcir_schema {}\npuir_schema {}\nmirror_link_format {}",
            v.parallax,
            v.pir_schema,
            v.protocol,
            v.snapshot,
            v.adapter_interface,
            v.ues_format,
            v.pcir_schema,
            v.puir_schema,
            v.mirror_link_format
        )
    })
    .as_str()
}

/// Parallax CLI.
#[derive(Debug, Parser)]
#[command(
    name = "parallax",
    about = "Parallax — universal polyglot execution & state migration",
    version = PARALLAX_VERSION,
    long_version = long_version()
)]
pub struct Cli {
    /// Emit machine-readable JSON for commands that support it.
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose diagnostics.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable structured tracing to stderr.
    #[arg(long, global = true)]
    pub trace: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Execute a guest program.
    Run {
        /// Program file.
        file: PathBuf,
        /// Runtime override (python|javascript|wasm).
        #[arg(long, short = 'r')]
        runtime: Option<String>,
        /// Timeout in milliseconds.
        #[arg(long, default_value_t = 30000)]
        timeout_ms: u64,
        /// WASM entry export name.
        #[arg(long)]
        entry: Option<String>,
        /// Capture binding names after execution (comma-separated).
        #[arg(long)]
        capture: Option<String>,
    },
    /// Inspect a `.plx` snapshot.
    Inspect {
        /// Snapshot path.
        snapshot: PathBuf,
    },
    /// Capture program state into a `.plx` snapshot.
    Snapshot {
        /// Program file.
        file: PathBuf,
        /// Output `.plx` path.
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Runtime override.
        #[arg(long, short = 'r')]
        runtime: Option<String>,
        /// Binding names to capture (default: state).
        #[arg(long, default_value = "state")]
        capture: String,
        /// Optional label.
        #[arg(long)]
        label: Option<String>,
    },
    /// Restore a snapshot into a runtime (verifies restore).
    Restore {
        /// Snapshot path.
        snapshot: PathBuf,
        /// Target runtime.
        #[arg(long, short = 't')]
        target: String,
    },
    /// Migrate state (PIR) or an entire project (Transmute).
    Migrate {
        /// Source program file, PIR JSON, or project directory.
        file: PathBuf,
        /// Target runtime (python|javascript) or language (rust|go|…).
        #[arg(long, short = 't')]
        to: String,
        /// Source runtime / language override.
        #[arg(long, short = 'f')]
        from: Option<String>,
        /// Binding names to migrate for value mode (default: state).
        #[arg(long, default_value = "state")]
        capture: String,
        /// Allow known-lossy conversions (e.g. unsafe int → Number).
        #[arg(long)]
        allow_lossy: bool,
        /// Disable automatic BigInt for integers outside JS safe range.
        #[arg(long)]
        no_prefer_bigint: bool,
        /// Reject unsupported values instead of encoding them as PIR Unsupported.
        #[arg(long)]
        reject_unsupported: bool,
        /// Strict policy: reject unsupported and potentially-lossy conversions.
        #[arg(long)]
        strict: bool,
        /// Write restored target-language source / project output.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Also write a `.plx` snapshot of migrated PIR (value mode).
        #[arg(long)]
        snapshot: Option<PathBuf>,
        /// Offline-only: treat file as JSON PIR document (skip live capture).
        #[arg(long)]
        pir_input: bool,
        /// Migration mode: `value` (PIR), `continuation` (UES), or `project` (Transmute).
        #[arg(long, default_value = "auto")]
        mode: String,
        /// Dry-run project migration (plan only).
        #[arg(long)]
        dry_run: bool,
        /// Verify migrated project (build/tests/behavior).
        #[arg(long)]
        verify: bool,
        /// Target code style: idiomatic|literal.
        #[arg(long, default_value = "idiomatic")]
        target_style: String,
        /// Preserve source layout when possible.
        #[arg(long)]
        preserve_layout: bool,
        /// Require successful target build.
        #[arg(long)]
        require_build: bool,
        /// Require target tests to pass.
        #[arg(long)]
        require_tests: bool,
        /// Minimum overall confidence \[0.0–1.0\].
        #[arg(long)]
        min_confidence: Option<f64>,
        /// Fail if unsupported regions remain.
        #[arg(long)]
        fail_on_unsupported: bool,
        /// Keep language/role untranslated (repeatable), e.g. sql, shell.
        #[arg(long)]
        keep: Vec<String>,
        /// Incremental update (reuse `.parallax/` workspace).
        #[arg(long)]
        update: bool,
        /// Force project (Transmute) migration even for a single file.
        #[arg(long)]
        project: bool,
    },
    /// Look up the original source location for a generated line (`file:line`).
    Origin {
        /// Generated `path:line` (relative to migrated project).
        location: String,
        /// Migrated project root (directory containing `.plxmap.json`).
        #[arg(long, short = 'C', default_value = ".")]
        project: PathBuf,
    },
    /// Continuum: explicit safepoint capture / UES inspect / checkpoint resume.
    Continuum {
        /// Program with `parallax.checkpoint(...)`, or a UES JSON file with `--inspect-ues`.
        file: PathBuf,
        /// Runtime override.
        #[arg(long, short = 'r')]
        runtime: Option<String>,
        /// Write captured UES JSON.
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Resume post-checkpoint region on the same runtime (experimental).
        #[arg(long)]
        resume: bool,
        /// Target runtime for contract analysis (default: same as source).
        #[arg(long, short = 't')]
        to: Option<String>,
        /// Only analyze the migration contract (no execute).
        #[arg(long)]
        analyze_only: bool,
        /// Inspect a previously written UES file instead of running a program.
        #[arg(long)]
        inspect_ues: bool,
        /// Timeout in milliseconds.
        #[arg(long, default_value_t = 30000)]
        timeout_ms: u64,
    },
    /// List registered runtimes and status.
    Runtimes,
    /// List the full language connector catalog (all languages Parallax knows).
    Connectors {
        /// Optional connector id / alias filter.
        id: Option<String>,
        /// Show highlighted transmute/mirror pairs.
        #[arg(long)]
        pairs: bool,
        /// Filter by family (systems|scripting|managed_vm|…).
        #[arg(long)]
        family: Option<String>,
        /// Filter by maturity (production|experimental|scaffold|planned).
        #[arg(long)]
        maturity: Option<String>,
    },
    /// Atlas: list / inspect modular adapters.
    Adapters {
        #[command(subcommand)]
        command: Option<AdaptersCommand>,
    },
    /// Atlas: scaffold or validate a third-party adapter (stubs).
    Adapter {
        #[command(subcommand)]
        command: AdapterCommand,
    },
    /// Atlas: detect project stack and adapter plan.
    Analyze {
        /// Project root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Target language for stack suggestion / estimate.
        #[arg(long, short = 't')]
        to: Option<String>,
        /// Write `parallax.lock` into the project root.
        #[arg(long)]
        write_lock: bool,
    },
    /// Atlas: list known target stack presets.
    Stacks,
    /// Atlas: show dependency equivalence mappings.
    Mappings {
        /// Optional package / ecosystem filter.
        query: Option<String>,
    },
    /// Atlas: language-pair compatibility scores.
    Compatibility {
        /// Source language.
        source: String,
        /// Target language.
        target: String,
    },
    /// Atlas: report scaffold / unsupported adapter surface for a project.
    Unsupported {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Atlas: explain why a target stack was selected.
    ExplainStack {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: inspect a project semantically (no migration).
    Observe {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Event Horizon: report hard barriers + preservation strategy.
    Impossible {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
        /// maximum-compatibility|maximum-native|maximum-performance|minimum-dependencies|fastest-migration
        #[arg(long)]
        strategy: Option<String>,
    },
    /// Event Horizon: shrink polyglot islands one step.
    Dissolve {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: compatibility debt / target purity.
    Debt {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: check if source runtime can be detached.
    Detach {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: source-less behavioral reconstruction (scaffold).
    Reconstruct,
    /// Event Horizon: propose replacing capsules/islands with native semantics.
    OptimizeMigration {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: explain a barrier from `plx impossible`.
    ExplainBarrier {
        /// Barrier id from `plx impossible`.
        id: u32,
        #[arg(long, short = 'C', default_value = ".")]
        path: PathBuf,
        #[arg(long, short = 't')]
        to: Option<String>,
    },
    /// Event Horizon: semantic blame for a generated location (scaffold).
    Blame {
        /// `file:line` in the target project.
        location: String,
    },
    /// Event Horizon: semantic cherry-pick (scaffold).
    CherryPick {
        commit: String,
    },
    /// Event Horizon: show / emit a semantic patch (.plxp) example.
    Patch {
        /// Print the built-in example patch.
        #[arg(long)]
        example: bool,
    },
    /// Show capability matrix.
    Capabilities {
        /// Optional runtime filter.
        runtime: Option<String>,
        /// Show Continuum continuation capability matrix.
        #[arg(long)]
        continuations: bool,
    },
    /// Probe host for runtime readiness.
    Doctor,
    /// Micro-benchmark capture/migrate/restore for the demo shape.
    Bench {
        /// Iterations.
        #[arg(long, default_value_t = 5)]
        iterations: u32,
        /// Source file (default: examples/demo.py if present).
        #[arg(long)]
        file: Option<PathBuf>,
        /// Target runtime.
        #[arg(long, default_value = "javascript")]
        to: String,
    },
    /// Link source and target projects for Mirror sync.
    Link {
        /// Source project directory.
        source: PathBuf,
        /// Target project directory.
        target: PathBuf,
        /// Sync policy: source-authoritative|target-authoritative|bidirectional|manual.
        #[arg(long, default_value = "source-authoritative")]
        policy: String,
    },
    /// Incrementally synchronize a linked pair.
    Sync {
        /// Path to linked project (target or `.parallax-link`).
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Check freshness without modifying files.
        #[arg(long)]
        check: bool,
        /// Attempt reverse sync (target → source) when safe.
        #[arg(long)]
        reverse: bool,
        /// Run formatter/linter checks.
        #[arg(long)]
        lint: bool,
        /// Write a patch preview only.
        #[arg(long)]
        patch: bool,
        /// Skip build/test verification.
        #[arg(long)]
        no_verify: bool,
        /// Request property-based equivalence notes.
        #[arg(long)]
        property: bool,
        /// Deterministic verification notes.
        #[arg(long)]
        deterministic: bool,
    },
    /// Show Mirror link status / drift.
    Status {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// CI entrypoint (sync --check + verify).
    Ci {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show sync history.
    History {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Rollback last sync snapshot.
    Rollback {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Explain a generated location (`file:line`).
    Explain {
        location: String,
        #[arg(long, short = 'C', default_value = ".")]
        path: PathBuf,
    },
    /// Explain why a target file changed.
    Why {
        file: String,
        #[arg(long, short = 'C', default_value = ".")]
        path: PathBuf,
    },
    /// Verify linked target (differential tests).
    Verify {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        property: bool,
        #[arg(long)]
        deterministic: bool,
        #[arg(long)]
        performance: bool,
    },
    /// Show version / build info.
    Version {
        #[arg(long, value_enum, default_value_t = VersionFmt::Text)]
        format: VersionFmt,
    },
}

/// `plx adapters` subcommands.
#[derive(Debug, Subcommand)]
pub enum AdaptersCommand {
    /// List adapters (default).
    List {
        /// Optional filter (id, language, type).
        query: Option<String>,
    },
    /// Show adapter manifest + capabilities.
    Info {
        /// Adapter id, name, or language.
        id: String,
    },
    /// Show capability flags only.
    Capabilities {
        id: String,
    },
    /// Health scores.
    Health,
    /// Check / apply adapter package updates (built-ins only today).
    Update {
        #[arg(long)]
        check: bool,
    },
    /// Local adapter telemetry summary.
    Report,
}

/// `plx adapter` tooling.
#[derive(Debug, Subcommand)]
pub enum AdapterCommand {
    /// Generate a new adapter scaffold (not yet implemented).
    New {
        name: String,
    },
    /// Validate an adapter package (not yet implemented).
    Validate {
        path: PathBuf,
    },
}

/// Version output format.
#[derive(Debug, Clone, ValueEnum)]
pub enum VersionFmt {
    /// Plain text.
    Text,
    /// JSON object.
    Json,
}

/// Entry point shared by binaries.
pub async fn entry(cli: Cli) -> ExitCode {
    parallax_diagnostics::init_tracing(cli.trace, cli.verbose);
    let json = cli.json;
    match app::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if json {
                if let Ok(v) = serde_json::to_string_pretty(&err) {
                    eprintln!("{v}");
                } else {
                    eprint!("{}", err.format_report(true));
                }
            } else {
                eprint!("{}", err.format_report(true));
            }
            ExitCode::from(1)
        }
    }
}

/// Result alias.
pub type Result<T> = std::result::Result<T, ParallaxError>;
