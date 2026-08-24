use crate::context::{SourceContext, TargetContext};
use crate::discovery::Discovery;
use crate::finding::Finding;
use crate::platform::{Group, Platform};
use crate::proposal::Proposal;
use anyhow::Result;

/// Static description of a topic, used for listing and filtering.
#[derive(Debug, Clone)]
pub struct TopicMeta {
    /// Stable id, e.g. `ssh`. Findings and proposals are keyed by it.
    pub id: &'static str,
    pub group: Group,
    pub title: &'static str,
    /// One line shown during Discover so users learn what's on their machine.
    pub description: &'static str,
    /// Source platforms `discover` knows how to read.
    pub sources: &'static [Platform],
}

/// The unit of authorship: one concern, discovered on the source and
/// proposed on the target.
///
/// `discover` must be side-effect free and must not put secrets into findings
/// (use `FileRef`s). `propose` composes `Operation` primitives; it receives the
/// whole `Discovery` too, for the rare cross-topic case (e.g. install a font
/// because the terminal config references it).
pub trait Topic: Send + Sync {
    fn meta(&self) -> &TopicMeta;
    fn discover(&self, ctx: &SourceContext) -> Result<Vec<Finding>>;
    fn propose(&self, mine: &[&Finding], all: &Discovery, ctx: &TargetContext) -> Vec<Proposal>;

    fn reads(&self, platform: Platform) -> bool {
        self.meta().sources.contains(&platform)
    }
}
