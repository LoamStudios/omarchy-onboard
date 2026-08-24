//! SSH keys → pull with 0600; config → rewritten without macOS-only options.

use omarchy_onboard_checks::macos::ssh::{self, SshConfig};
use omarchy_onboard_core::{ConfigMode, Decision, Discovery, Group, Operation, Proposal, Rule, TargetContext};

pub struct Ssh;

/// ssh_config options only OpenSSH-on-macOS understands; Linux ssh errors on them.
const MACOS_ONLY: &[&str] = &["usekeychain", "addkeystoagent"];

impl Rule for Ssh {
    fn id(&self) -> &'static str {
        "ssh"
    }

    fn consumes(&self) -> &'static [&'static str] {
        &[ssh::ID]
    }

    fn propose(&self, discovery: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let mut out = Vec::new();
        let ssh_dir = ctx.home.join(".ssh");
        for f in discovery.for_check(ssh::ID) {
            if let Some(name) = f.key.strip_prefix("key/") {
                out.push(Proposal {
                    id: format!("ssh/key/{name}"),
                    group: Group::Keys,
                    title: format!("Copy SSH key {name}"),
                    rationale: "Keys are yours; copied with 0600 so ssh will accept them.".into(),
                    findings: vec![f.id()],
                    // Executor treats a single-item dest as the file path, multi-item as a directory.
                    operation: Operation::PullFiles {
                        items: f.files.clone(),
                        dest: if f.files.len() == 1 { ssh_dir.join(name) } else { ssh_dir.clone() },
                        mode: Some(0o600),
                    },
                    default: Decision::Accept,
                });
            } else if f.key == "known_hosts" {
                out.push(Proposal {
                    id: "ssh/known_hosts".into(),
                    group: Group::Keys,
                    title: "Copy ~/.ssh/known_hosts".into(),
                    rationale: "Keeps host fingerprints you've already trusted.".into(),
                    findings: vec![f.id()],
                    operation: Operation::PullFiles {
                        items: f.files.clone(),
                        dest: ssh_dir.join("known_hosts"),
                        mode: Some(0o600),
                    },
                    default: Decision::Accept,
                });
            } else if f.key == "config" {
                let Ok(cfg) = serde_json::from_value::<SshConfig>(f.value.clone()) else { continue };
                let (content, dropped) = strip_macos_options(&cfg.content);
                let rationale = if dropped.is_empty() {
                    "Copied as-is.".to_string()
                } else {
                    format!("Dropped macOS-only options ({}); Linux ssh rejects them.", dropped.join(", "))
                };
                out.push(Proposal {
                    id: "ssh/config".into(),
                    group: Group::Keys,
                    title: format!("Write ~/.ssh/config ({} hosts)", cfg.hosts.len()),
                    rationale,
                    findings: vec![f.id()],
                    operation: Operation::WriteConfig { path: ssh_dir.join("config"), content, mode: ConfigMode::Replace },
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
            let key = line.trim().split(|c: char| c.is_whitespace() || c == '=').next().unwrap_or("").to_ascii_lowercase();
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
        let (s, d) = strip_macos_options("Host *\n  UseKeychain yes\n  AddKeysToAgent yes\n  IdentityFile ~/.ssh/id_ed25519\n");
        assert_eq!(s, "Host *\n  IdentityFile ~/.ssh/id_ed25519\n");
        assert_eq!(d, vec!["usekeychain", "addkeystoagent"]);
    }
}
