use crate::{scan, ui};
use anyhow::{Context, Result};
use console::style;
use demand::{DemandOption, MultiSelect};
use omam_core::{Decision, Discovery, Operation, Plan, Platform, Proposal, TargetContext};
use std::path::Path;

pub fn plan(discovery: &Path, local: bool, out: &Path, yes: bool) -> Result<()> {
    let discovery: Discovery = if local {
        scan::discover(&[])?
    } else {
        let s = std::fs::read_to_string(discovery)
            .with_context(|| format!("reading {} (run `omam scan` first, or pass --local)", discovery.display()))?;
        serde_json::from_str(&s)?
    };

    let ctx = TargetContext {
        platform: Platform::current(),
        home: std::env::var_os("HOME").map(Into::into).unwrap_or_default(),
    };
    let mut plan = omam_rules::propose(&discovery, &ctx);

    if plan.proposals.is_empty() {
        println!("Nothing to propose.");
        return Ok(());
    }

    if yes {
        for p in &plan.proposals {
            plan.decisions.insert(p.id.clone(), p.default);
        }
    } else {
        decide_interactively(&mut plan)?;
    }

    print_plan(&plan);
    std::fs::write(out, serde_json::to_string_pretty(&plan)?)?;
    let accepted = plan.accepted().count();
    println!("\nWrote plan with {accepted}/{} accepted to {}", plan.proposals.len(), out.display());
    Ok(())
}

/// One multi-select per group; the proposal's default sets the initial checkbox.
fn decide_interactively(plan: &mut Plan) -> Result<()> {
    let groups = plan.by_group();
    let mut chosen: Vec<String> = Vec::new();
    for (group, proposals) in &groups {
        let mut ms = MultiSelect::new(format!("{} ({})", group.title(), proposals.len()))
            .description("space: toggle · a: all · enter: confirm")
            .filterable(true);
        for p in proposals {
            ms = ms.option(
                DemandOption::new(p.id.clone())
                    .label(&p.title)
                    .description(&p.rationale)
                    .selected(p.default == Decision::Accept),
            );
        }
        chosen.extend(ms.run()?);
    }
    let all_ids: Vec<String> = plan.proposals.iter().map(|p| p.id.clone()).collect();
    for id in all_ids {
        let d = if chosen.contains(&id) { Decision::Accept } else { Decision::Skip };
        plan.decide(&id, d);
    }
    Ok(())
}

pub fn print_plan(plan: &Plan) {
    ui::heading("Plan");
    for (group, proposals) in plan.by_group() {
        let accepted = proposals.iter().filter(|p| plan.decision(p) == Decision::Accept).count();
        ui::group(group.title(), accepted);
        for p in proposals {
            let mark = match plan.decision(p) {
                Decision::Accept => style("✓").green(),
                Decision::Skip => style("·").dim(),
            };
            ui::item(&format!("{mark} {}", p.title));
            ui::note(&describe(p));
        }
    }
}

pub fn describe(p: &Proposal) -> String {
    match &p.operation {
        Operation::InstallPackages { packages } => {
            let names: Vec<_> = packages.iter().map(|p| format!("{} [{:?}]", p.name, p.source)).collect();
            format!("install {}", names.join(", "))
        }
        Operation::InstallEditorExtension { editor, extension } => format!("{editor}: install extension {extension}"),
        Operation::PullFiles { items, dest } => {
            let size: u64 = items.iter().map(|i| i.size).sum();
            format!("pull {} item(s), {} → {}", items.len(), ui::human_bytes(size), dest.display())
        }
        Operation::WriteConfig { path, mode, .. } => format!("write {} ({mode:?})", path.display()),
        Operation::SetTheme { name } => format!("set theme {name}"),
        Operation::RunCommand { argv } => format!("run {}", argv.join(" ")),
        Operation::Manual { instructions } => format!("manual: {instructions}"),
    }
}
