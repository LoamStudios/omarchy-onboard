//! Discover on macOS: `~/Library/Application Support/Code/User` and
//! `~/.vscode/extensions/extensions.json` (no `code` CLI needed).

use super::{ConfigFile, Extension, ID};
use crate::fs::file_ref;
use anyhow::Result;
use omarchy_onboard_core::{Finding, Group, SourceContext};
use serde::Deserialize;

pub fn discover(ctx: &SourceContext) -> Result<Vec<Finding>> {
    let user = ctx.home.join("Library/Application Support/Code/User");
    let mut findings = Vec::new();
    if !user.exists() {
        return Ok(findings);
    }
    for (key, file) in [
        ("settings", "settings.json"),
        ("keybindings", "keybindings.json"),
    ] {
        if let Ok(content) = std::fs::read_to_string(user.join(file)) {
            let title = format!("VS Code {file} ({} bytes)", content.len());
            findings.push(
                Finding::new(ID, Group::Editors, key, title).with_value(ConfigFile { content }),
            );
        }
    }
    if let Some(fr) = file_ref(&user.join("snippets"))
        && fr.size > 0
    {
        findings
            .push(Finding::new(ID, Group::Editors, "snippets", "VS Code snippets").with_file(fr));
    }
    for ext in read_extensions(&ctx.home.join(".vscode/extensions/extensions.json")) {
        let title = format!(
            "VS Code extension {}{}",
            ext.id,
            ext.version
                .as_deref()
                .map(|v| format!(" {v}"))
                .unwrap_or_default()
        );
        findings.push(
            Finding::new(ID, Group::Editors, format!("extension/{}", ext.id), title)
                .with_value(ext),
        );
    }
    Ok(findings)
}

#[derive(Deserialize)]
struct Entry {
    identifier: Identifier,
    #[serde(default)]
    version: Option<String>,
}
#[derive(Deserialize)]
struct Identifier {
    id: String,
}

fn read_extensions(path: &std::path::Path) -> Vec<Extension> {
    let Ok(s) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let Ok(entries) = serde_json::from_str::<Vec<Entry>>(&s) else {
        return vec![];
    };
    let mut v: Vec<Extension> = entries
        .into_iter()
        .map(|e| Extension {
            id: e.identifier.id,
            version: e.version,
        })
        .collect();
    v.sort_by(|a, b| a.id.cmp(&b.id));
    v.dedup_by(|a, b| a.id == b.id);
    v
}
