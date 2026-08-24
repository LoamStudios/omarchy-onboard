//! Shell configuration: which shell, and its dotfiles.

use crate::fs::file_ref;
use anyhow::Result;
use omam_core::{Check, CheckMeta, Finding, Group, Platform, SourceContext};
use serde::{Deserialize, Serialize};

pub const ID: &str = "macos.shell";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInfo {
    pub shell: String,
}

pub struct ShellDotfiles;

static META: CheckMeta = CheckMeta {
    id: ID,
    group: Group::Shell,
    title: "Shell & dotfiles",
    description: "Login shell and its config files (.zshrc, .zprofile, …)",
    platforms: &[Platform::MacOs, Platform::Linux],
};

const DOTFILES: &[&str] =
    &[".zshrc", ".zprofile", ".zshenv", ".bashrc", ".bash_profile", ".profile", ".aliases"];

impl Check for ShellDotfiles {
    fn meta(&self) -> &CheckMeta {
        &META
    }

    fn run(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        if let Ok(shell) = std::env::var("SHELL") {
            let name = shell.rsplit('/').next().unwrap_or(&shell).to_string();
            findings.push(
                Finding::new(ID, Group::Shell, "login-shell", format!("Login shell: {name}"))
                    .with_value(ShellInfo { shell: name }),
            );
        }
        for name in DOTFILES {
            let path = ctx.home.join(name);
            if let Some(fr) = file_ref(&path) {
                let title = format!("~/{name} ({} bytes)", fr.size);
                findings.push(
                    Finding::new(ID, Group::Shell, format!("dotfile/{name}"), title).with_file(fr),
                );
            }
        }
        Ok(findings)
    }
}
