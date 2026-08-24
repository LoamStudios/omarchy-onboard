//! Discover on macOS/Linux: `$SHELL` and well-known dotfiles in `$HOME`.

use super::{ID, ShellInfo};
use crate::fs::file_ref;
use anyhow::Result;
use omarchy_onboard_core::{Finding, Group, SourceContext};

const DOTFILES: &[&str] = &[
    ".zshrc",
    ".zprofile",
    ".zshenv",
    ".bashrc",
    ".bash_profile",
    ".profile",
    ".aliases",
];

pub fn discover(ctx: &SourceContext) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();
    if let Ok(shell) = std::env::var("SHELL") {
        let name = shell.rsplit('/').next().unwrap_or(&shell).to_string();
        findings.push(
            Finding::new(
                ID,
                Group::Shell,
                "login-shell",
                format!("Login shell: {name}"),
            )
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
