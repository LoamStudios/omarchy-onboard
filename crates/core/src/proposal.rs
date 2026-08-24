use crate::operation::Operation;
use crate::plan::Decision;
use crate::platform::Group;
use serde::{Deserialize, Serialize};

/// One thing the user accepts or skips, composed of `Operation` primitives
/// applied in order, traceable back to the findings that justified it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Stable id, e.g. `packages/ripgrep`. Used to persist decisions.
    pub id: String,
    pub group: Group,
    pub title: String,
    /// Why this is the equivalent — shown to the user beside the title.
    pub rationale: String,
    /// Ids of the findings (`topic:key`) this proposal derives from.
    pub findings: Vec<String>,
    pub operations: Vec<Operation>,
    /// What we recommend if the user hits "accept all defaults".
    pub default: Decision,
}

impl Proposal {
    pub fn needs_source_files(&self) -> bool {
        self.operations.iter().any(Operation::needs_source_files)
    }

    pub fn is_manual_only(&self) -> bool {
        self.operations.iter().all(Operation::is_manual)
    }
}
