use crate::ui;
use anyhow::{Context, Result};
use omarchy_onboard_core::{Discovery, Platform, SourceContext};
use std::collections::BTreeMap;
use std::path::Path;

pub fn list_topics(all: bool) -> Result<()> {
    let platform = Platform::current();
    let topics = if all {
        omarchy_onboard_topics::all()
    } else {
        omarchy_onboard_topics::for_source(platform)
    };
    ui::heading(&format!(
        "Topics{}",
        if all {
            ""
        } else {
            " this machine can be read for"
        }
    ));
    let mut by_group: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for t in &topics {
        by_group.entry(t.meta().group).or_default().push(t.meta());
    }
    for (group, metas) in by_group {
        ui::group(group.title(), &metas.len().to_string());
        for m in metas {
            let sources: Vec<String> = m
                .sources
                .iter()
                .map(|p| format!("{p:?}").to_lowercase())
                .collect();
            ui::item(&format!(
                "{}  {}  {}",
                console::style(m.id).green(),
                m.title,
                console::style(format!("[{}]", sources.join(", "))).dim()
            ));
            ui::note(m.description);
        }
    }
    Ok(())
}

/// Run every applicable topic on this machine.
pub fn discover(only: &[String]) -> Result<Discovery> {
    let ctx = SourceContext::current()?;
    omarchy_onboard_topics::discover(&ctx, &hostname(), only)
}

pub fn scan(out: &Path, only: &[String]) -> Result<()> {
    let discovery = discover(only)?;
    print_discovery(&discovery);
    std::fs::write(out, serde_json::to_string_pretty(&discovery)?)
        .with_context(|| format!("writing {}", out.display()))?;
    println!(
        "\nWrote {} findings to {}",
        discovery.findings.len(),
        out.display()
    );
    Ok(())
}

pub fn print_discovery(d: &Discovery) {
    ui::heading(&format!(
        "Discovered on {} ({:?})",
        d.source_host, d.source_platform
    ));
    for (group, findings) in d.by_group() {
        ui::group(group.title(), &findings.len().to_string());
        for f in findings {
            ui::item(&f.title);
        }
    }
    for (topic, err) in &d.topics_failed {
        println!("{} {topic}: {err}", console::style("failed").red());
    }
}

pub fn hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".into())
}
