use crate::ui;
use anyhow::{Context, Result};
use omamigrate_core::{Discovery, Platform, SourceContext};
use std::collections::BTreeMap;
use std::path::Path;

pub fn list_checks(all: bool) -> Result<()> {
    let platform = Platform::current();
    let checks = if all { omamigrate_checks::all() } else { omamigrate_checks::for_platform(platform) };
    ui::heading(&format!("Checks{}", if all { "" } else { " for this machine" }));
    let mut by_group: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for c in &checks {
        by_group.entry(c.meta().group).or_default().push(c.meta());
    }
    for (group, metas) in by_group {
        ui::group(group.title(), metas.len());
        for m in metas {
            ui::item(&format!("{}  {}", console::style(m.id).green(), m.title));
            ui::note(m.description);
        }
    }
    Ok(())
}

/// Run every applicable check on this machine.
pub fn discover(only: &[String]) -> Result<Discovery> {
    let ctx = SourceContext::current()?;
    let mut discovery = Discovery {
        source_platform: ctx.platform,
        source_host: hostname(),
        checks_run: vec![],
        checks_failed: BTreeMap::new(),
        findings: vec![],
    };
    for check in omamigrate_checks::for_platform(ctx.platform) {
        let id = check.meta().id;
        if !only.is_empty() && !only.iter().any(|o| o == id) {
            continue;
        }
        tracing::info!(check = id, "running");
        discovery.checks_run.push(id.to_string());
        match check.run(&ctx) {
            Ok(f) => discovery.findings.extend(f),
            Err(e) => {
                discovery.checks_failed.insert(id.to_string(), format!("{e:#}"));
            }
        }
    }
    Ok(discovery)
}

pub fn scan(out: &Path, only: &[String]) -> Result<()> {
    let discovery = discover(only)?;
    print_discovery(&discovery);
    std::fs::write(out, serde_json::to_string_pretty(&discovery)?)
        .with_context(|| format!("writing {}", out.display()))?;
    println!("\nWrote {} findings to {}", discovery.findings.len(), out.display());
    Ok(())
}

pub fn print_discovery(d: &Discovery) {
    ui::heading(&format!("Discovered on {} ({:?})", d.source_host, d.source_platform));
    for (group, findings) in d.by_group() {
        ui::group(group.title(), findings.len());
        for f in findings {
            ui::item(&f.title);
        }
    }
    for (check, err) in &d.checks_failed {
        println!("{} {check}: {err}", console::style("failed").red());
    }
}

fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}
