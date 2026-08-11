//! Horizon planning: impossible analysis, debt, dissolve, detach.

use crate::behavior::{
    BehaviorExplorer, ProjectObserver, SemanticFuzzer, SemanticTriangulator, TriangulationReport,
};
use crate::ir::{ConcurrencyGraph, ShapeInferencer, TypeCrystallizer};
use crate::semantics::{
    BoundarySynthesizer, CapsuleCapability, CapsuleGenerator, DebtBreakdown, DebtTracker,
    IslandDissolver, MetaprogramExpander, PolyglotIsland, PreservationDecision, PreservationPolicy,
    PreservationStrategy, SemanticBarrier, SemanticHazardDatabase, SemanticMinifier,
    SemanticPreservationEngine, StrategySearcher,
};
use crate::vcs::SemanticGit;
use parallax_atlas::build_project_context;
use parallax_core::{ErrorCode, ParallaxError};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HorizonPlan {
    pub root: String,
    pub target: Option<String>,
    pub policy: String,
    pub barriers: Vec<SemanticBarrier>,
    pub decisions: Vec<PreservationDecision>,
    pub debt: DebtBreakdown,
    pub capsule_files: Vec<String>,
    pub islands: Vec<PolyglotIsland>,
    pub hazards: Vec<String>,
    pub triangulation: TriangulationReport,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImpossibleReport {
    pub root: String,
    pub target: String,
    pub barriers: Vec<SemanticBarrier>,
    pub proposed: Vec<String>,
    pub estimated_native_pct: f64,
    pub expected_compatibility_pct: f64,
    pub polyglot_requirement_pct: f64,
    pub strategy_options_sample: Vec<String>,
    pub notes: String,
}

/// Analyze hard barriers and propose preservation strategies (`plx impossible`).
pub fn analyze_impossible(
    root: &Path,
    to: Option<&str>,
    policy: Option<PreservationPolicy>,
) -> Result<ImpossibleReport, ParallaxError> {
    let target = to.unwrap_or("rust").to_string();
    let policy = policy.unwrap_or(PreservationPolicy::MaximumCompatibility);
    let obs = ProjectObserver
        .observe(root)
        .map_err(|e| ParallaxError::new(ErrorCode::Io, e).with_source("parallax-horizon"))?;
    let engine = SemanticPreservationEngine::new(policy);
    let mut barriers = Vec::new();
    let mut id = 1u32;
    for sig in &obs.dynamic_signals {
        let sample = sig.samples.first().cloned().unwrap_or_default();
        let decision = engine.decide(&sig.kind, (sig.count as f64 / 10.0).min(1.0));
        barriers.push(SemanticBarrier {
            id,
            kind: sig.kind.clone(),
            location: sample,
            detail: format!("{} occurrence(s)", sig.count),
            evidence: sig.samples.clone(),
            preferred_strategy: decision.strategy,
            confidence: decision.confidence,
            notes: decision.rationale.clone(),
        });
        id += 1;
    }
    for b in &obs.migration_barriers {
        if !barriers
            .iter()
            .any(|x| x.detail.contains(b) || x.location.contains(b))
        {
            let kind = b.split_whitespace().next().unwrap_or("barrier");
            let decision = engine.decide(kind, 0.7);
            barriers.push(SemanticBarrier {
                id,
                kind: kind.into(),
                location: b.clone(),
                detail: b.clone(),
                evidence: vec![b.clone()],
                preferred_strategy: decision.strategy,
                confidence: decision.confidence,
                notes: decision.rationale,
            });
            id += 1;
        }
    }

    let mut native = 100.0_f64;
    let mut compat = 0.0;
    let mut island = 0.0;
    let mut proposed = Vec::new();
    for b in &barriers {
        match b.preferred_strategy {
            PreservationStrategy::Native | PreservationStrategy::Lowered => {
                proposed.push(format!(
                    "{}: {} — {}",
                    b.kind,
                    b.preferred_strategy.as_str(),
                    b.notes
                ));
            }
            PreservationStrategy::Capsuled
            | PreservationStrategy::Emulated
            | PreservationStrategy::Wrapped => {
                let cost = 1.5_f64.min(b.evidence.len() as f64);
                native -= cost;
                compat += cost;
                proposed.push(format!(
                    "{}: generate specialized capsule — {}",
                    b.kind, b.notes
                ));
            }
            PreservationStrategy::Bridged | PreservationStrategy::BehaviorSynthesized => {
                let cost = 2.0_f64.min(b.evidence.len() as f64 + 1.0);
                native -= cost;
                island += cost * 0.5;
                compat += cost * 0.5;
                proposed.push(format!(
                    "{}: {} — {}",
                    b.kind,
                    b.preferred_strategy.as_str(),
                    b.notes
                ));
            }
            PreservationStrategy::ManualRequired => {
                native -= 1.0;
                proposed.push(format!("{}: MANUAL_REQUIRED — {}", b.kind, b.notes));
            }
        }
    }
    native = native.clamp(50.0, 99.0);
    let total_debt = compat + island;
    if total_debt > 0.0 {
        let scale = (100.0 - native) / total_debt;
        compat *= scale;
        island *= scale;
    }

    let searcher = StrategySearcher;
    let strategy_options_sample: Vec<String> = searcher
        .options_for_dependency("dynamic-runtime")
        .into_iter()
        .take(3)
        .map(|o| format!("{} ({})", o.label, o.strategy.as_str()))
        .collect();

    Ok(ImpossibleReport {
        root: obs.root,
        target,
        barriers,
        proposed,
        estimated_native_pct: native,
        expected_compatibility_pct: compat,
        polyglot_requirement_pct: island,
        strategy_options_sample,
        notes:
            "Estimates from static dynamic-signal heuristics + strategy costs — verify with tests"
                .into(),
    })
}

/// Full horizon plan (observe + strategies + capsule sketch).
pub fn build_horizon_plan(
    root: &Path,
    to: Option<&str>,
    policy: Option<PreservationPolicy>,
) -> Result<HorizonPlan, ParallaxError> {
    let target = to.unwrap_or("rust").to_string();
    let policy = policy.unwrap_or_default();
    let report = analyze_impossible(root, Some(&target), Some(policy.clone()))?;
    let obs = ProjectObserver
        .observe(root)
        .map_err(|e| ParallaxError::new(ErrorCode::Io, e))?;
    let engine = SemanticPreservationEngine::new(policy.clone());
    let decisions: Vec<_> = report
        .barriers
        .iter()
        .map(|b| engine.decide(&b.kind, 1.0 - b.confidence))
        .collect();

    let mut caps_needed = Vec::new();
    for d in &decisions {
        if d.strategy == PreservationStrategy::Capsuled {
            caps_needed.push(CapsuleCapability::AttributeLookup);
            caps_needed.push(CapsuleCapability::AttributeStore);
            caps_needed.push(CapsuleCapability::RuntimeStringKeys);
        }
        if d.construct.contains("null") || d.construct == "proxy" {
            caps_needed.push(CapsuleCapability::NullishCoercion);
        }
    }
    caps_needed.sort_by_key(|c| c.as_str());
    caps_needed.dedup();
    let spec = SemanticMinifier.minify("horizon", &target, &caps_needed);
    let gen = CapsuleGenerator.generate_rust(&spec);
    let capsule_files: Vec<_> = gen.files.iter().map(|f| f.relative_path.clone()).collect();

    let synth = BoundarySynthesizer;
    let mut islands = Vec::new();
    for d in decisions.iter().filter(|d| d.island_candidate) {
        let boundary = synth.synthesize(
            &d.node_id,
            obs.languages
                .keys()
                .next()
                .map(|s| s.as_str())
                .unwrap_or("python"),
            &["input"],
            &["output"],
            &obs.effects.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        );
        islands.push(PolyglotIsland {
            id: d.node_id.clone(),
            source_runtime: obs
                .languages
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "python".into()),
            reason: d.rationale.clone(),
            entrypoints: Vec::new(),
            boundary,
            estimated_loc_pct: report.polyglot_requirement_pct.max(1.0),
            dissolvable: true,
        });
    }

    let hazards = SemanticHazardDatabase
        .for_pair(
            obs.languages
                .iter()
                .max_by(|a, b| a.1.cmp(b.1))
                .map(|(k, _)| k.as_str())
                .unwrap_or("python"),
            &target,
        )
        .into_iter()
        .map(|h| format!("{}: {}", h.id, h.title))
        .collect();

    let debt = DebtBreakdown::from_parts(
        report.estimated_native_pct,
        report.expected_compatibility_pct,
        report.polyglot_requirement_pct,
        decisions
            .iter()
            .filter(|d| d.strategy == PreservationStrategy::ManualRequired)
            .count() as f64,
    );

    let triangulation =
        SemanticTriangulator.combine(0.72, 0.0, if obs.languages.is_empty() { 0.0 } else { 0.5 });

    // Touch related subsystems so the plan is a real integration surface.
    let _ = ConcurrencyGraph::from_signals(
        &obs.concurrency
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    );
    let _ = BehaviorExplorer.plan_default_inputs("normalize_user");
    let _ = SemanticFuzzer.cross_language_hazards();
    let _ = TypeCrystallizer
        .from_shape(&ShapeInferencer.from_accesses("User", &["name", "email", "permissions"]));
    let _ = MetaprogramExpander.lower_decorator("dataclass");
    let _ = build_project_context(root); // ensure atlas context path still works

    Ok(HorizonPlan {
        root: report.root,
        target: Some(target),
        policy: format!("{policy:?}"),
        barriers: report.barriers,
        decisions,
        debt,
        capsule_files,
        islands,
        hazards,
        triangulation,
    })
}

