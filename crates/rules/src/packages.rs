//! Homebrew formulae/casks → target packages.

use crate::maps;
use omarchy_onboard_checks::macos::homebrew::{self, BrewPackage};
use omarchy_onboard_core::{Decision, Discovery, Group, Operation, Package, PackageSource, Proposal, Rule, TargetContext};

pub struct HomebrewToPacman;

impl Rule for HomebrewToPacman {
    fn id(&self) -> &'static str {
        "homebrew-to-pacman"
    }

    fn consumes(&self) -> &'static [&'static str] {
        &[homebrew::ID]
    }

    fn propose(&self, discovery: &Discovery, _ctx: &TargetContext) -> Vec<Proposal> {
        let mut out = Vec::new();
        for f in discovery.for_check(homebrew::ID) {
            let Ok(pkg) = serde_json::from_value::<BrewPackage>(f.value.clone()) else { continue };
            // Dependencies come along with whatever needs them on the target.
            if !pkg.requested {
                continue;
            }
            let group = if pkg.cask { Group::Applications } else { Group::Packages };
            let mapped = if pkg.cask { maps::cask(&pkg.name) } else { maps::formula(&pkg.name) };
            let id = format!("{}/{}", if pkg.cask { "apps" } else { "packages" }, pkg.name);

            let proposal = match mapped {
                Some(eq) if eq.manual().is_some() => Proposal {
                    id,
                    group,
                    title: format!("{}: no direct equivalent", pkg.name),
                    rationale: eq.manual().unwrap().to_string(),
                    findings: vec![f.id()],
                    operation: Operation::Manual { instructions: eq.manual().unwrap().to_string() },
                    default: Decision::Skip,
                },
                Some(eq) => {
                    let target = eq.package().expect("non-manual map entry has a package");
                    let rationale = match eq.note() {
                        Some(n) => format!("{n}."),
                        None => format!("`{}` is the same tool, packaged for Arch.", target.name),
                    };
                    Proposal {
                        id,
                        group,
                        title: format!("Install {}", describe(&target)),
                        rationale,
                        findings: vec![f.id()],
                        operation: Operation::InstallPackages { packages: vec![target] },
                        default: Decision::Accept,
                    }
                }
                None if pkg.cask => Proposal {
                    id,
                    group,
                    title: format!("{}: unknown equivalent", pkg.name),
                    rationale: "No mapping yet for this app. Search the AUR or find an alternative.".into(),
                    findings: vec![f.id()],
                    operation: Operation::Manual {
                        instructions: format!("Search for an equivalent: `yay -Ss {}`", pkg.name),
                    },
                    default: Decision::Skip,
                },
                None => {
                    let target = Package { name: pkg.name.clone(), source: PackageSource::Pacman };
                    Proposal {
                        id,
                        group,
                        title: format!("Install {}", describe(&target)),
                        rationale: "Not in the mapping table; assuming the Arch package has the same name.".into(),
                        findings: vec![f.id()],
                        operation: Operation::InstallPackages { packages: vec![target] },
                        default: Decision::Accept,
                    }
                }
            };
            out.push(proposal);
        }
        out
    }
}

fn describe(p: &Package) -> String {
    match p.source {
        PackageSource::Pacman => p.name.clone(),
        PackageSource::Aur => format!("{} (AUR)", p.name),
        PackageSource::DistroInstaller => format!("{} (omarchy installer)", p.name),
    }
}
