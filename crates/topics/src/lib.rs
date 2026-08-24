//! Every topic, one directory each:
//!
//! ```text
//! topics/ssh/mod.rs     meta + propose (target side)
//! topics/ssh/macos.rs   discover on macOS
//! topics/ssh/windows.rs discover on Windows (when it exists)
//! ```
//!
//! Register new topics in [`all`]. Mapping tables (`*.toml`) live beside the
//! topic that owns them.

use anyhow::Result;
use omarchy_onboard_core::{Discovery, Plan, Platform, SourceContext, TargetContext, Topic};

pub mod fonts;
pub mod fs;
pub mod homebrew;
pub mod input;
pub mod shell;
pub mod ssh;
pub mod terminal;
pub mod vscode;

pub fn all() -> Vec<Box<dyn Topic>> {
    vec![
        Box::new(homebrew::Homebrew),
        Box::new(shell::Shell),
        Box::new(ssh::Ssh),
        Box::new(input::Input),
        Box::new(vscode::VsCode),
        Box::new(fonts::Fonts),
        Box::new(terminal::Terminal),
    ]
}

/// Topics that can read `platform`.
pub fn for_source(platform: Platform) -> Vec<Box<dyn Topic>> {
    all().into_iter().filter(|t| t.reads(platform)).collect()
}

/// Run every applicable topic's `discover` on this machine.
pub fn discover(ctx: &SourceContext, host: &str, only: &[String]) -> Result<Discovery> {
    let mut discovery = Discovery::new(ctx.platform, host);
    for topic in for_source(ctx.platform) {
        let id = topic.meta().id;
        if !only.is_empty() && !only.iter().any(|o| o == id) {
            continue;
        }
        tracing::info!(topic = id, "discovering");
        discovery.topics_run.push(id.to_string());
        match topic.discover(ctx) {
            Ok(f) => discovery.findings.extend(f),
            Err(e) => {
                discovery
                    .topics_failed
                    .insert(id.to_string(), format!("{e:#}"));
            }
        }
    }
    Ok(discovery)
}

/// Run every topic's `propose` against a discovery.
pub fn propose(discovery: &Discovery, ctx: &TargetContext) -> Plan {
    let mut proposals = Vec::new();
    for topic in all() {
        let mine: Vec<&_> = discovery.for_topic(topic.meta().id).collect();
        if mine.is_empty() {
            continue;
        }
        proposals.extend(topic.propose(&mine, discovery, ctx));
    }
    Plan::new(proposals)
}