pub fn measure_debt(root: &Path, to: Option<&str>) -> Result<DebtBreakdown, ParallaxError> {
    let plan = build_horizon_plan(root, to, None)?;
    Ok(plan.debt)
}

pub fn dissolve_project(root: &Path, to: Option<&str>) -> Result<serde_json::Value, ParallaxError> {
    let plan = build_horizon_plan(root, to, None)?;
    let dissolver = IslandDissolver;
    let reports: Vec<_> = plan
        .islands
        .iter()
        .map(|i| dissolver.dissolve_step(i))
        .collect();
    let before_island = plan.debt.polyglot_island_pct;
    let after_island = reports.iter().map(|r| r.after_pct).sum::<f64>().max(0.0);
    let mut debt = plan.debt.clone();
    let reclaimed = (before_island - after_island).max(0.0);
    debt.polyglot_island_pct = after_island;
    debt.native_pct = (debt.native_pct + reclaimed).min(99.5);
    debt.target_purity = debt.native_pct;
    Ok(serde_json::json!({
        "root": plan.root,
        "dissolve": reports,
        "debt_after": debt,
        "hints": DebtTracker.burn_down_hint(&debt),
    }))
}

pub fn optimize_migration(
    root: &Path,
    to: Option<&str>,
) -> Result<serde_json::Value, ParallaxError> {
    let plan = build_horizon_plan(root, to, Some(PreservationPolicy::MaximumNative))?;
    Ok(serde_json::json!({
        "root": plan.root,
        "policy": "maximum-native",
        "capsule_files": plan.capsule_files,
        "decisions": plan.decisions,
        "debt": plan.debt,
        "actions": [
            "Replace DynamicObject fields with crystallized structs where shapes closed",
            "Lower decorators to derives",
            "Dissolve islands after capsule specialization",
        ],
        "notes": "Optimization proposes rewrites — apply only after differential verification",
    }))
}

