use crate::platform::Group;
use crate::proposal::Proposal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Accept,
    Skip,
}

/// Output of the Propose phase plus the user's decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub proposals: Vec<Proposal>,
    /// Proposal id → decision. Missing entries fall back to the proposal's default.
    #[serde(default)]
    pub decisions: BTreeMap<String, Decision>,
}

impl Plan {
    pub fn new(proposals: Vec<Proposal>) -> Self {
        Self {
            proposals,
            decisions: BTreeMap::new(),
        }
    }

    pub fn decision(&self, p: &Proposal) -> Decision {
        self.decisions.get(&p.id).copied().unwrap_or(p.default)
    }

    pub fn decide(&mut self, id: &str, d: Decision) {
        self.decisions.insert(id.to_string(), d);
    }

    pub fn decide_group(&mut self, group: Group, d: Decision) {
        let ids: Vec<String> = self
            .proposals
            .iter()
            .filter(|p| p.group == group)
            .map(|p| p.id.clone())
            .collect();
        for id in ids {
            self.decisions.insert(id, d);
        }
    }

    pub fn accept_all(&mut self) {
        let ids: Vec<String> = self.proposals.iter().map(|p| p.id.clone()).collect();
        for id in ids {
            self.decisions.insert(id, Decision::Accept);
        }
    }

    pub fn accepted(&self) -> impl Iterator<Item = &Proposal> {
        self.proposals
            .iter()
            .filter(|p| self.decision(p) == Decision::Accept)
    }

    pub fn by_group(&self) -> BTreeMap<Group, Vec<&Proposal>> {
        let mut map: BTreeMap<Group, Vec<&Proposal>> = BTreeMap::new();
        for p in &self.proposals {
            map.entry(p.group).or_default().push(p);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    fn p(id: &str, group: Group, default: Decision) -> Proposal {
        Proposal {
            id: id.into(),
            group,
            title: id.into(),
            rationale: String::new(),
            findings: vec![],
            operations: vec![Operation::Manual {
                instructions: String::new(),
            }],
            default,
        }
    }

    #[test]
    fn decisions_fall_back_to_defaults_and_can_be_overridden_per_group() {
        let mut plan = Plan::new(vec![
            p("a", Group::Shell, Decision::Accept),
            p("b", Group::Shell, Decision::Skip),
            p("c", Group::Keys, Decision::Skip),
        ]);
        assert_eq!(
            plan.accepted().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            ["a"]
        );
        plan.decide_group(Group::Shell, Decision::Skip);
        assert_eq!(plan.accepted().count(), 0);
        plan.decide("c", Decision::Accept);
        assert_eq!(
            plan.accepted().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            ["c"]
        );
        plan.accept_all();
        assert_eq!(plan.accepted().count(), 3);
    }

    #[test]
    fn plan_round_trips_through_json() {
        let mut plan = Plan::new(vec![p("a", Group::Shell, Decision::Accept)]);
        plan.decide("a", Decision::Skip);
        let back: Plan = serde_json::from_str(&serde_json::to_string(&plan).unwrap()).unwrap();
        assert_eq!(back.decision(&back.proposals[0]), Decision::Skip);
    }
}
