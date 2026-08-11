//! Exhaustive language connector catalog.
//!
//! Every entry is a first-class Parallax connector identity. Maturity is honest:
//! Production adapters ship workers; scaffolds register with Unsupported execute/restore
//! until a real frontend/backend exists.

use serde::{Deserialize, Serialize};

/// How far a connector is implemented.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorMaturity {
    /// Full worker / engine with tests (Python, JS, WASM today).
    Production,
    /// Partial analysis or execute-only path.
    Experimental,
    /// Registered identity + capability matrix; ops return Unsupported.
    Scaffold,
    /// Documented intent only (same as scaffold in registry, softer docs tone).
    Planned,
}

impl ConnectorMaturity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Experimental => "experimental",
            Self::Scaffold => "scaffold",
            Self::Planned => "planned",
        }
    }
}

/// Ecosystem / family grouping for UX.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageFamily {
    Systems,
    ManagedVm,
    Scripting,
    Functional,
    Mobile,
    WebAssembly,
    DataScience,
    Shell,
    SmartContract,
    HardwareHdl,
    Query,
    Markup,
    Other,
}

impl LanguageFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Systems => "systems",
            Self::ManagedVm => "managed_vm",
            Self::Scripting => "scripting",
            Self::Functional => "functional",
            Self::Mobile => "mobile",
            Self::WebAssembly => "webassembly",
            Self::DataScience => "data_science",
            Self::Shell => "shell",
            Self::SmartContract => "smart_contract",
            Self::HardwareHdl => "hardware_hdl",
            Self::Query => "query",
            Self::Markup => "markup",
            Self::Other => "other",
        }
    }
}

/// Declared roles for a language connector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorRoles {
    /// Live execute via RuntimeAdapter.
    pub runtime: bool,
    /// PIR value capture/restore (state migrate).
    pub value_migrate: bool,
    /// Transmute / Mirror source frontend.
    pub transmute_source: bool,
    /// Transmute / Mirror target backend.
    pub transmute_target: bool,
}

/// Static connector definition (compile-time table; not deserialized).
#[derive(Clone, Debug)]
pub struct ConnectorDef {
    pub id: &'static str,
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub extensions: &'static [&'static str],
    pub family: LanguageFamily,
    pub maturity: ConnectorMaturity,
    pub roles: ConnectorRoles,
    /// Host binaries probed for readiness (empty = no host tool).
    pub host_binaries: &'static [&'static str],
    pub notes: &'static str,
}

const R_RUNTIME: ConnectorRoles = ConnectorRoles {
    runtime: true,
    value_migrate: false,
    transmute_source: false,
    transmute_target: false,
};

const R_FULL_MIGRATE: ConnectorRoles = ConnectorRoles {
    runtime: true,
    value_migrate: true,
    transmute_source: true,
    transmute_target: true,
};

const R_TRANS: ConnectorRoles = ConnectorRoles {
    runtime: false,
    value_migrate: false,
    transmute_source: true,
    transmute_target: true,
};

const R_SRC: ConnectorRoles = ConnectorRoles {
    runtime: false,
    value_migrate: false,
    transmute_source: true,
    transmute_target: false,
};

const R_TGT: ConnectorRoles = ConnectorRoles {
    runtime: false,
    value_migrate: false,
    transmute_source: false,
    transmute_target: true,
};

const R_BOTH_RT_SRC: ConnectorRoles = ConnectorRoles {
    runtime: true,
    value_migrate: false,
    transmute_source: true,
    transmute_target: true,
};

macro_rules! c {
    ($id:expr, $name:expr, $aliases:expr, $ext:expr, $fam:expr, $mat:expr, $roles:expr, $bins:expr, $notes:expr) => {
        ConnectorDef {
            id: $id,
            name: $name,
            aliases: $aliases,
            extensions: $ext,
            family: $fam,
            maturity: $mat,
            roles: $roles,
            host_binaries: $bins,
            notes: $notes,
        }
    };
}

