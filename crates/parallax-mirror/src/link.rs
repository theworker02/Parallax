//! LinkedProject and `.parallax-link/` persistence.

use crate::diff::build_index;
use crate::identity::SemanticId;
use crate::ownership::{ManualRegion, RegionOwnership};
use crate::policy::SyncPolicy;
use chrono::{DateTime, Utc};
use parallax_core::{ErrorCode, MIRROR_LINK_FORMAT_VERSION, ParallaxError, Remediation};
use parallax_project::{SourceLanguage, TargetLanguage};
use parallax_puir::PuirProgram;
use parallax_transmute::{analyze_project, looks_like_project};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Directory name for link metadata.
pub const LINK_DIR: &str = ".parallax-link";

/// Mapping between source semantic node and target artifact.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticMapping {
    pub id: SemanticId,
    pub qualified_name: String,
    pub kind: String,
    pub source_file: Option<String>,
    pub target_file: Option<String>,
    pub reverse_safe: crate::ownership::ReverseSafety,
}

/// Linked project record (`link.json`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkedProject {
    pub format_version: u32,
    pub source_root: String,
    pub target_root: String,
    pub source_language: SourceLanguage,
    pub target_language: TargetLanguage,
    pub policy: SyncPolicy,
    pub source_commit: Option<String>,
    pub target_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_source_fingerprint: String,
    pub pair_tier: String,
    /// Absolute path to `.parallax-link`.
    #[serde(skip)]
    pub link_dir: PathBuf,
    #[serde(default)]
    pub semantic_map: Vec<SemanticMapping>,
    #[serde(default)]
    pub generated_files: Vec<String>,
}

