use crate::platform::Group;
use crate::proposal::{Kind, NoteCategory, Proposal};
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

    pub fn actions(&self) -> impl Iterator<Item = &Proposal> {
        self.proposals.iter().filter(|p| p.is_action())
    }

    pub fn notes(&self) -> impl Iterator<Item = &Proposal> {
        self.proposals.iter().filter(|p| !p.is_action())
    }

    /// Accepted actions. Notes are never "accepted".
    pub fn accepted(&self) -> impl Iterator<Item = &Proposal> {
        self.actions()
            .filter(|p| self.decision(p) == Decision::Accept)
    }

    /// Actions by group (the checklist).
    pub fn by_group(&self) -> BTreeMap<Group, Vec<&Proposal>> {
        let mut map: BTreeMap<Group, Vec<&Proposal>> = BTreeMap::new();
        for p in self.actions() {
            map.entry(p.group).or_default().push(p);
        }
        map
    }

    /// Notes by category.
    pub fn notes_by_category(&self) -> BTreeMap<NoteCategory, Vec<&Proposal>> {
        let mut map: BTreeMap<NoteCategory, Vec<&Proposal>> = BTreeMap::new();
        for p in self.notes() {
            if let Kind::Note { category } = p.kind {
                map.entry(category).or_default().push(p);
            }
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    fn p(id: &str, group: Group, default: Decision) -> Proposal {
        let mut p = Proposal::action(id, group, id, "").with(Operation::Manual {
            instructions: String::new(),
        });
        p.default = default;
        p
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

#[cfg(test)]
mod note_tests {
    use super::*;
    use crate::proposal::NoteCategory;

    #[test]
    fn notes_are_never_accepted_and_group_by_category() {
        let mut plan = Plan::new(vec![
            Proposal::action("a", Group::Packages, "a", ""),
            Proposal::note(
                "n1",
                NoteCategory::Covered,
                Group::Packages,
                "git",
                "ships with Omarchy",
            ),
            Proposal::note("n2", NoteCategory::Unknown, Group::Packages, "foo", "?"),
        ]);
        plan.accept_all();
        assert_eq!(
            plan.accepted().map(|p| p.id.as_str()).collect::<Vec<_>>(),
            ["a"]
        );
        assert_eq!(plan.by_group()[&Group::Packages].len(), 1);
        let notes = plan.notes_by_category();
        assert_eq!(notes[&NoteCategory::Covered][0].id, "n1");
        assert_eq!(notes[&NoteCategory::Unknown][0].id, "n2");
    }
}
