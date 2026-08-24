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
    /// Check ids that ran (including those that found nothing).
    pub checks_run: Vec<String>,
    /// Check id → error message, for checks that failed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub checks_failed: BTreeMap<String, String>,
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

    pub fn for_check<'a>(&'a self, check: &str) -> impl Iterator<Item = &'a Finding> {
        self.findings.iter().filter(move |f| f.check == check)
    }
}
