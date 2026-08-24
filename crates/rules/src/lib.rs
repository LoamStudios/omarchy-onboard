//! Target-side rules. Each rule consumes findings from one or more checks and
//! proposes the semantically equivalent operation on the target.
//!
//! Mapping knowledge (brew formula → pacman package, mac app → Linux app) lives
//! in TOML tables under `maps/`, embedded at compile time, so contributors can
//! extend coverage without touching Rust.

use omarchy_onboard_core::{Discovery, Plan, Rule, TargetContext};

pub mod maps;
pub mod packages;
pub mod shell;
pub mod ssh;

pub fn all() -> Vec<Box<dyn Rule>> {
    vec![Box::new(packages::HomebrewToPacman), Box::new(shell::ShellDotfiles), Box::new(ssh::Ssh)]
}

/// Run every rule against a discovery and collect proposals into a plan.
pub fn propose(discovery: &Discovery, ctx: &TargetContext) -> Plan {
    let mut proposals = Vec::new();
    for rule in all() {
        proposals.extend(rule.propose(discovery, ctx));
    }
    Plan::new(proposals)
}