impl LinkedProject {
    /// Create link metadata by analyzing source and indexing target maps.
    pub async fn create(
        source: &Path,
        target: &Path,
        policy: SyncPolicy,
    ) -> Result<Self, ParallaxError> {
        let source = normalize_path(source)?;
        let target = normalize_path(target)?;
        if !looks_like_project(&source) {
            return Err(ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!("source is not a project: {}", source.display()),
            )
            .with_source("parallax-mirror"));
        }
        if !target.is_dir() {
            return Err(ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!("target is not a directory: {}", target.display()),
            )
            .with_source("parallax-mirror"));
        }

        let analysis = analyze_project(&source, None).await?;
        let tier = crate::pair_tier(&analysis.primary_language, &infer_target_lang(&target));
        if matches!(tier, crate::PairTier::Unsupported) {
            return Err(ParallaxError::new(
                ErrorCode::UnsupportedValue,
                format!(
                    "language pair {} → {:?} is unsupported for Mirror",
                    analysis.primary_language,
                    infer_target_lang(&target)
                ),
            )
            .with_source("parallax-mirror")
            .remediate(Remediation::new(
                "Tier-1 Mirror pair today: TypeScript/JavaScript → Rust",
            )));
        }

        let link_dir = target.join(LINK_DIR);
        fs::create_dir_all(link_dir.join("baselines")).map_err(crate::io_err)?;
        fs::create_dir_all(link_dir.join("history")).map_err(crate::io_err)?;

        let index = build_index(&analysis.puir);
        let mut semantic_map = Vec::new();
        for (qn, id) in &index {
            let kind = if qn.chars().next().unwrap_or('a').is_uppercase()
                || qn
                    .split('.')
                    .next_back()
                    .map(|s| s.chars().next().unwrap_or('a').is_uppercase())
                    == Some(true)
            {
                // heuristic: types often PascalCase last segment
                if analysis.puir.modules.values().any(|m| {
                    m.items.iter().any(|i| matches!(i, parallax_puir::PuirItem::Type(t) if qn.ends_with(&t.name)))
                }) {
                    "type"
                } else {
                    "function"
                }
            } else {
                "function"
            };
            let source_file = analysis
                .puir
                .modules
                .values()
                .find(|m| qn.starts_with(&m.id.replace('/', ".")))
                .map(|m| m.path.clone());
            let target_file = guess_target_file(&target, &source_file);
            semantic_map.push(SemanticMapping {
                id: id.clone(),
                qualified_name: qn.clone(),
                kind: kind.into(),
                source_file,
                target_file,
                reverse_safe: crate::ownership::ReverseSafety::IdiomaticPartial,
            });
        }

        let generated = list_generated(&target);
        let fp = fingerprint_puir(&analysis.puir);

        // Persist baselines
        fs::write(
            link_dir.join("baselines/source-puir.json"),
            serde_json::to_string_pretty(&analysis.puir).unwrap(),
        )
        .map_err(crate::io_err)?;
        write_bin_json(&link_dir.join("source-index.bin"), &index)?;
        write_bin_json(&link_dir.join("semantic-map.bin"), &semantic_map)?;
        fs::write(
            link_dir.join("dependency-map.json"),
            serde_json::to_string_pretty(&analysis.graph.packages).unwrap_or_default(),
        )
        .map_err(crate::io_err)?;
        fs::write(
            link_dir.join("manual-regions.json"),
            serde_json::to_string_pretty(&Vec::<ManualRegion>::new()).unwrap(),
        )
        .map_err(crate::io_err)?;

        // Ownership hashes for generated rust files
        let ownership = build_ownership(&semantic_map, &target)?;
        fs::write(
            link_dir.join("ownership.json"),
            serde_json::to_string_pretty(&ownership).unwrap(),
        )
        .map_err(crate::io_err)?;

        let link = Self {
            format_version: MIRROR_LINK_FORMAT_VERSION,
            source_root: source.display().to_string(),
            target_root: target.display().to_string(),
            source_language: analysis.primary_language.clone(),
            target_language: infer_target_lang(&target),
            policy,
            source_commit: git_head(&source),
            target_commit: git_head(&target),
            created_at: Utc::now(),
            last_sync_at: Some(Utc::now()),
            last_source_fingerprint: fp,
            pair_tier: format!("{tier:?}"),
            link_dir: link_dir.clone(),
            semantic_map,
            generated_files: generated,
        };
        link.save()?;
        // seed history
        crate::history::SyncHistory::append(
            &link_dir,
            crate::history::HistoryEntry {
                at: Utc::now(),
                source_commit: link.source_commit.clone(),
                target_commit: link.target_commit.clone(),
                semantic_changes: 0,
                files_touched: link.generated_files.clone(),
                verification: "link-created".into(),
                confidence: "HIGH".into(),
                fingerprint: link.last_source_fingerprint.clone(),
            },
        )?;
        Ok(link)
    }

    pub fn save(&self) -> Result<(), ParallaxError> {
        let mut value = serde_json::to_value(self).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
        })?;
        if let Some(obj) = value.as_object_mut() {
            obj.remove("link_dir");
        }
        fs::write(
            self.link_dir.join("link.json"),
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .map_err(crate::io_err)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, ParallaxError> {
        let link_dir = find_link_dir(path)?;
        let text = fs::read_to_string(link_dir.join("link.json")).map_err(crate::io_err)?;
        let mut link: LinkedProject = serde_json::from_str(&text).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
                .with_source("parallax-mirror")
        })?;
        if link.format_version > MIRROR_LINK_FORMAT_VERSION {
            return Err(ParallaxError::new(
                ErrorCode::InvalidArgument,
                format!(
                    "mirror link format {} newer than supported {}",
                    link.format_version, MIRROR_LINK_FORMAT_VERSION
                ),
            ));
        }
        link.link_dir = link_dir;
        if link.semantic_map.is_empty() {
            if let Ok(bytes) = fs::read(link.link_dir.join("semantic-map.bin")) {
                if let Ok(map) = serde_json::from_slice(&bytes) {
                    link.semantic_map = map;
                }
            }
        }
        Ok(link)
    }

    pub fn baseline_puir(&self) -> Result<PuirProgram, ParallaxError> {
        let text = fs::read_to_string(self.link_dir.join("baselines/source-puir.json"))
            .map_err(crate::io_err)?;
        serde_json::from_str(&text).map_err(|e| {
            ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
        })
    }

    pub fn write_baseline_puir(&self, puir: &PuirProgram) -> Result<(), ParallaxError> {
        fs::write(
            self.link_dir.join("baselines/source-puir.json"),
            serde_json::to_string_pretty(puir).unwrap(),
        )
        .map_err(crate::io_err)
    }
}

