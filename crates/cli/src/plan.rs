use crate::{scan, ui};
use anyhow::{Context, Result};
use console::style;
use demand::{DemandOption, MultiSelect};
use omarchy_onboard_core::{
    Decision, Discovery, NoIndex, Operation, PackageIndex, Plan, Proposal, TargetContext,
};
use omarchy_onboard_target::{ListIndex, PacmanIndex};
use std::path::Path;
use std::sync::Arc;

pub fn plan(
    discovery: &Path,
    local: bool,
    out: &Path,
    yes: bool,
    packages: Option<&Path>,
) -> Result<()> {
    let discovery: Discovery = if local {
        scan::discover(&[])?
    } else {
        let s = std::fs::read_to_string(discovery).with_context(|| {
            format!(
                "reading {} (run `omarchy-onboard scan` first, or pass --local)",
                discovery.display()
            )
        })?;
        serde_json::from_str(&s)?
    };

    let mut plan = propose(&discovery, packages)?;
    if plan.proposals.is_empty() {
        println!("Nothing to propose.");
        return Ok(());
    }

    if yes {
        let defaults: Vec<_> = plan.actions().map(|p| (p.id.clone(), p.default)).collect();
        plan.decisions.extend(defaults);
    } else {
        decide_interactively(&mut plan)?;
    }

    print_plan(&plan);
    print_notes(&plan);
    std::fs::write(out, serde_json::to_string_pretty(&plan)?)?;
    println!(
        "\nWrote plan with {}/{} actions accepted and {} notes to {}",
        plan.accepted().count(),
        plan.actions().count(),
        plan.notes().count(),
        out.display()
    );
    Ok(())
}

/// Build the target context (live pacman index on Arch, a list file, or nothing) and run topics.
pub fn propose(discovery: &Discovery, packages: Option<&Path>) -> Result<Plan> {
    let index: Arc<dyn PackageIndex> = match packages {
        Some(p) => {
            Arc::new(ListIndex::from_file(p).with_context(|| format!("reading {}", p.display()))?)
        }
        None => match PacmanIndex::load() {
            Ok(i) => Arc::new(i),
            Err(e) => {
                tracing::warn!(
                    "no pacman index ({e:#}); unmapped packages will be proposed as manual"
                );
                Arc::new(NoIndex)
            }
        },
    };
    let ctx = TargetContext::current(index)?;
    Ok(omarchy_onboard_topics::propose(discovery, &ctx))
}

/// One multi-select per group; the proposal's default sets the initial checkbox.
pub fn decide_interactively(plan: &mut Plan) -> Result<()> {
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
    let all_ids: Vec<String> = plan.actions().map(|p| p.id.clone()).collect();
    for id in all_ids {
        let d = if chosen.contains(&id) {
            Decision::Accept
        } else {
            Decision::Skip
        };
        plan.decide(&id, d);
    }
    Ok(())
}

pub fn print_plan(plan: &Plan) {
    ui::heading("Plan");
    for (group, proposals) in plan.by_group() {
        let accepted = proposals
            .iter()
            .filter(|p| plan.decision(p) == Decision::Accept)
            .count();
        ui::group(group.title(), &format!("{accepted}/{}", proposals.len()));
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

pub fn print_notes(plan: &Plan) {
    let notes = plan.notes_by_category();
    if notes.is_empty() {
        return;
    }
    ui::heading("Notes");
    for (category, items) in notes {
        ui::group(category.title(), &items.len().to_string());
        for p in items {
            ui::item(&format!(
                "{}  {}",
                style(&p.title).bold(),
                style(&p.rationale).dim()
            ));
        }
    }
}

pub fn describe(p: &Proposal) -> String {
    p.operations
        .iter()
        .map(describe_op)
        .collect::<Vec<_>>()
        .join("; ")
}

fn describe_op(op: &Operation) -> String {
    match op {
        Operation::InstallPackages { packages } => {
            let names: Vec<_> = packages
                .iter()
                .map(|p| format!("{} [{:?}]", p.name, p.source))
                .collect();
            format!("install {}", names.join(", "))
        }
        Operation::InstallEditorExtension { editor, extension } => {
            format!("{editor}: install extension {extension}")
        }
        Operation::PullFiles { items, dest, mode } => {
            let size: u64 = items.iter().map(|i| i.size).sum();
            let m = mode.map(|m| format!(" mode {m:o}")).unwrap_or_default();
            format!(
                "pull {} item(s), {} → {}{m}",
                items.len(),
                ui::human_bytes(size),
                dest.display()
            )
        }
        Operation::WriteConfig { path, mode, .. } => format!("write {} ({mode:?})", path.display()),
        Operation::SetTheme { name } => format!("set theme {name}"),
        Operation::RunCommand { argv } => format!("run {}", argv.join(" ")),
        Operation::Manual { instructions } => format!("manual: {instructions}"),
    }
}
