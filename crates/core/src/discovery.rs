use crate::finding::Finding;
use crate::platform::{Group, Platform};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Output of the Discover phase: everything the source machine reported.
/// Serializable so it can be saved, diffed, and re-planned offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    pub source_platform: Platform,
    pub source_host: String,
    /// Topic ids that ran (including those that found nothing).
    pub topics_run: Vec<String>,
    /// Topic id → error message, for topics that failed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub topics_failed: BTreeMap<String, String>,
    pub findings: Vec<Finding>,
}

impl Discovery {
    pub fn by_group(&self) -> BTreeMap<Group, Vec<&Finding>> {
        let mut map: BTreeMap<Group, Vec<&Finding>> = BTreeMap::new();
        for f in &self.findings {
            map.entry(f.group).or_default().push(f);
        }
        map
    }

    pub fn for_topic<'a>(&'a self, topic: &str) -> impl Iterator<Item = &'a Finding> {
        self.findings.iter().filter(move |f| f.topic == topic)
    }

    pub fn new(source_platform: Platform, source_host: impl Into<String>) -> Self {
        Self {
            source_platform,
            source_host: source_host.into(),
            topics_run: Vec::new(),
            topics_failed: BTreeMap::new(),
            findings: Vec::new(),
        }
    }
}