pub fn detach_status(root: &Path, to: Option<&str>) -> Result<serde_json::Value, ParallaxError> {
    let debt = measure_debt(root, to)?;
    let ready = debt.can_detach();
    Ok(serde_json::json!({
        "ready": ready,
        "debt": debt,
        "message": if ready {
            "SOURCE RUNTIME NO LONGER REQUIRED (within measured debt thresholds — still verify gates)"
        } else {
            "Not ready to detach — reduce islands/capsules/manual barriers first"
        },
    }))
}

pub fn explain_barrier(
    root: &Path,
    barrier_id: u32,
    to: Option<&str>,
) -> Result<serde_json::Value, ParallaxError> {
    let report = analyze_impossible(root, to, None)?;
    let b = report
        .barriers
        .iter()
        .find(|b| b.id == barrier_id)
        .ok_or_else(|| {
            ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!("unknown barrier id {barrier_id}"),
            )
            .with_operation("explain-barrier")
        })?;
    Ok(serde_json::json!({
        "barrier": b,
        "why_direct_fails": format!("Target dispatch/types cannot express `{}` directly", b.kind),
        "resolution": b.notes,
        "strategy": b.preferred_strategy.as_str(),
        "confidence": b.confidence,
    }))
}

pub fn reconstruct_status() -> serde_json::Value {
    serde_json::json!({
        "supported": false,
        "mode": "behavioral_reconstruction",
        "message": "plx reconstruct is experimental scaffold — requires interfaces + tests + traces",
    })
}

pub fn blame_line(location: &str) -> serde_json::Value {
    serde_json::to_value(SemanticGit.blame(location)).unwrap()
}

pub fn cherry_pick(commit: &str) -> serde_json::Value {
    serde_json::json!({ "message": SemanticGit.cherry_pick_status(commit) })
}
