//! VS Code: settings, keybindings, snippets, and extensions.
//!
//! Extensions are proposed as `InstallEditorExtension`, never copied.
//! Keybindings are rewritten `cmd+` → `ctrl+`, which is what VS Code itself
//! does for its default keymap across platforms.

mod macos;

use anyhow::Result;
use omarchy_onboard_core::{
    ConfigMode, Decision, Discovery, Finding, Group, Operation, Platform, Proposal, SourceContext,
    TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "vscode";
/// Editor id used in `InstallEditorExtension`; the executor maps it to the `code` binary.
pub const EDITOR: &str = "vscode";
/// Where VS Code (Microsoft build) keeps user config on Linux.
const LINUX_USER_DIR: &str = ".config/Code/User";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub content: String,
}

pub struct VsCode;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Editors,
    title: "VS Code",
    description: "User settings, keybindings, snippets, and installed extensions",
    sources: &[Platform::MacOs],
};

impl Topic for VsCode {
    fn meta(&self) -> &TopicMeta {
        &META
    }

    fn discover(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        match ctx.platform {
            Platform::MacOs => macos::discover(ctx),
            _ => Ok(vec![]),
        }
    }

    fn propose(&self, mine: &[&Finding], _all: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let user_dir = ctx.home.join(LINUX_USER_DIR);
        let mut out = Vec::new();
        let mut extensions = Vec::new();
        for f in mine {
            if let Some(id) = f.key.strip_prefix("extension/") {
                extensions.push((f.id(), id.to_string()));
            } else if f.key == "settings" {
                let Ok(c) = serde_json::from_value::<ConfigFile>(f.value.clone()) else {
                    continue;
                };
                out.push(Proposal {
                    id: "vscode/settings".into(),
                    group: Group::Editors,
                    title: "Copy VS Code settings.json".into(),
                    rationale: "User settings are portable; written to ~/.config/Code/User.".into(),
                    findings: vec![f.id()],
                    operations: vec![Operation::WriteConfig {
                        path: user_dir.join("settings.json"),
                        content: c.content,
                        mode: ConfigMode::Replace,
                    }],
                    default: Decision::Accept,
                });
            } else if f.key == "keybindings" {
                let Ok(c) = serde_json::from_value::<ConfigFile>(f.value.clone()) else {
                    continue;
                };
                let (content, n) = rewrite_cmd(&c.content);
                out.push(Proposal {
                    id: "vscode/keybindings".into(),
                    group: Group::Editors,
                    title: "Copy VS Code keybindings.json".into(),
                    rationale: if n > 0 {
                        format!("Rewrote {n} `cmd+` chords to `ctrl+`, matching VS Code's own Linux keymap.")
                    } else {
                        "No macOS-specific chords found; copied as-is.".into()
                    },
                    findings: vec![f.id()],
                    operations: vec![Operation::WriteConfig {
                        path: user_dir.join("keybindings.json"),
                        content,
                        mode: ConfigMode::Replace,
                    }],
                    default: Decision::Accept,
                });
            } else if f.key == "snippets" {
                out.push(Proposal {
                    id: "vscode/snippets".into(),
                    group: Group::Editors,
                    title: "Copy VS Code snippets".into(),
                    rationale: "Your snippets directory, as-is.".into(),
                    findings: vec![f.id()],
                    operations: vec![Operation::PullFiles {
                        items: f.files.clone(),
                        dest: user_dir.join("snippets"),
                        mode: None,
                    }],
                    default: Decision::Accept,
                });
            }
        }
        if !extensions.is_empty() {
            let (finding_ids, ids): (Vec<_>, Vec<_>) = extensions.into_iter().unzip();
            out.push(Proposal {
                id: "vscode/extensions".into(),
                group: Group::Editors,
                title: format!("Install {} VS Code extensions", ids.len()),
                rationale: format!(
                    "Installed through `code --install-extension`, not copied: {}.",
                    ids.join(", ")
                ),
                findings: finding_ids,
                operations: ids
                    .into_iter()
                    .map(|extension| Operation::InstallEditorExtension {
                        editor: EDITOR.into(),
                        extension,
                    })
                    .collect(),
                default: Decision::Accept,
            });
        }
        out
    }
}

/// `"cmd+k cmd+s"` → `"ctrl+k ctrl+s"` inside `"key": "…"` values only. Returns the count rewritten.
fn rewrite_cmd(content: &str) -> (String, usize) {
    let mut out = String::with_capacity(content.len());
    let mut rest = content;
    let mut n = 0;
    while let Some(i) = rest.find("\"key\"") {
        let after = i + 5;
        // Find the opening quote of the value after the colon.
        let Some(q1) = rest[after..].find('"').map(|j| after + j + 1) else {
            break;
        };
        let Some(q2) = rest[q1..].find('"').map(|j| q1 + j) else {
            break;
        };
        out.push_str(&rest[..q1]);
        let value = &rest[q1..q2];
        if value.contains("cmd+") {
            n += 1;
            out.push_str(&value.replace("cmd+", "ctrl+"));
        } else {
            out.push_str(value);
        }
        rest = &rest[q2..];
    }
    out.push_str(rest);
    (out, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_only_key_lines() {
        let src = "[\n  {\n    \"key\": \"cmd+k cmd+s\",\n    \"command\": \"cmd+foo\"\n  }\n]\n";
        let (out, n) = rewrite_cmd(src);
        assert_eq!(n, 1);
        assert!(out.contains("\"key\": \"ctrl+k ctrl+s\""));
        assert!(out.contains("\"command\": \"cmd+foo\""));
    }
}
