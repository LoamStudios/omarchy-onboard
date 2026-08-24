use crate::discovery::Discovery;
use crate::platform::Platform;
use crate::proposal::Proposal;
use std::path::PathBuf;

/// What a rule may know about the target machine.
#[derive(Debug, Clone)]
pub struct TargetContext {
    pub platform: Platform,
    pub home: PathBuf,
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