/// Full Parallax language connector catalog.
pub static CONNECTORS: &[ConnectorDef] = &[
    // ── Production runtimes ───────────────────────────────────────────
    c!(
        "python",
        "Python",
        &["py", "cpython"],
        &["py", "pyi"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Production,
        R_FULL_MIGRATE,
        &["python", "python3", "py"],
        "NDJSON worker; PIR value migrate; Transmute/Mirror source (Tier 2→Rust)"
    ),
    c!(
        "javascript",
        "JavaScript (Node.js)",
        &["js", "node", "nodejs"],
        &["js", "mjs", "cjs", "jsx"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Production,
        R_FULL_MIGRATE,
        &["node", "nodejs"],
        "NDJSON worker; PIR value migrate; Transmute/Mirror source (Tier 1→Rust)"
    ),
    c!(
        "typescript",
        "TypeScript",
        &["ts"],
        &["ts", "tsx"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Production,
        ConnectorRoles {
            runtime: false,
            value_migrate: false,
            transmute_source: true,
            transmute_target: true,
        },
        &["node"],
        "Analyze via TypeScript compiler API; execute via javascript runtime; Tier 1→Rust"
    ),
    c!(
        "wasm",
        "WebAssembly",
        &["webassembly", "wat"],
        &["wasm", "wat"],
        LanguageFamily::WebAssembly,
        ConnectorMaturity::Production,
        ConnectorRoles {
            runtime: true,
            value_migrate: false,
            transmute_source: false,
            transmute_target: true,
        },
        &[],
        "In-process wasmtime execute; no binding migrate (honest NO)"
    ),
    // ── Systems ───────────────────────────────────────────────────────
    c!(
        "rust",
        "Rust",
        &["rs"],
        &["rs"],
        LanguageFamily::Systems,
        ConnectorMaturity::Experimental,
        R_TRANS,
        &["rustc", "cargo"],
        "Primary Transmute/Mirror target (Axum); reverse sync experimental"
    ),
    c!(
        "go",
        "Go",
        &["golang"],
        &["go"],
        LanguageFamily::Systems,
        ConnectorMaturity::Experimental,
        R_BOTH_RT_SRC,
        &["go"],
        "Experimental NDJSON worker (go run); execute + stdio; no binding migrate yet"
    ),
    c!(
        "c",
        "C",
        &["clang", "gcc"],
        &["c", "h"],
        LanguageFamily::Systems,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["clang", "gcc", "cc"],
        "Native connector scaffold; FFI/unsafe boundary review required"
    ),
    c!(
        "cpp",
        "C++",
        &["c++", "cxx"],
        &["cpp", "cc", "cxx", "hpp", "hh"],
        LanguageFamily::Systems,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["clang++", "g++", "c++"],
        "Native connector scaffold; ownership/lifetime mapping is hard"
    ),
    c!(
        "zig",
        "Zig",
        &[],
        &["zig"],
        LanguageFamily::Systems,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["zig"],
        "Systems language scaffold"
    ),
    c!(
        "nim",
        "Nim",
        &[],
        &["nim"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["nim"],
        "Planned Transmute pair"
    ),
    c!(
        "crystal",
        "Crystal",
        &["cr"],
        &["cr"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["crystal"],
        "Ruby-like syntax → native; planned"
    ),
    c!(
        "d",
        "D",
        &["dlang"],
        &["d"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["dmd", "ldc2", "gdc"],
        "Planned systems connector"
    ),
    c!(
        "fortran",
        "Fortran",
        &["f90", "f95"],
        &["f", "f90", "f95", "f03", "for"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_SRC,
        &["gfortran"],
        "Scientific/HPC source frontend planned"
    ),
    c!(
        "ada",
        "Ada",
        &[],
        &["adb", "ads", "ada"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["gnat"],
        "Safety-critical systems planned"
    ),
    c!(
        "cobol",
        "COBOL",
        &[],
        &["cob", "cbl", "cobol"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_SRC,
        &["cobc"],
        "Legacy modernization source planned"
    ),
    c!(
        "pascal",
        "Pascal / Object Pascal",
        &["delphi", "freepascal", "fpc"],
        &["pas", "pp", "dpr"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["fpc"],
        "Planned"
    ),
    c!(
        "assembly",
        "Assembly",
        &["asm", "nasm"],
        &["asm", "s", "S"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_SRC,
        &["nasm", "as"],
        "Low-level source; migrate only with heavy review"
    ),
    c!(
        "v",
        "V",
        &["vlang"],
        &["v"],
        LanguageFamily::Systems,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["v"],
        "Planned"
    ),
    // ── Managed / VM ──────────────────────────────────────────────────
    c!(
        "java",
        "Java",
        &[],
        &["java"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["java", "javac"],
        "JVM connector scaffold; bytecode/source dual path planned"
    ),
    c!(
        "kotlin",
        "Kotlin",
        &["kt"],
        &["kt", "kts"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Scaffold,
        R_TRANS,
        &["kotlinc", "kotlin"],
        "JVM/Native/JS multiplatform source/target scaffold"
    ),
    c!(
        "scala",
        "Scala",
        &[],
        &["scala", "sc"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Scaffold,
        R_TRANS,
        &["scala", "scalac"],
        "JVM connector scaffold"
    ),
    c!(
        "groovy",
        "Groovy",
        &[],
        &["groovy", "gvy"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Planned,
        R_SRC,
        &["groovy"],
        "Planned JVM scripting source"
    ),
    c!(
        "clojure",
        "Clojure",
        &["clj"],
        &["clj", "cljs", "cljc", "edn"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["clojure", "clj"],
        "Lisp-on-JVM scaffold"
    ),
    c!(
        "csharp",
        "C#",
        &["cs", "c#", "dotnet"],
        &["cs", "csx"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["dotnet", "csc"],
        ".NET connector scaffold"
    ),
    c!(
        "fsharp",
        "F#",
        &["fs", "f#"],
        &["fs", "fsx", "fsi"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_TRANS,
        &["dotnet"],
        ".NET functional scaffold"
    ),
    c!(
        "vbnet",
        "Visual Basic .NET",
        &["vb", "vbn"],
        &["vb"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Planned,
        R_SRC,
        &["dotnet"],
        "Legacy .NET source planned"
    ),
    c!(
        "dart",
        "Dart",
        &[],
        &["dart"],
        LanguageFamily::ManagedVm,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["dart"],
        "Flutter/server scaffold"
    ),
    // ── Scripting ─────────────────────────────────────────────────────
    c!(
        "ruby",
        "Ruby",
        &["rb"],
        &["rb", "rake", "gemspec"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Experimental,
        ConnectorRoles {
            runtime: true,
            value_migrate: true,
            transmute_source: true,
            transmute_target: true,
        },
        &["ruby"],
        "Experimental NDJSON worker; execute + capture/restore for simple values"
    ),
    c!(
        "php",
        "PHP",
        &[],
        &["php", "phtml"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Experimental,
        ConnectorRoles {
            runtime: true,
            value_migrate: true,
            transmute_source: true,
            transmute_target: true,
        },
        &["php"],
        "Experimental NDJSON worker; execute + capture/restore for simple values"
    ),
    c!(
        "perl",
        "Perl",
        &["pl"],
        &["pl", "pm", "t"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["perl"],
        "Perl connector scaffold"
    ),
    c!(
        "lua",
        "Lua",
        &[],
        &["lua"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["lua", "luajit"],
        "Embeddable scripting scaffold"
    ),
    c!(
        "tcl",
        "Tcl",
        &[],
        &["tcl", "tk"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Planned,
        R_RUNTIME,
        &["tclsh", "wish"],
        "Planned"
    ),
    c!(
        "r",
        "R",
        &["rscript"],
        &["r", "R", "rmd"],
        LanguageFamily::DataScience,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["Rscript", "R"],
        "Data science connector scaffold"
    ),
    c!(
        "julia",
        "Julia",
        &["jl"],
        &["jl"],
        LanguageFamily::DataScience,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["julia"],
        "Scientific computing scaffold"
    ),
    c!(
        "matlab",
        "MATLAB / Octave",
        &["octave", "m"],
        &["m"],
        LanguageFamily::DataScience,
        ConnectorMaturity::Planned,
        R_SRC,
        &["octave", "matlab"],
        "Numeric source planned (Octave-compatible path preferred)"
    ),
    // ── Functional / concurrent ───────────────────────────────────────
    c!(
        "haskell",
        "Haskell",
        &["hs"],
        &["hs", "lhs"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["ghc", "runghc"],
        "Pure functional scaffold"
    ),
    c!(
        "ocaml",
        "OCaml",
        &["ml"],
        &["ml", "mli"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_TRANS,
        &["ocaml", "ocamlc", "dune"],
        "ML-family scaffold"
    ),
    c!(
        "elixir",
        "Elixir",
        &["ex"],
        &["ex", "exs"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["elixir", "iex", "mix"],
        "BEAM connector scaffold"
    ),
    c!(
        "erlang",
        "Erlang",
        &["erl"],
        &["erl", "hrl"],
        LanguageFamily::Functional,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["erl", "erlc"],
        "BEAM connector scaffold"
    ),
    c!(
        "elm",
        "Elm",
        &[],
        &["elm"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_SRC,
        &["elm"],
        "Frontend FP source planned"
    ),
    c!(
        "purescript",
        "PureScript",
        &["purs"],
        &["purs"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_SRC,
        &["purs"],
        "Planned"
    ),
    c!(
        "rescript",
        "ReScript",
        &["reason", "resi"],
        &["res", "resi"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["rescript"],
        "Planned"
    ),
    // ── Mobile / Apple ────────────────────────────────────────────────
    c!(
        "swift",
        "Swift",
        &[],
        &["swift"],
        LanguageFamily::Mobile,
        ConnectorMaturity::Scaffold,
        R_BOTH_RT_SRC,
        &["swift", "swiftc"],
        "Apple/server Swift scaffold"
    ),
    c!(
        "objc",
        "Objective-C",
        &["objective-c", "objectivec"],
        &["m", "mm"],
        LanguageFamily::Mobile,
        ConnectorMaturity::Planned,
        R_SRC,
        &["clang"],
        "Legacy Apple source planned"
    ),
    // ── Shell / ops ───────────────────────────────────────────────────
    c!(
        "shell",
        "POSIX Shell / Bash",
        &["bash", "sh", "zsh"],
        &["sh", "bash", "zsh"],
        LanguageFamily::Shell,
        ConnectorMaturity::Scaffold,
        R_RUNTIME,
        &["bash", "sh"],
        "Script execute scaffold; not a Transmute target"
    ),
    c!(
        "powershell",
        "PowerShell",
        &["pwsh", "ps1"],
        &["ps1", "psm1"],
        LanguageFamily::Shell,
        ConnectorMaturity::Scaffold,
        R_RUNTIME,
        &["pwsh", "powershell"],
        "Windows/cross-platform shell scaffold"
    ),
    // ── Query / markup-adjacent ───────────────────────────────────────
    c!(
        "sql",
        "SQL",
        &["postgres", "mysql", "sqlite"],
        &["sql"],
        LanguageFamily::Query,
        ConnectorMaturity::Scaffold,
        R_SRC,
        &[],
        "Schema/query source for data-model migration; dialect-aware later"
    ),
    c!(
        "graphql",
        "GraphQL",
        &["gql"],
        &["graphql", "gql"],
        LanguageFamily::Query,
        ConnectorMaturity::Planned,
        R_SRC,
        &[],
        "API schema source planned"
    ),
    c!(
        "hcl",
        "HCL (Terraform)",
        &["terraform", "tf"],
        &["tf", "hcl"],
        LanguageFamily::Other,
        ConnectorMaturity::Planned,
        R_SRC,
        &["terraform"],
        "IaC source planned"
    ),
    // ── Smart contracts ───────────────────────────────────────────────
    c!(
        "solidity",
        "Solidity",
        &["sol"],
        &["sol"],
        LanguageFamily::SmartContract,
        ConnectorMaturity::Scaffold,
        R_SRC,
        &["solc"],
        "EVM contract source scaffold — security review mandatory"
    ),
    c!(
        "vyper",
        "Vyper",
        &[],
        &["vy"],
        LanguageFamily::SmartContract,
        ConnectorMaturity::Planned,
        R_SRC,
        &["vyper"],
        "EVM Python-like contracts planned"
    ),
    c!(
        "move",
        "Move",
        &[],
        &["move"],
        LanguageFamily::SmartContract,
        ConnectorMaturity::Planned,
        R_SRC,
        &["aptos", "sui"],
        "Move VM source planned"
    ),
    c!(
        "cairo",
        "Cairo",
        &[],
        &["cairo"],
        LanguageFamily::SmartContract,
        ConnectorMaturity::Planned,
        R_SRC,
        &["cairo-compile"],
        "Starknet source planned"
    ),
    // ── HDL ───────────────────────────────────────────────────────────
    c!(
        "verilog",
        "Verilog / SystemVerilog",
        &["sv", "systemverilog"],
        &["v", "sv", "vh"],
        LanguageFamily::HardwareHdl,
        ConnectorMaturity::Planned,
        R_SRC,
        &["iverilog", "verilator"],
        "HDL source planned"
    ),
    c!(
        "vhdl",
        "VHDL",
        &[],
        &["vhd", "vhdl"],
        LanguageFamily::HardwareHdl,
        ConnectorMaturity::Planned,
        R_SRC,
        &["ghdl"],
        "HDL source planned"
    ),
    // ── Web / transpile ecosystem ─────────────────────────────────────
    c!(
        "coffeescript",
        "CoffeeScript",
        &["coffee"],
        &["coffee"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Planned,
        R_SRC,
        &["coffee"],
        "Legacy JS transpile source planned"
    ),
    c!(
        "haxe",
        "Haxe",
        &[],
        &["hx"],
        LanguageFamily::Other,
        ConnectorMaturity::Planned,
        R_TRANS,
        &["haxe"],
        "Multi-target language planned"
    ),
    c!(
        "hack",
        "Hack",
        &[],
        &["hack", "hh"],
        LanguageFamily::Scripting,
        ConnectorMaturity::Planned,
        R_SRC,
        &["hhvm"],
        "PHP-family planned"
    ),
    c!(
        "racket",
        "Racket",
        &["rkt"],
        &["rkt"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_RUNTIME,
        &["racket"],
        "Scheme-family planned"
    ),
    c!(
        "scheme",
        "Scheme",
        &["scm"],
        &["scm", "ss"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_RUNTIME,
        &["scheme", "guile", "chicken"],
        "Planned"
    ),
    c!(
        "commonlisp",
        "Common Lisp",
        &["lisp", "cl", "sbcl"],
        &["lisp", "lsp", "cl"],
        LanguageFamily::Functional,
        ConnectorMaturity::Planned,
        R_RUNTIME,
        &["sbcl", "ecl", "ccl"],
        "Planned"
    ),
    c!(
        "prolog",
        "Prolog",
        &[],
        &["pl", "pro", "P"],
        LanguageFamily::Other,
        ConnectorMaturity::Planned,
        R_RUNTIME,
        &["swipl"],
        "Logic programming planned"
    ),
    c!(
        "wasm_component",
        "WASM Component Model",
        &["wit", "component"],
        &["wit"],
        LanguageFamily::WebAssembly,
        ConnectorMaturity::Experimental,
        R_TGT,
        &[],
        "Component-model target intent (experimental)"
    ),
];

/// Look up by id or alias.
pub fn find(name: &str) -> Option<&'static ConnectorDef> {
    let key = name.to_ascii_lowercase();
    CONNECTORS
        .iter()
        .find(|c| c.id == key || c.aliases.iter().any(|a| a.eq_ignore_ascii_case(&key)))
}

/// Infer connector id from file extension.
pub fn from_extension(ext: &str) -> Option<&'static ConnectorDef> {
    let e = ext.to_ascii_lowercase();
    // Prefer more specific / production languages when extensions collide.
    let preferred = [
        "typescript",
        "javascript",
        "python",
        "rust",
        "go",
        "java",
        "kotlin",
        "csharp",
        "ruby",
        "php",
        "swift",
    ];
    for id in preferred {
        if let Some(c) = find(id) {
            if c.extensions.iter().any(|x| x.eq_ignore_ascii_case(&e)) {
                return Some(c);
            }
        }
    }
    CONNECTORS
        .iter()
        .find(|c| c.extensions.iter().any(|x| x.eq_ignore_ascii_case(&e)))
}

/// Canonical ids that already have dedicated production adapter crates.
pub fn production_runtime_ids() -> &'static [&'static str] {
    &["python", "javascript", "wasm"]
}

/// All connectors that should register a RuntimeAdapter scaffold (exclude production crates).
pub fn scaffold_runtime_connectors() -> impl Iterator<Item = &'static ConnectorDef> {
    CONNECTORS.iter().filter(|c| {
        c.roles.runtime
            && !production_runtime_ids().contains(&c.id)
            && !matches!(c.maturity, ConnectorMaturity::Production)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_large() {
        assert!(
            CONNECTORS.len() >= 50,
            "expected broad catalog, got {}",
            CONNECTORS.len()
        );
    }

    #[test]
    fn find_aliases() {
        assert_eq!(find("py").unwrap().id, "python");
        assert_eq!(find("golang").unwrap().id, "go");
        assert_eq!(find("c#").unwrap().id, "csharp");
        assert_eq!(find("node").unwrap().id, "javascript");
    }

    #[test]
    fn unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for c in CONNECTORS {
            assert!(seen.insert(c.id), "duplicate connector id {}", c.id);
        }
    }
}
