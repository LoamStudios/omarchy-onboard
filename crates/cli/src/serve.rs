use crate::scan;
use anyhow::Result;
use console::style;
use omarchy_onboard_core::{Discovery, Platform};
use omarchy_onboard_net::protocol::CheckInfo;
use omarchy_onboard_net::server::Source;
use omarchy_onboard_net::PairingCode;
use std::sync::Arc;

struct LocalSource;

impl Source for LocalSource {
    fn host(&self) -> String {
        scan::hostname()
    }
    fn platform(&self) -> Platform {
        Platform::current()
    }
    fn checks(&self) -> Vec<CheckInfo> {
        omarchy_onboard_checks::for_platform(Platform::current())
            .iter()
            .map(|c| {
                let m = c.meta();
                CheckInfo { id: m.id.into(), group: m.group, title: m.title.into(), description: m.description.into() }
            })
            .collect()
    }
    fn discover(&self, only: &[String]) -> Result<Discovery> {
        eprintln!("Running checks…");
        scan::discover(only)
    }
}

pub fn serve(code: Option<&str>) -> Result<()> {
    let code = match code {
        Some(c) => PairingCode::parse(c)?,
        None => PairingCode::generate(),
    };
    println!("On your Omarchy machine, run:\n");
    println!("    omarchy-onboard migrate {}\n", style(code.to_string()).bold().green());
    println!("Waiting for it to pair… (Ctrl-C to stop)");
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(omarchy_onboard_net::serve(code, Arc::new(LocalSource)))?;
    println!("Done.");
    Ok(())
}
