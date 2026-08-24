//! Installed Homebrew formulae and casks.

use anyhow::{Context, Result};
use omarchy_onboard_core::{Check, CheckMeta, Finding, Group, Platform, SourceContext};
use serde::{Deserialize, Serialize};
use std::process::Command;

pub const ID: &str = "macos.homebrew";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrewPackage {
    pub name: String,
    pub version: String,
    /// `true` if the user asked for it (vs. pulled in as a dependency).
    pub requested: bool,
    pub cask: bool,
}

pub struct Homebrew;

static META: CheckMeta = CheckMeta {
    id: ID,
    group: Group::Packages,
    title: "Homebrew packages",
    description: "Command-line tools and apps installed with `brew`",
    platforms: &[Platform::MacOs],
};

impl Check for Homebrew {
    fn meta(&self) -> &CheckMeta {
        &META
    }

    fn run(&self, _ctx: &SourceContext) -> Result<Vec<Finding>> {
        let Some(brew) = find_brew() else {
            tracing::debug!("brew not installed");
            return Ok(vec![]);
        };
        let out = Command::new(&brew)
            .args(["info", "--json=v2", "--installed"])
            .output()
            .with_context(|| format!("running {brew}"))?;
        anyhow::ensure!(
            out.status.success(),
            "brew info failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let info: BrewInfo = serde_json::from_slice(&out.stdout).context("parsing brew json")?;

        let mut findings = Vec::new();
        for f in info.formulae {
            let requested = f.installed.iter().any(|i| i.installed_on_request);
            let version = f.installed.first().map(|i| i.version.clone()).unwrap_or_default();
            let key = format!("formula/{}", f.name);
            let title = format!("{} {}{}", f.name, version, if requested { "" } else { " (dependency)" });
            findings.push(Finding::new(ID, Group::Packages, key, title).with_value(BrewPackage {
                name: f.name,
                version,
                requested,
                cask: false,
            }));
        }
        for c in info.casks {
            let key = format!("cask/{}", c.token);
            let display = c.name.first().cloned().unwrap_or_else(|| c.token.clone());
            let version = c.installed.unwrap_or_default();
            let title = format!("{display} {version} (cask)");
            findings.push(Finding::new(ID, Group::Applications, key, title).with_value(BrewPackage {
                name: c.token,
                version,
                requested: true,
                cask: true,
            }));
        }
        Ok(findings)
    }
}

fn find_brew() -> Option<String> {
    ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"]
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
        .map(str::to_string)
}

#[derive(Deserialize)]
struct BrewInfo {
    formulae: Vec<Formula>,
    casks: Vec<Cask>,
}

#[derive(Deserialize)]
struct Formula {
    name: String,
    installed: Vec<Installed>,
}

#[derive(Deserialize)]
struct Installed {
    version: String,
    installed_on_request: bool,
}

#[derive(Deserialize)]
struct Cask {
    token: String,
    #[serde(default)]
    name: Vec<String>,
    installed: Option<String>,
}
