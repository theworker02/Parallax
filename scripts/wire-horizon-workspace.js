const fs = require("fs");
const path = require("path");
const root = process.cwd();
const cargoPath = path.join(root, "Cargo.toml");
let cargo = fs.readFileSync(cargoPath, "utf8");
const horizonMembers = ["parallax-horizon"];
const memberLines = horizonMembers.map((m) => `    "crates/${m}",`).join("\n");
if (!cargo.includes("parallax-horizon")) {
  cargo = cargo.replace(
    `    "crates/parallax-atlas",\n    "crates/parallax-cli",\n]`,
    `    "crates/parallax-atlas",\n${memberLines}\n    "crates/parallax-cli",\n]`
  );
}
const depLines = horizonMembers
  .map((m) => `${m} = { path = "crates/${m}" }`)
  .join("\n");
if (!cargo.includes("parallax-horizon =")) {
  cargo = cargo.replace(
    `parallax-atlas = { path = "crates/parallax-atlas" }\n`,
    `parallax-atlas = { path = "crates/parallax-atlas" }\n${depLines}\n`
  );
}
fs.writeFileSync(cargoPath, cargo);
console.log("workspace updated");
