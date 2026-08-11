//! `plx connectors` — language connector catalog.

use parallax_connectors::{catalog_snapshot, find, ConnectorInfo};
use parallax_core::ParallaxError;

pub fn cmd_connectors(
    json: bool,
    id: Option<String>,
    pairs: bool,
    family: Option<String>,
    maturity: Option<String>,
) -> Result<(), ParallaxError> {
    let snap = catalog_snapshot();

    if let Some(id) = id {
        let def = find(&id).ok_or_else(|| {
            parallax_core::ParallaxError::new(
                parallax_core::ErrorCode::InvalidArgument,
                format!("unknown connector: {id}"),
            )
            .with_operation("connectors")
            .remediate(parallax_core::Remediation::new(
                "Run `plx connectors` for the full catalog",
            ))
        })?;
        let info = snap
            .connectors
            .iter()
            .find(|c| c.id == def.id)
            .cloned()
            .unwrap();
        if json {
            println!("{}", serde_json::to_string_pretty(&info).unwrap());
        } else {
            print_one(&info);
            if pairs {
                println!("\nHighlighted pairs involving {}:", def.id);
                for p in &snap.highlighted_pairs {
                    if p.source == def.id || p.target == def.id {
                        println!("  {} → {}  [{}]", p.source, p.target, p.maturity.as_str());
                    }
                }
            }
        }
        return Ok(());
    }

    let fam = family.map(|s| s.to_ascii_lowercase());
    let mat = maturity.map(|s| s.to_ascii_lowercase());
    let filtered: Vec<&ConnectorInfo> = snap
        .connectors
        .iter()
        .filter(|c| fam.as_ref().map(|f| c.family == *f).unwrap_or(true))
        .filter(|c| mat.as_ref().map(|m| c.maturity == *m).unwrap_or(true))
        .collect();

    if json {
        if pairs {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "count": filtered.len(),
                    "connectors": filtered,
                    "highlighted_pairs": snap.highlighted_pairs,
                }))
                .unwrap()
            );
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "count": filtered.len(),
                    "connectors": filtered,
                }))
                .unwrap()
            );
        }
        return Ok(());
    }

    println!("PARALLAX LANGUAGE CONNECTORS");
    println!(
        "Total catalogued: {}  (showing {})",
        snap.count,
        filtered.len()
    );
    println!("ID             MATURITY   FAMILY         RT     SRC    TGT    NAME");
    println!("{}", "-".repeat(78));
    for c in &filtered {
        let line = format!(
            "{:<14} {:<10} {:<14} {:<6} {:<6} {:<6} {}",
            c.id,
            c.maturity,
            c.family,
            yn(c.runtime),
            yn(c.transmute_source),
            yn(c.transmute_target),
            c.name
        );
        println!("{line}");
    }
    println!("RT=runtime adapter  SRC=transmute source  TGT=transmute target");
    println!("Production execute/migrate today: python, javascript, wasm (+ typescript analyze).");
    println!(
        "Scaffold = registered identity; execute/restore return Unsupported until implemented."
    );
    if pairs {
        println!("HIGHLIGHTED PAIRS");
        for p in &snap.highlighted_pairs {
            let line = format!("  {} → {}  [{}]", p.source, p.target, p.maturity.as_str());
            println!("{line}");
        }
    } else {
        println!(
            "Tip: plx connectors --pairs  |  plx connectors go  |  plx connectors --maturity production"
        );
    }
    Ok(())
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "-"
    }
}

fn print_one(c: &ConnectorInfo) {
    println!("{}", c.name);
    println!("  id:           {}", c.id);
    println!("  maturity:     {}", c.maturity);
    println!("  family:       {}", c.family);
    println!("  aliases:      {}", c.aliases.join(", "));
    println!("  extensions:   {}", c.extensions.join(", "));
    println!("  runtime:      {}", c.runtime);
    println!("  value_migrate:{}", c.value_migrate);
    println!("  transmute_src:{}", c.transmute_source);
    println!("  transmute_tgt:{}", c.transmute_target);
    println!("  host tools:   {}", c.host_binaries.join(", "));
    println!("  notes:        {}", c.notes);
}
