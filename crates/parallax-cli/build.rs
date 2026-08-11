//! Enlarge the default Windows stack so clap's derive Command graph fits in debug builds.
fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os == "windows" && env == "msvc" {
        // Default Windows stack is 1 MiB; clap derive for a large Subcommand enum
        // can overflow during Command construction in debug builds.
        println!("cargo:rustc-link-arg=/stack:{}", 8 * 1024 * 1024);
    }
}