fn find_link_dir(path: &Path) -> Result<PathBuf, ParallaxError> {
    let candidates = [
        path.join(LINK_DIR),
        path.to_path_buf(),
        path.parent().unwrap_or(path).join(LINK_DIR),
    ];
    for c in candidates {
        if c.join("link.json").is_file() {
            return Ok(if c.ends_with(LINK_DIR) {
                c
            } else if c.file_name().and_then(|s| s.to_str()) == Some("link.json") {
                c.parent().unwrap().to_path_buf()
            } else {
                c
            });
        }
        if c.is_file() && c.file_name().and_then(|s| s.to_str()) == Some("link.json") {
            return Ok(c.parent().unwrap().to_path_buf());
        }
    }
    // also: path is project root containing .parallax-link
    if path.join(LINK_DIR).join("link.json").is_file() {
        return Ok(path.join(LINK_DIR));
    }
    Err(ParallaxError::new(
        ErrorCode::InvalidArgument,
        format!("no {LINK_DIR}/link.json near {}", path.display()),
    )
    .with_source("parallax-mirror")
    .remediate(Remediation::new(
        "Run: plx link <source> <target>",
    )))
}

fn infer_target_lang(target: &Path) -> TargetLanguage {
    if target.join("Cargo.toml").exists() {
        TargetLanguage::Rust
    } else if target.join("go.mod").exists() {
        TargetLanguage::Go
    } else if target.join("package.json").exists() {
        TargetLanguage::TypeScript
    } else {
        TargetLanguage::Rust
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, ParallaxError> {
    let p = path.canonicalize().map_err(crate::io_err)?;
    let s = p.to_string_lossy();
    Ok(PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s)))
}

fn guess_target_file(target: &Path, source_file: &Option<String>) -> Option<String> {
    let src = source_file.as_ref()?;
    let stem = Path::new(src)
        .file_stem()
        .and_then(|s| s.to_str())?
        .replace('-', "_");
    let name = if stem == "index" { "app" } else { &stem };
    let _ = target;
    Some(format!("src/{name}.rs"))
}

fn list_generated(target: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let src = target.join("src");
    if let Ok(rd) = fs::read_dir(src) {
        for e in rd.flatten() {
            if e.path().extension().and_then(|x| x.to_str()) == Some("rs") {
                out.push(format!(
                    "src/{}",
                    e.file_name().to_string_lossy()
                ));
            }
        }
    }
    if target.join("Cargo.toml").exists() {
        out.push("Cargo.toml".into());
    }
    out
}

fn fingerprint_puir(puir: &PuirProgram) -> String {
    let bytes = serde_json::to_vec(puir).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

fn write_bin_json(path: &Path, value: &impl Serialize) -> Result<(), ParallaxError> {
    let bytes = serde_json::to_vec(value).map_err(|e| {
        ParallaxError::new(ErrorCode::SerializationFailure, e.to_string())
    })?;
    fs::write(path, bytes).map_err(crate::io_err)
}

fn build_ownership(
    map: &[SemanticMapping],
    target: &Path,
) -> Result<Vec<RegionOwnership>, ParallaxError> {
    let mut out = Vec::new();
    for m in map {
        let hash = if let Some(f) = &m.target_file {
            let p = target.join(f);
            if p.exists() {
                let bytes = fs::read(&p).map_err(crate::io_err)?;
                let mut h = Sha256::new();
                h.update(&bytes);
                hex::encode(h.finalize())
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        out.push(RegionOwnership {
            id: m.id.clone(),
            kind: crate::ownership::RegionKind::Generated,
            target_file: m.target_file.clone().unwrap_or_default(),
            content_hash: hash,
            reverse_safe: m.reverse_safe,
        });
    }
    Ok(out)
}

fn git_head(dir: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
