//! `omarchy-onboard` — Omarchy migration assistant.
//!
//! Source machine:  `omarchy-onboard serve`           (advertise, wait for a pair request)
//! Target machine:  `omarchy-onboard migrate <code>`  (discover → propose → migrate)
//!
//! Either machine, offline:
//!   `omarchy-onboard topics`             list what Discover can look at
//!   `omarchy-onboard scan`               run checks here, write discovery.json
//!   `omarchy-onboard plan`               propose from a discovery, decide, write plan.json
//!   `omarchy-onboard apply --dry-run`    show what Migrate would do

mod apply;
mod migrate;
mod plan;
mod scan;
mod serve;
mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "omarchy-onboard",
    version,
    about = "Move your computer to Omarchy"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List topics: what Discover can look at, and on which platforms.
    Topics {
        /// Show topics for all platforms, not just this one.
        #[arg(long)]
        all: bool,
    },
    /// Discover: run topics on this machine and write findings.
    Scan {
        /// Where to write the discovery.
        #[arg(short, long, default_value = "discovery.json")]
        out: PathBuf,
        /// Only run these topic ids.
        #[arg(long)]
        topic: Vec<String>,
        /// List every finding instead of a per-group summary.
        #[arg(short, long)]
        verbose: bool,
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
        /// Package list to use instead of the live pacman index (e.g. `pacman -Slq > pkgs.txt`).
        #[arg(long)]
        packages: Option<PathBuf>,
    },
    /// Migrate: apply the accepted operations in a plan (needs a paired source for file pulls).
    Apply {
        #[arg(default_value = "plan.json")]
        plan: PathBuf,
        /// Print what would happen without doing it.
        #[arg(long)]
        dry_run: bool,
        /// Pairing code of the source, needed if the plan pulls files.
        #[arg(long)]
        code: Option<String>,
    },
    /// Run on the source machine: wait for a target to pair and pull from us.
    Serve {
        /// Use a fixed pairing code instead of generating one.
        #[arg(long)]
        code: Option<String>,
    },
    /// Run on the target machine: pair with a source and migrate from it.
    Migrate {
        /// Pairing code shown by `omarchy-onboard serve`.
        code: String,
        /// Accept every proposal's default without prompting.
        #[arg(short, long)]
        yes: bool,
        /// Plan only; don't apply.
        #[arg(long)]
        dry_run: bool,
        /// Package list to use instead of the live pacman index.
        #[arg(long)]
        packages: Option<PathBuf>,
        /// List every finding instead of a per-group summary.
        #[arg(short, long)]
        verbose: bool,
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
        Cmd::Topics { all } => scan::list_topics(all),
        Cmd::Scan {
            out,
            topic,
            verbose,
        } => scan::scan(&out, &topic, verbose),
        Cmd::Plan {
            discovery,
            local,
            out,
            yes,
            packages,
        } => plan::plan(&discovery, local, &out, yes, packages.as_deref()),
        Cmd::Apply {
            plan,
            dry_run,
            code,
        } => apply::apply(&plan, dry_run, code.as_deref()),
        Cmd::Serve { code } => serve::serve(code.as_deref()),
        Cmd::Migrate {
            code,
            yes,
            dry_run,
            packages,
            verbose,
        } => migrate::migrate(&code, yes, dry_run, packages.as_deref(), verbose),
        Cmd::Usage => {
            let mut buf = Vec::new();
            clap_usage::generate(&mut Cli::command(), "omarchy-onboard", &mut buf);
            print!("{}", String::from_utf8(buf)?);
            Ok(())
        }
    }
}
