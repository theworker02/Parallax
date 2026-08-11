//! `plx` binary — alias of `parallax`.

use clap::Parser;
use parallax_cli::{entry, Cli};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    entry(Cli::parse()).await
}
