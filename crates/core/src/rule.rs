use crate::discovery::Discovery;
use crate::operation::PackageSource;
use crate::platform::Platform;
use crate::proposal::Proposal;
use std::path::PathBuf;
use std::sync::Arc;

/// Answers "can the target install `name`, and from where?". Rules use this
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

/// What a rule may know about, and ask of, the target machine.
#[derive(Clone)]
pub struct TargetContext {
    pub platform: Platform,
    pub home: PathBuf,
    pub packages: Arc<dyn PackageIndex>,
}

impl std::fmt::Debug for TargetContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TargetContext").field("platform", &self.platform).field("home", &self.home).finish()
    }
}

/// Runs on the **target** machine. Turns findings into proposals.
///
/// A rule sees the whole `Discovery` (not just its own check's findings) so it
/// can, e.g., propose a terminal font install only if the font was also found.
pub trait Rule: Send + Sync {
    /// Stable id, e.g. `homebrew-to-pacman`.
    fn id(&self) -> &'static str;
    /// Which source checks this rule consumes. Used to explain to the user
    /// why a finding produced no proposal.
    fn consumes(&self) -> &'static [&'static str];
    fn propose(&self, discovery: &Discovery, ctx: &TargetContext) -> Vec<Proposal>;
}
