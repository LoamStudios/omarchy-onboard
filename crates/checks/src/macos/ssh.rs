//! SSH: key pairs, client config, known hosts.
//!
//! Private key *contents* never enter a finding — only `FileRef`s. The config
//! text is included so the rule can rewrite macOS-only options.

use crate::fs::file_ref;
use anyhow::Result;
use omarchy_onboard_core::{Check, CheckMeta, Finding, Group, Platform, SourceContext};
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

pub struct Ssh;

static META: CheckMeta = CheckMeta {
    id: ID,
    group: Group::Keys,
    title: "SSH keys & config",
    description: "Key pairs, ~/.ssh/config hosts, and known_hosts",
    platforms: &[Platform::MacOs, Platform::Linux],
};

impl Check for Ssh {
    fn meta(&self) -> &CheckMeta {
        &META
    }

    fn run(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        let dir = ctx.home.join(".ssh");
        let mut findings = Vec::new();
        let Ok(entries) = std::fs::read_dir(&dir) else { return Ok(findings) };

        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();

        for name in &names {
            let path = dir.join(name);
            match name.as_str() {
                "config" => {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    let hosts = content
                        .lines()
                        .filter_map(|l| l.trim().strip_prefix("Host "))
                        .map(|h| h.trim().to_string())
                        .filter(|h| h != "*")
                        .collect::<Vec<_>>();
                    let title = format!("~/.ssh/config ({} hosts)", hosts.len());
                    findings.push(Finding::new(ID, Group::Keys, "config", title).with_value(SshConfig { content, hosts }));
                }
                "known_hosts" => {
                    if let Some(fr) = file_ref(&path) {
                        findings.push(Finding::new(ID, Group::Keys, "known_hosts", "~/.ssh/known_hosts").with_file(fr));
                    }
                }
                _ if name.ends_with(".pub") || name == "authorized_keys" || name.starts_with("known_hosts") => {}
                _ if is_private_key(&path) => {
                    let pub_path = dir.join(format!("{name}.pub"));
                    let (key_type, comment) = std::fs::read_to_string(&pub_path)
                        .ok()
                        .map(|s| {
                            let mut it = s.split_whitespace();
                            let t = it.next().map(str::to_string);
                            let c = it.nth(1).map(str::to_string);
                            (t, c)
                        })
                        .unwrap_or((None, None));
                    let has_public = pub_path.exists();
                    let mut f = Finding::new(
                        ID,
                        Group::Keys,
                        format!("key/{name}"),
                        format!("SSH key {name} ({}{})", key_type.as_deref().unwrap_or("unknown type"), comment.as_deref().map(|c| format!(", {c}")).unwrap_or_default()),
                    )
                    .with_value(KeyPair { name: name.clone(), has_public, key_type, comment });
                    if let Some(fr) = file_ref(&path) {
                        f = f.with_file(fr);
                    }
                    if let Some(fr) = file_ref(&pub_path) {
                        f = f.with_file(fr);
                    }
                    findings.push(f);
                }
                _ => {}
            }
        }
        Ok(findings)
    }
}

fn is_private_key(path: &std::path::Path) -> bool {
    let mut head = [0u8; 40];
    let Ok(mut f) = std::fs::File::open(path) else { return false };
    use std::io::Read;
    let Ok(n) = f.read(&mut head) else { return false };
    let s = String::from_utf8_lossy(&head[..n]);
    s.starts_with("-----BEGIN ") && s.contains("PRIVATE KEY")
}
