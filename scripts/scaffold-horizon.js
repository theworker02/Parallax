const fs = require("fs");
const path = require("path");
const root = process.cwd();
const crates = [
  [
    "parallax-horizon",
    "Event Horizon — semantic reconstruction for impossible migrations (includes pvabi, semantics, behavior, ir, vcs modules)",
  ],
];
for (const [name, desc] of crates) {
  const dir = path.join(root, "crates", name, "src");
  fs.mkdirSync(dir, { recursive: true });
  const toml = `[package]
name = "${name}"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "${desc}"
rust-version.workspace = true

[dependencies]
parallax-core = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
indexmap = { workspace = true }
chrono = { workspace = true }
`;
  fs.writeFileSync(path.join(root, "crates", name, "Cargo.toml"), toml);
  fs.writeFileSync(path.join(root, "crates", name, "src", "lib.rs"), "#![deny(unsafe_code)]\n");
  console.log("created", name);
}
