//! Shell dotfiles → pull to the target, with a warning where zsh ≠ bash.

use omarchy_onboard_checks::macos::shell;
use omarchy_onboard_core::{Decision, Discovery, Group, Operation, Proposal, Rule, TargetContext};

pub struct ShellDotfiles;

impl Rule for ShellDotfiles {
    fn id(&self) -> &'static str {
        "shell-dotfiles"
    }

    fn consumes(&self) -> &'static [&'static str] {
        &[shell::ID]
    }

    fn propose(&self, discovery: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let mut out = Vec::new();
        for f in discovery.for_check(shell::ID) {
            let Some(name) = f.key.strip_prefix("dotfile/") else { continue };
            let Some(file) = f.files.first() else { continue };
            let zsh = name.starts_with(".zsh");
            out.push(Proposal {
                id: format!("shell/{name}"),
                group: Group::Shell,
                title: format!("Copy ~/{name}"),
                rationale: if zsh {
                    "Omarchy's default shell is bash; this will only take effect if you install and switch to zsh (`chsh -s /usr/bin/zsh`).".into()
                } else {
                    "Shell config is user-owned; copying it is the equivalent.".into()
                },
                findings: vec![f.id()],
                operation: Operation::PullFiles { items: vec![file.clone()], dest: ctx.home.join(name) },
                default: if zsh { Decision::Skip } else { Decision::Accept },
            });
        }
        out
    }
}
