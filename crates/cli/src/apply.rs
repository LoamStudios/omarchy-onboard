//! Migrate phase. Only dry-run for now; the real executor lands with the
//! transport, since `PullFiles` needs the source connection.

use crate::{plan::describe, ui};
use anyhow::{Context, Result};
use omarchy_onboard_core::Plan;
use std::path::Path;

pub fn apply(path: &Path, dry_run: bool) -> Result<()> {
    let plan: Plan = serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?,
    )?;
    if !dry_run {
        anyhow::bail!("executor not implemented yet; use --dry-run");
    }
    ui::heading("Would apply");
    for p in plan.accepted() {
        ui::item(&format!("{}  {}", console::style(&p.id).green(), describe(p)));
    }
    Ok(())
}
