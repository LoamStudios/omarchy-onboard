use crate::operation::Operation;
use crate::plan::Decision;
use crate::platform::Group;
use serde::{Deserialize, Serialize};

/// Whether a proposal is something to *do* or something to *know*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Kind {
    /// Appears in the checklist; runs operations when accepted.
    Action,
    /// Informational. Shown after the plan, never a decision.
    Note { category: NoteCategory },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteCategory {
    /// Omarchy already provides it.
    Covered,
    /// Source-platform-specific; no purpose on Linux.
    NotNeeded,
    /// No direct equivalent, but here's what people use instead.
    Suggestion,
    /// We don't know this one yet — a to-do for the mapping tables.
    Unknown,
}

impl NoteCategory {
    pub const ALL: &[NoteCategory] = &[
        NoteCategory::Covered,
        NoteCategory::NotNeeded,
        NoteCategory::Suggestion,
        NoteCategory::Unknown,
    ];

    pub fn title(self) -> &'static str {
        match self {
            NoteCategory::Covered => "Already covered by Omarchy",
            NoteCategory::NotNeeded => "Not needed on Linux",
            NoteCategory::Suggestion => "No direct equivalent — suggestions",
            NoteCategory::Unknown => "Unknown — no mapping yet",
        }
    }
}

/// One thing the user accepts or skips (an action), or one thing worth
/// telling them (a note). Actions are composed of `Operation` primitives
/// applied in order; both are traceable to the findings that justified them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Stable id, e.g. `packages/ripgrep`. Used to persist decisions.
    pub id: String,
    #[serde(flatten)]
    pub kind: Kind,
    pub group: Group,
    pub title: String,
    /// Why this is the equivalent — shown beside the title. For notes, the note itself.
    pub rationale: String,
    /// Ids of the findings (`topic:key`) this proposal derives from.
    pub findings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<Operation>,
    /// What we recommend if the user hits "accept all defaults".
    pub default: Decision,
}

impl Proposal {
    pub fn action(
        id: impl Into<String>,
        group: Group,
        title: impl Into<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: Kind::Action,
            group,
            title: title.into(),
            rationale: rationale.into(),
            findings: vec![],
            operations: vec![],
            default: Decision::Accept,
        }
    }

    pub fn note(
        id: impl Into<String>,
        category: NoteCategory,
        group: Group,
        title: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: Kind::Note { category },
            group,
            title: title.into(),
            rationale: text.into(),
            findings: vec![],
            operations: vec![],
            default: Decision::Skip,
        }
    }

    pub fn from(mut self, finding_id: String) -> Self {
        self.findings.push(finding_id);
        self
    }

    pub fn with(mut self, op: Operation) -> Self {
        self.operations.push(op);
        self
    }

    pub fn skip_by_default(mut self) -> Self {
        self.default = Decision::Skip;
        self
    }

    pub fn is_action(&self) -> bool {
        self.kind == Kind::Action
    }

    pub fn needs_source_files(&self) -> bool {
        self.operations.iter().any(Operation::needs_source_files)
    }
}
