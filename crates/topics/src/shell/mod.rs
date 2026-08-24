//! Login shell and its dotfiles.

mod unix;

use anyhow::Result;
use omarchy_onboard_core::{
    Decision, Discovery, Finding, Group, Kind, Operation, Platform, Proposal, SourceContext,
    TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "shell";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInfo {
    pub shell: String,
}

pub struct Shell;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Shell,
    title: "Shell & dotfiles",
    description: "Login shell and its config files (.zshrc, .zprofile, …)",
    sources: &[Platform::MacOs, Platform::Linux],
};

impl Topic for Shell {
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
        let mut out = Vec::new();
        for f in mine {
            let Some(name) = f.key.strip_prefix("dotfile/") else {
                continue;
            };
            let Some(file) = f.files.first() else {
                continue;
            };
            let zsh = name.starts_with(".zsh");
            out.push(Proposal {
                id: format!("shell/{name}"),
                kind: Kind::Action,
                group: Group::Shell,
                title: format!("Copy ~/{name}"),
                rationale: if zsh {
                    "Omarchy's default shell is bash; this only takes effect if you install and switch to zsh (`chsh -s /usr/bin/zsh`).".into()
                } else {
                    "Shell config is user-owned; copying it is the equivalent.".into()
                },
                findings: vec![f.id()],
                operations: vec![Operation::PullFiles { items: vec![file.clone()], dest: ctx.home.join(name), mode: None }],
                default: if zsh { Decision::Skip } else { Decision::Accept },
            });
        }
        out
    }
}
