//! User-installed fonts. Known families become package installs; everything
//! else (commercial fonts, one-offs) is pulled into `~/.local/share/fonts`.

mod macos;

use anyhow::Result;
use omarchy_onboard_core::{
    Discovery, Finding, Group, Operation, Package, PackageSource, Platform, Proposal,
    SourceContext, TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub const ID: &str = "fonts";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Family {
    /// Display name, e.g. "Berkeley Mono".
    pub name: String,
    pub file_count: usize,
}

pub struct Fonts;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Fonts,
    title: "Fonts",
    description: "Fonts you installed yourself (not system fonts)",
    sources: &[Platform::MacOs],
};

#[derive(Deserialize)]
struct MapFile {
    family: BTreeMap<String, String>,
}
static MAP: LazyLock<MapFile> =
    LazyLock::new(|| toml::from_str(include_str!("map.toml")).expect("fonts/map.toml is valid"));

/// "Berkeley Mono" / "BerkeleyMono-Regular" / "berkeley mono" → "berkeleymono".
pub fn normalise(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub fn package_for(family: &str) -> Option<&'static str> {
    MAP.family.get(&normalise(family)).map(String::as_str)
}

impl Topic for Fonts {
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
        let mut out = Vec::new();
        for f in mine {
            let Ok(fam) = serde_json::from_value::<Family>(f.value.clone()) else {
                continue;
            };
            let id = format!("fonts/{}", normalise(&fam.name));
            let p = match package_for(&fam.name) {
                Some(pkg) => Proposal::action(
                    &id,
                    Group::Fonts,
                    format!("Install font {} ({pkg})", fam.name),
                    "Packaged for Arch, so it stays updated.",
                )
                .with(Operation::InstallPackages {
                    packages: vec![Package { name: pkg.into(), source: PackageSource::Pacman }],
                }),
                None => Proposal::action(
                    &id,
                    Group::Fonts,
                    format!("Copy font {} ({} files)", fam.name, fam.file_count),
                    "Not packaged for Arch; copied into ~/.local/share/fonts and fc-cache refreshed.",
                )
                .with(Operation::PullFiles {
                    items: f.files.clone(),
                    dest: ctx.home.join(".local/share/fonts").join(&fam.name),
                    mode: None,
                })
                .with(Operation::RunCommand { argv: vec!["fc-cache".into(), "-f".into()] }),
            };
            out.push(p.from(f.id()));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_and_maps() {
        assert_eq!(
            normalise("JetBrainsMono Nerd Font"),
            "jetbrainsmononerdfont"
        );
        assert_eq!(
            package_for("JetBrainsMono Nerd Font"),
            Some("ttf-jetbrains-mono-nerd")
        );
        assert_eq!(package_for("Berkeley Mono"), None);
    }
}
