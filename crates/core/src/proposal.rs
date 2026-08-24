use crate::operation::Operation;
use crate::plan::Decision;
use crate::platform::Group;
use serde::{Deserialize, Serialize};

/// A proposed operation, traceable back to the findings that justified it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Stable id, e.g. `packages/ripgrep`. Used to persist decisions.
    pub id: String,
    pub group: Group,
    pub title: String,
    /// Why this is the equivalent — shown to the user beside the title.
    pub rationale: String,
    /// Ids of the findings (`check:key`) this proposal derives from.
    pub findings: Vec<String>,
    pub operation: Operation,
    /// What we recommend if the user hits "accept all defaults".
    pub default: Decision,
}
