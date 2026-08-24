//! `omarchy-migrate` — Omarchy migration assistant.
//!
//! Source machine:  `omarchy-migrate serve`           (advertise, wait for a pair request)
//! Target machine:  `omarchy-migrate migrate <code>`  (discover → propose → migrate)
//!
//! Either machine, offline:
//!   `omarchy-migrate checks`             list what Discover can look at
//!   `omarchy-migrate scan`               run checks here, write discovery.json
//!   `omarchy-migrate plan`               propose from a discovery, decide, write plan.json
//!   `omarchy-migrate apply --dry-run`    show what Migrate would do

mod apply;
mod plan;
mod scan;
mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "omarchy-migrate", version, about = "Omarchy migration assistant")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List available checks for this (or a given) platform.
    Checks {
        /// Show checks for all platforms, not just this one.
        #[arg(long)]
        all: bool,
    },
    /// Discover: run checks on this machine and write findings.
    Scan {
        /// Where to write the discovery.
        #[arg(short, long, default_value = "discovery.json")]
        out: PathBuf,
        /// Only run these check ids.
        #[arg(long)]
        check: Vec<String>,
    },
    /// Propose: turn a discovery into a plan, interactively deciding what to accept.
    Plan {
        /// Discovery to read. Use `--local` to scan this machine instead.
        #[arg(short, long, default_value = "discovery.json")]
        discovery: PathBuf,
        /// Scan this machine now rather than reading a discovery file.
        #[arg(long)]
        local: bool,
        /// Where to write the plan.
        #[arg(short, long, default_value = "plan.json")]
        out: PathBuf,
        /// Accept every proposal's default without prompting.
        #[arg(short, long)]
        yes: bool,
    },
    /// Migrate: apply the accepted operations in a plan.
    Apply {
        #[arg(default_value = "plan.json")]
        plan: PathBuf,
        /// Print what would happen without doing it.
        #[arg(long)]
        dry_run: bool,
    },
    /// Run on the source machine: wait for a target to pair and pull from us.
    Serve,
    /// Run on the target machine: pair with a source and migrate from it.
    Migrate {
        /// Pairing code shown by `omarchy-migrate serve`.
        code: String,
    },
    /// Print the usage spec (for completions and docs).
    #[command(hide = true)]
    Usage,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    match Cli::parse().cmd {
        Cmd::Checks { all } => scan::list_checks(all),
        Cmd::Scan { out, check } => scan::scan(&out, &check),
        Cmd::Plan { discovery, local, out, yes } => plan::plan(&discovery, local, &out, yes),
        Cmd::Apply { plan, dry_run } => apply::apply(&plan, dry_run),
        Cmd::Serve => anyhow::bail!("pairing transport not implemented yet — use `omarchy-migrate scan` and copy discovery.json"),
        Cmd::Migrate { .. } => anyhow::bail!("pairing transport not implemented yet — use `omarchy-migrate plan --discovery <file>`"),
        Cmd::Usage => {
            let mut buf = Vec::new();
            clap_usage::generate(&mut Cli::command(), "omarchy-migrate", &mut buf);
            print!("{}", String::from_utf8(buf)?);
            Ok(())
        }
    }
}
