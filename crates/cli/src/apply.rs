//! Migrate phase.

use crate::{migrate, plan::describe, ui};
use anyhow::{Context, Result};
use console::style;
use omarchy_onboard_core::{FileRef, Plan};
use omarchy_onboard_target::{Executor, FileSource, Outcome};
use std::path::Path;

pub fn apply(path: &Path, dry_run: bool, code: Option<&str>) -> Result<()> {
    let plan: Plan = serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?,
    )?;
    let needs_source = plan.accepted().any(|p| p.operation.needs_source_files());
    match (needs_source && !dry_run, code) {
        (true, Some(code)) => {
            let mut client = migrate::connect(code)?;
            run(&plan, dry_run, &mut client)?;
            client.close();
            Ok(())
        }
        (true, None) => anyhow::bail!("plan pulls files from the source; pass --code <pairing code>"),
        _ => run(&plan, dry_run, &mut NoSource),
    }
}

pub fn run(plan: &Plan, dry_run: bool, files: &mut dyn FileSource) -> Result<()> {
    let exec = Executor { dry_run };
    let mut manual = Vec::new();
    let mut failed = 0;
    for p in plan.accepted() {
        print!("{} {} … ", style("▸").cyan(), p.title);
        match exec.apply(&p.operation, files) {
            Ok(Outcome::Done) => println!("{}", style("done").green()),
            Ok(Outcome::Skipped(why)) => println!("{} ({why}: {})", style("skipped").dim(), describe(p)),
            Ok(Outcome::Manual(text)) => {
                println!("{}", style("manual").yellow());
                manual.push((p.title.clone(), text));
            }
            Err(e) => {
                failed += 1;
                println!("{}: {e:#}", style("failed").red());
            }
        }
    }
    if !manual.is_empty() {
        ui::heading("To do by hand");
        for (title, text) in manual {
            ui::item(&format!("{}", style(title).bold()));
            ui::note(&text);
        }
    }
    anyhow::ensure!(failed == 0, "{failed} operation(s) failed");
    Ok(())
}

struct NoSource;

impl FileSource for NoSource {
    fn fetch(&mut self, item: &FileRef, _dest: &Path) -> Result<()> {
        anyhow::bail!("no source connected; cannot pull {}", item.path.display())
    }
}
