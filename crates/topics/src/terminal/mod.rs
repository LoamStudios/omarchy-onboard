//! Terminal emulator configs. Portable ones (Ghostty, Alacritty, Kitty,
//! WezTerm) are rewritten to their Linux path minus macOS-only settings;
//! iTerm2 is macOS-only and becomes a suggestion. Cross-checks the fonts
//! topic so a referenced font that won't exist on the target is called out.

mod macos;

use anyhow::Result;
use omarchy_onboard_core::{
    ConfigMode, Discovery, Finding, Group, NoteCategory, Operation, Platform, Proposal,
    SourceContext, TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "terminal";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emulator {
    Ghostty,
    Alacritty,
    Kitty,
    WezTerm,
    ITerm2,
}

impl Emulator {
    pub fn name(self) -> &'static str {
        match self {
            Emulator::Ghostty => "Ghostty",
            Emulator::Alacritty => "Alacritty",
            Emulator::Kitty => "Kitty",
            Emulator::WezTerm => "WezTerm",
            Emulator::ITerm2 => "iTerm2",
        }
    }
    /// Config path relative to `$HOME` on Linux.
    fn linux_config(self) -> Option<&'static str> {
        match self {
            Emulator::Ghostty => Some(".config/ghostty/config"),
            Emulator::Alacritty => Some(".config/alacritty/alacritty.toml"),
            Emulator::Kitty => Some(".config/kitty/kitty.conf"),
            Emulator::WezTerm => Some(".config/wezterm/wezterm.lua"),
            Emulator::ITerm2 => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub emulator: Emulator,
    pub content: String,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
}

pub struct Terminal;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Terminal,
    title: "Terminal emulator",
    description: "Ghostty / Alacritty / Kitty / WezTerm config, font and theme",
    sources: &[Platform::MacOs],
};

impl Topic for Terminal {
    fn meta(&self) -> &TopicMeta {
        &META
    }

    fn discover(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        match ctx.platform {
            Platform::MacOs => macos::discover(ctx),
            _ => Ok(vec![]),
        }
    }

    fn propose(&self, mine: &[&Finding], all: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let mut out = Vec::new();
        for f in mine {
            let Ok(cfg) = serde_json::from_value::<TerminalConfig>(f.value.clone()) else {
                continue;
            };
            let id = format!("terminal/{:?}", cfg.emulator).to_lowercase();
            let Some(rel) = cfg.emulator.linux_config() else {
                out.push(
                    Proposal::note(
                        &id,
                        NoteCategory::Suggestion,
                        Group::Terminal,
                        cfg.emulator.name(),
                        "macOS only. Omarchy ships Alacritty and Ghostty; pick one with `omarchy-menu` → Setup.",
                    )
                    .from(f.id()),
                );
                continue;
            };
            let (content, dropped) = strip_macos(cfg.emulator, &cfg.content);
            let mut rationale = if dropped.is_empty() {
                "Copied as-is.".to_string()
            } else {
                format!("Dropped macOS-only settings: {}.", dropped.join(", "))
            };
            if let Some(font) = &cfg.font_family {
                match font_status(font, all) {
                    FontStatus::Found => rationale.push_str(&format!(" Font \"{font}\" comes over via the Fonts topic.")),
                    FontStatus::Packaged(pkg) => rationale.push_str(&format!(" Font \"{font}\" is available as `{pkg}`.")),
                    FontStatus::Missing => rationale.push_str(&format!(
                        " Font \"{font}\" was not found among your fonts — the terminal will fall back until you install it."
                    )),
                }
            }
            let mut p = Proposal::action(
                &id,
                Group::Terminal,
                format!("Copy {} config", cfg.emulator.name()),
                rationale,
            )
            .from(f.id())
            .with(Operation::WriteConfig {
                path: ctx.home.join(rel),
                content,
                mode: ConfigMode::Replace,
            });
            if let Some(FontStatus::Packaged(pkg)) =
                cfg.font_family.as_deref().map(|f| font_status(f, all))
            {
                p = p.with(Operation::InstallPackages {
                    packages: vec![omarchy_onboard_core::Package {
                        name: pkg.into(),
                        source: omarchy_onboard_core::PackageSource::Pacman,
                    }],
                });
            }
            out.push(p);
        }
        out
    }
}

enum FontStatus {
    /// The fonts topic found it on the source; it will be copied.
    Found,
    /// Not on the source as a user font, but packaged for Arch.
    Packaged(&'static str),
    Missing,
}

fn font_status(font: &str, all: &Discovery) -> FontStatus {
    let want = crate::fonts::normalise(font);
    let found = all
        .for_topic(crate::fonts::ID)
        .any(|f| f.key == format!("family/{want}"));
    if found {
        FontStatus::Found
    } else if let Some(pkg) = crate::fonts::package_for(font) {
        FontStatus::Packaged(pkg)
    } else {
        FontStatus::Missing
    }
}

/// Remove settings that only mean something on macOS. Returns the kept
/// content and the names of what was dropped.
fn strip_macos(emulator: Emulator, content: &str) -> (String, Vec<String>) {
    let mut dropped = Vec::new();
    let is_mac_line = |line: &str| -> Option<String> {
        let t = line.trim_start();
        match emulator {
            Emulator::Ghostty => t
                .starts_with("macos-")
                .then(|| t.split(['=', ' ']).next().unwrap_or(t).to_string()),
            Emulator::Kitty => t
                .starts_with("macos_")
                .then(|| t.split_whitespace().next().unwrap_or(t).to_string()),
            Emulator::Alacritty => t
                .starts_with("option_as_alt")
                .then(|| "option_as_alt".to_string()),
            Emulator::WezTerm | Emulator::ITerm2 => None,
        }
    };
    let kept: Vec<&str> = content
        .lines()
        .filter(|l| match is_mac_line(l) {
            Some(name) => {
                if !dropped.contains(&name) {
                    dropped.push(name);
                }
                false
            }
            None => true,
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
    fn strips_ghostty_macos_keys() {
        let (s, d) = strip_macos(
            Emulator::Ghostty,
            "theme = bamboo\nmacos-titlebar-style = \"tabs\"\nfont-size = 18\n",
        );
        assert_eq!(s, "theme = bamboo\nfont-size = 18\n");
        assert_eq!(d, vec!["macos-titlebar-style"]);
    }
}
