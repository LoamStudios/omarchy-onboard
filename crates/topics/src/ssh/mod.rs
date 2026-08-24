//! SSH keys, client config, known hosts.
//!
//! Private key *contents* never enter a finding — only `FileRef`s. Keys are
//! pulled with 0600; the config is rewritten without macOS-only options.

mod unix;

use anyhow::Result;
use omarchy_onboard_core::{
    ConfigMode, Decision, Discovery, Finding, Group, Operation, Platform, Proposal, SourceContext,
    TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "ssh";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub name: String,
    pub has_public: bool,
    /// From the `.pub` file's key-type field, e.g. `ssh-ed25519`.
    pub key_type: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub content: String,
    pub hosts: Vec<String>,
}

/// ssh_config options only OpenSSH-on-macOS understands; Linux ssh errors on them.
const MACOS_ONLY: &[&str] = &["usekeychain", "addkeystoagent"];

pub struct Ssh;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Keys,
    title: "SSH keys & config",
    description: "Key pairs, ~/.ssh/config hosts, and known_hosts",
    sources: &[Platform::MacOs, Platform::Linux],
};

impl Topic for Ssh {
    fn meta(&self) -> &TopicMeta {
        &META
    }

    fn discover(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        match ctx.platform {
            Platform::MacOs | Platform::Linux => unix::discover(ctx),
            Platform::Windows => Ok(vec![]),
        }
    }

    fn propose(&self, mine: &[&Finding], _all: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let ssh_dir = ctx.home.join(".ssh");
        let mut out = Vec::new();
        for f in mine {
            if let Some(name) = f.key.strip_prefix("key/") {
                out.push(Proposal {
                    id: format!("ssh/key/{name}"),
                    group: Group::Keys,
                    title: format!("Copy SSH key {name}"),
                    rationale: "Keys are yours; copied with 0600 so ssh will accept them.".into(),
                    findings: vec![f.id()],
                    // Executor treats a single-item dest as the file path, multi-item as a directory.
                    operations: vec![Operation::PullFiles {
                        items: f.files.clone(),
                        dest: if f.files.len() == 1 {
                            ssh_dir.join(name)
                        } else {
                            ssh_dir.clone()
                        },
                        mode: Some(0o600),
                    }],
                    default: Decision::Accept,
                });
            } else if f.key == "known_hosts" {
                out.push(Proposal {
                    id: "ssh/known_hosts".into(),
                    group: Group::Keys,
                    title: "Copy ~/.ssh/known_hosts".into(),
                    rationale: "Keeps host fingerprints you've already trusted.".into(),
                    findings: vec![f.id()],
                    operations: vec![Operation::PullFiles {
                        items: f.files.clone(),
                        dest: ssh_dir.join("known_hosts"),
                        mode: Some(0o600),
                    }],
                    default: Decision::Accept,
                });
            } else if f.key == "config" {
                let Ok(cfg) = serde_json::from_value::<SshConfig>(f.value.clone()) else {
                    continue;
                };
                let (content, dropped) = strip_macos_options(&cfg.content);
                let rationale = if dropped.is_empty() {
                    "Copied as-is.".to_string()
                } else {
                    format!(
                        "Dropped macOS-only options ({}); Linux ssh rejects them.",
                        dropped.join(", ")
                    )
                };
                out.push(Proposal {
                    id: "ssh/config".into(),
                    group: Group::Keys,
                    title: format!("Write ~/.ssh/config ({} hosts)", cfg.hosts.len()),
                    rationale,
                    findings: vec![f.id()],
                    operations: vec![Operation::WriteConfig {
                        path: ssh_dir.join("config"),
                        content,
                        mode: ConfigMode::Replace,
                    }],
                    default: Decision::Accept,
                });
            }
        }
        out
    }
}

fn strip_macos_options(content: &str) -> (String, Vec<String>) {
    let mut dropped = Vec::new();
    let kept: Vec<&str> = content
        .lines()
        .filter(|line| {
            let key = line
                .trim()
                .split(|c: char| c.is_whitespace() || c == '=')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if MACOS_ONLY.contains(&key.as_str()) {
                if !dropped.contains(&key) {
                    dropped.push(key);
                }
                false
            } else {
                true
            }
        })
        .collect();
    let mut s = kept.join("\n");
    if content.ends_with('\n') {
        s.push('\n');
    }
    (s, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_usekeychain() {
        let (s, d) = strip_macos_options(
            "Host *\n  UseKeychain yes\n  AddKeysToAgent yes\n  IdentityFile ~/.ssh/id_ed25519\n",
        );
        assert_eq!(s, "Host *\n  IdentityFile ~/.ssh/id_ed25519\n");
        assert_eq!(d, vec!["usekeychain", "addkeystoagent"]);
    }
}
