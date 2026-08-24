use crate::{apply, plan, scan, ui};
use anyhow::Result;
use omarchy_onboard_net::{Client, PairingCode};
use std::path::Path;
use std::time::Duration;

pub fn connect(code: &str) -> Result<Client> {
    let code = PairingCode::parse(code)?;
    println!("Looking for a source with code {code} on the local network…");
    let client = Client::pair(&code, Duration::from_secs(30))?;
    println!(
        "Paired with {} ({:?})",
        console::style(&client.host).bold(),
        client.platform
    );
    Ok(client)
}

pub fn migrate(code: &str, yes: bool, dry_run: bool, packages: Option<&Path>) -> Result<()> {
    let mut client = connect(code)?;

    ui::heading("Discover");
    let discovery = client.discover(&[])?;
    scan::print_discovery(&discovery);

    ui::heading("Propose");
    let mut p = plan::propose(&discovery, packages)?;
    if p.proposals.is_empty() {
        println!("Nothing to propose.");
        client.close();
        return Ok(());
    }
    if yes {
        for pr in &p.proposals {
            p.decisions.insert(pr.id.clone(), pr.default);
        }
    } else {
        plan::decide_interactively(&mut p)?;
    }
    plan::print_plan(&p);
    std::fs::write("plan.json", serde_json::to_string_pretty(&p)?)?;

    ui::heading("Migrate");
    apply::run(&p, dry_run, &mut client)?;
    client.close();
    Ok(())
}
