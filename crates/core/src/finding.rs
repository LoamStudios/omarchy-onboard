use crate::platform::Group;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A fact discovered on the source machine. Findings are deliberately dumb:
/// they describe what *is*, never what to do about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub topic: String,
    pub group: Group,
    /// Stable key unique within the check, e.g. `formula/ripgrep`.
    /// Topics match on `key`.
    pub key: String,
    /// Short human description, e.g. "Homebrew formula ripgrep 14.1.0".
    pub title: String,
    /// Check-specific structured payload.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub value: serde_json::Value,
    /// Files on the source machine this finding refers to. The target can
    /// request them by reference during Migrate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FileRef>,
}

impl Finding {
    pub fn new(
        topic: &str,
        group: Group,
        key: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            topic: topic.to_string(),
            group,
            key: key.into(),
            title: title.into(),
            value: serde_json::Value::Null,
            files: Vec::new(),
        }
    }

    pub fn with_value(mut self, value: impl Serialize) -> Self {
        self.value = serde_json::to_value(value).expect("finding value serializes");
        self
    }

    pub fn with_file(mut self, file: FileRef) -> Self {
        self.files.push(file);
        self
    }

    /// `topic:key` — globally unique id for this finding.
    pub fn id(&self) -> String {
        format!("{}:{}", self.topic, self.key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    File,
    Directory,
}

/// A file or directory on the source machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    pub path: PathBuf,
    pub kind: FileKind,
    /// Total bytes (recursive for directories). Used to show the user what
    /// a `PullFiles` operation will cost.
    pub size: u64,
}
