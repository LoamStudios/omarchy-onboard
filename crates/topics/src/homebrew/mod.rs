//! Homebrew formulae and casks → Arch packages (or a note when there is no equivalent).

mod macos;
mod map;

use anyhow::Result;
use omarchy_onboard_core::{
    Discovery, Finding, Group, NoteCategory, Operation, Package, PackageSource, Platform, Proposal,
    SourceContext, TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "homebrew";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewPackage {
    pub name: String,
    pub version: String,
    /// `true` if the user asked for it (vs. pulled in as a dependency).
    pub requested: bool,
    pub cask: bool,
}

pub struct Homebrew;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Packages,
    title: "Homebrew packages",
    description: "Command-line tools and apps installed with `brew`",
    sources: &[Platform::MacOs],
};

impl Topic for Homebrew {
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
            let Ok(pkg) = serde_json::from_value::<BrewPackage>(f.value.clone()) else {
                continue;
            };
            // Dependencies come along with whatever needs them on the target.
            if !pkg.requested {
                continue;
            }
            out.push(propose_one(f, &pkg, ctx));
        }
        out
    }
}

fn propose_one(f: &Finding, pkg: &BrewPackage, ctx: &TargetContext) -> Proposal {
    let group = if pkg.cask {
        Group::Applications
    } else {
        Group::Packages
    };
    let id = format!(
        "{}/{}",
        if pkg.cask { "apps" } else { "packages" },
        pkg.name
    );
    let mapped = if pkg.cask {
        map::cask(&pkg.name)
    } else {
        map::formula(&pkg.name)
    };

    let install = |target: Package, rationale: String| {
        Proposal::action(
            &id,
            group,
            format!("Install {}", describe(&target)),
            rationale,
        )
        .from(f.id())
        .with(Operation::InstallPackages {
            packages: vec![target],
        })
    };

    match mapped {
        Some(eq) => {
            if let Some((category, text)) = eq.as_note() {
                return Proposal::note(&id, category, group, &pkg.name, text).from(f.id());
            }
            let target = eq.package().expect("map row is installable or a note");
            let rationale = match eq.note() {
                Some(n) => format!("{n}."),
                None => format!("`{}` is the same tool, packaged for Arch.", target.name),
            };
            install(target, rationale)
        }
        None => match ctx.packages.lookup(&pkg.name) {
            Some(source) if !pkg.cask => install(
                Package {
                    name: pkg.name.clone(),
                    source,
                },
                format!(
                    "Not in the mapping table, but `{}` exists on the target with the same name.",
                    pkg.name
                ),
            ),
            _ => Proposal::note(
                &id,
                NoteCategory::Unknown,
                group,
                &pkg.name,
                format!(
                    "No mapping yet. Search for an equivalent: `yay -Ss {}`",
                    pkg.name
                ),
            )
            .from(f.id()),
        },
    }
}

fn describe(p: &Package) -> String {
    match p.source {
        PackageSource::Pacman => p.name.clone(),
        PackageSource::Aur => format!("{} (AUR)", p.name),
        PackageSource::DistroInstaller => format!("{} (omarchy installer)", p.name),
    }
}
