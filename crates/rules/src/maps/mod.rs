use omarchy_onboard_core::{Package, PackageSource};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// One row of a package map: how to get the equivalent on the target.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Equivalent {
    /// Shorthand: same name, official repos.
    Name(String),
    Full {
        #[serde(default)]
        pacman: Option<String>,
        #[serde(default)]
        aur: Option<String>,
        #[serde(default)]
        installer: Option<String>,
        /// Human note, e.g. "Omarchy ships this by default".
        #[serde(default)]
        note: Option<String>,
        /// No Linux equivalent; instructions for the user.
        #[serde(default)]
        manual: Option<String>,
    },
}

impl Equivalent {
    pub fn package(&self) -> Option<Package> {
        match self {
            Equivalent::Name(n) => Some(Package { name: n.clone(), source: PackageSource::Pacman }),
            Equivalent::Full { pacman: Some(n), .. } => {
                Some(Package { name: n.clone(), source: PackageSource::Pacman })
            }
            Equivalent::Full { aur: Some(n), .. } => {
                Some(Package { name: n.clone(), source: PackageSource::Aur })
            }
            Equivalent::Full { installer: Some(n), .. } => {
                Some(Package { name: n.clone(), source: PackageSource::DistroInstaller })
            }
            Equivalent::Full { .. } => None,
        }
    }

    pub fn note(&self) -> Option<&str> {
        match self {
            Equivalent::Full { note, .. } => note.as_deref(),
            _ => None,
        }
    }

    pub fn manual(&self) -> Option<&str> {
        match self {
            Equivalent::Full { manual, .. } => manual.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MapFile {
    #[serde(default)]
    formula: BTreeMap<String, Equivalent>,
    #[serde(default)]
    cask: BTreeMap<String, Equivalent>,
}

static HOMEBREW: LazyLock<MapFile> =
    LazyLock::new(|| toml::from_str(include_str!("homebrew.toml")).expect("maps/homebrew.toml is valid"));

pub fn formula(name: &str) -> Option<&'static Equivalent> {
    HOMEBREW.formula.get(name)
}

pub fn cask(name: &str) -> Option<&'static Equivalent> {
    HOMEBREW.cask.get(name)
}
