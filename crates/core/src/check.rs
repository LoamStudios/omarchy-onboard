use crate::finding::Finding;
use crate::platform::{Group, Platform};
use anyhow::Result;
use std::path::PathBuf;

/// Static description of a check, used for listing and filtering.
#[derive(Debug, Clone)]
pub struct CheckMeta {
    /// Stable id, e.g. `macos.homebrew`.
    pub id: &'static str,
    pub group: Group,
    pub title: &'static str,
    /// One line shown during Discover so users learn what's on their machine.
    pub description: &'static str,
    pub platforms: &'static [Platform],
}

/// What a check may look at on the source machine.
#[derive(Debug, Clone)]
pub struct SourceContext {
    pub platform: Platform,
    pub home: PathBuf,
}

impl SourceContext {
    pub fn current() -> Result<Self> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
        Ok(Self { platform: Platform::current(), home })
    }
}

/// Runs on the **source** machine. Must be side-effect free.
pub trait Check: Send + Sync {
    fn meta(&self) -> &CheckMeta;
    fn run(&self, ctx: &SourceContext) -> Result<Vec<Finding>>;

    fn supports(&self, platform: Platform) -> bool {
        self.meta().platforms.contains(&platform)
    }
}
