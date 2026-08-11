//! Host binary discovery for Python / Node.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A discovered interpreter/runtime binary.
#[derive(Clone, Debug)]
pub struct DiscoveredBinary {
    /// Absolute or PATH-resolved command.
    pub path: PathBuf,
    /// Version string from probing.
    pub version: Option<String>,
}

/// Discover a usable Python interpreter.
///
/// Tries `python`, `python3`, `py`, then common Windows install locations.
/// Rejects Windows Store stub aliases that fail to run.
pub fn discover_python() -> Option<DiscoveredBinary> {
    let mut candidates: Vec<PathBuf> = vec![
        PathBuf::from("python"),
        PathBuf::from("python3"),
        PathBuf::from("py"),
    ];

    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let base = PathBuf::from(local).join("Programs").join("Python");
        if let Ok(entries) = std::fs::read_dir(&base) {
            for ent in entries.flatten() {
                let p = ent.path().join("python.exe");
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }
    // Also check Program Files style paths.
    for pf in [
        std::env::var_os("ProgramFiles"),
        std::env::var_os("ProgramFiles(x86)"),
    ]
    .into_iter()
    .flatten()
    {
        let base = PathBuf::from(pf).join("Python");
        if let Ok(entries) = std::fs::read_dir(&base) {
            for ent in entries.flatten() {
                let p = ent.path().join("python.exe");
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    for cand in candidates {
        if let Some(bin) = probe_python(&cand) {
            return Some(bin);
        }
    }
    None
}

fn probe_python(cmd: &Path) -> Option<DiscoveredBinary> {
    let output = Command::new(cmd)
        .args(["-c", "import sys; print(sys.version.split()[0])"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return None;
    }
    // Reject Microsoft Store stub noise.
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("Microsoft Store") {
        return None;
    }
    Some(DiscoveredBinary {
        path: cmd.to_path_buf(),
        version: Some(version),
    })
}

/// Discover Node.js.
pub fn discover_javascript() -> Option<DiscoveredBinary> {
    let candidates = [PathBuf::from("node"), PathBuf::from("nodejs")];
    for cand in candidates {
        if let Some(bin) = probe_node(&cand) {
            return Some(bin);
        }
    }
    // Common Windows install path.
    let pf = std::env::var_os("ProgramFiles").map(PathBuf::from);
    if let Some(pf) = pf {
        let p = pf.join("nodejs").join("node.exe");
        if let Some(bin) = probe_node(&p) {
            return Some(bin);
        }
    }
    None
}

fn probe_node(cmd: &Path) -> Option<DiscoveredBinary> {
    let output = Command::new(cmd).arg("-v").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .trim_start_matches('v')
        .to_string();
    if version.is_empty() {
        return None;
    }
    Some(DiscoveredBinary {
        path: cmd.to_path_buf(),
        version: Some(version),
    })
}
