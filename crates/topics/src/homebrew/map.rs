use omarchy_onboard_core::{NoteCategory, Package, PackageSource};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

/// One row of `map.toml`: how to get the equivalent on the target, or why not.
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
        /// Extra context shown with an install, e.g. "Microsoft build".
        #[serde(default)]
        note: Option<String>,
        /// Omarchy already ships it.
        #[serde(default)]
        covered: Option<String>,
        /// A macOS-ism with no purpose on Linux.
        #[serde(default)]
        not_needed: Option<String>,
        /// No direct equivalent; what to use instead.
        #[serde(default)]
        suggestion: Option<String>,
    },
}

impl Equivalent {
    pub fn package(&self) -> Option<Package> {
        match self {
            Equivalent::Name(n) => Some(Package {
                name: n.clone(),
                source: PackageSource::Pacman,
            }),
            Equivalent::Full {
                pacman: Some(n), ..
            } => Some(Package {
                name: n.clone(),
                source: PackageSource::Pacman,
            }),
            Equivalent::Full { aur: Some(n), .. } => Some(Package {
                name: n.clone(),
                source: PackageSource::Aur,
            }),
            Equivalent::Full {
                installer: Some(n), ..
            } => Some(Package {
                name: n.clone(),
                source: PackageSource::DistroInstaller,
            }),
            Equivalent::Full { .. } => None,
        }
    }

    pub fn note(&self) -> Option<&str> {
        match self {
            Equivalent::Full { note, .. } => note.as_deref(),
            _ => None,
        }
    }

    /// If this row is informational rather than installable.
    pub fn as_note(&self) -> Option<(NoteCategory, &str)> {
        match self {
            Equivalent::Full {
                covered: Some(t), ..
            } => Some((NoteCategory::Covered, t)),
            Equivalent::Full {
                not_needed: Some(t),
                ..
            } => Some((NoteCategory::NotNeeded, t)),
            Equivalent::Full {
                suggestion: Some(t),
                ..
            } => Some((NoteCategory::Suggestion, t)),
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

static MAP: LazyLock<MapFile> =
    LazyLock::new(|| toml::from_str(include_str!("map.toml")).expect("homebrew/map.toml is valid"));

pub fn formula(name: &str) -> Option<&'static Equivalent> {
    MAP.formula.get(name)
}

pub fn cask(name: &str) -> Option<&'static Equivalent> {
    MAP.cask.get(name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn map_parses_and_every_row_is_installable_or_a_note() {
        for (name, eq) in super::MAP.formula.iter().chain(super::MAP.cask.iter()) {
            assert!(
                eq.package().is_some() || eq.as_note().is_some(),
                "{name}: neither package nor note"
            );
        }
    }
}
