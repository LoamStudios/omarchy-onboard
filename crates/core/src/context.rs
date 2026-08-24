use crate::operation::PackageSource;
use crate::platform::Platform;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// What a topic may look at on the source machine.
#[derive(Debug, Clone)]
pub struct SourceContext {
    pub platform: Platform,
    pub home: PathBuf,
}

impl SourceContext {
    pub fn current() -> Result<Self> {
        Ok(Self {
            platform: Platform::current(),
            home: home_dir()?,
        })
    }
}

/// Answers "can the target install `name`, and from where?". Topics use this
/// to avoid proposing packages that don't exist. Implementations live in the
/// target crate; [`NoIndex`] is the stub for platforms without one.
pub trait PackageIndex: Send + Sync {
    fn lookup(&self, name: &str) -> Option<PackageSource>;
}

/// Knows nothing; every lookup is `None`.
pub struct NoIndex;

impl PackageIndex for NoIndex {
    fn lookup(&self, _name: &str) -> Option<PackageSource> {
        None
    }
}

/// What a topic may know about, and ask of, the target machine.
#[derive(Clone)]
pub struct TargetContext {
    pub platform: Platform,
    pub home: PathBuf,
    pub packages: Arc<dyn PackageIndex>,
}

impl TargetContext {
    pub fn current(packages: Arc<dyn PackageIndex>) -> Result<Self> {
        Ok(Self {
            platform: Platform::current(),
            home: home_dir()?,
            packages,
        })
    }
}

impl std::fmt::Debug for TargetContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetContext")
            .field("platform", &self.platform)
            .field("home", &self.home)
            .finish()
    }
}

pub fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))
}
