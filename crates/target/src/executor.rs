use anyhow::{Context, Result};
use omarchy_onboard_core::{ConfigMode, FileRef, Operation, PackageSource};
use std::path::Path;
use std::process::Command;

/// Where `PullFiles` gets its bytes. Implemented by the network client; a
/// local implementation can copy from disk for testing.
pub trait FileSource {
    /// Materialise `item` at `dest` (file → file, directory → directory).
    fn fetch(&mut self, item: &FileRef, dest: &Path) -> Result<()>;
}

#[derive(Debug)]
pub enum Outcome {
    Done,
    /// Nothing to execute; shown to the user.
    Manual(String),
    Skipped(String),
}

pub struct Executor {
    pub dry_run: bool,
}

impl Executor {
    pub fn apply(&self, op: &Operation, files: &mut dyn FileSource) -> Result<Outcome> {
        if self.dry_run {
            return Ok(Outcome::Skipped("dry run".into()));
        }
        match op {
            Operation::InstallPackages { packages } => {
                let pacman: Vec<&str> =
                    packages.iter().filter(|p| p.source == PackageSource::Pacman).map(|p| p.name.as_str()).collect();
                let aur: Vec<&str> =
                    packages.iter().filter(|p| p.source == PackageSource::Aur).map(|p| p.name.as_str()).collect();
                if !pacman.is_empty() {
                    run("sudo", &[&["pacman", "-S", "--needed", "--noconfirm"][..], &pacman].concat())?;
                }
                if !aur.is_empty() {
                    run("yay", &[&["-S", "--needed", "--noconfirm"][..], &aur].concat())?;
                }
                for p in packages.iter().filter(|p| p.source == PackageSource::DistroInstaller) {
                    run(&format!("omarchy-install-{}", p.name), &[])?;
                }
                Ok(Outcome::Done)
            }
            Operation::InstallEditorExtension { editor, extension } => {
                let bin = match editor.as_str() {
                    "vscode" => "code",
                    "cursor" => "cursor",
                    other => other,
                };
                run(bin, &["--install-extension", extension])?;
                Ok(Outcome::Done)
            }
            Operation::PullFiles { items, dest } => {
                for item in items {
                    let target = if items.len() == 1 {
                        dest.clone()
                    } else {
                        dest.join(item.path.file_name().unwrap_or_default())
                    };
                    if let Some(parent) = target.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    files.fetch(item, &target)?;
                }
                Ok(Outcome::Done)
            }
            Operation::WriteConfig { path, content, mode } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                match mode {
                    ConfigMode::Replace => std::fs::write(path, content)?,
                    ConfigMode::Append => {
                        let existing = std::fs::read_to_string(path).unwrap_or_default();
                        if !existing.contains(content.as_str()) {
                            let mut s = existing;
                            if !s.is_empty() && !s.ends_with('\n') {
                                s.push('\n');
                            }
                            s.push_str(content);
                            std::fs::write(path, s)?;
                        }
                    }
                }
                Ok(Outcome::Done)
            }
            Operation::SetTheme { name } => {
                run("omarchy-theme-set", &[name])?;
                Ok(Outcome::Done)
            }
            Operation::RunCommand { argv } => {
                let (bin, args) = argv.split_first().context("empty argv")?;
                let args: Vec<&str> = args.iter().map(String::as_str).collect();
                run(bin, &args)?;
                Ok(Outcome::Done)
            }
            Operation::Manual { instructions } => Ok(Outcome::Manual(instructions.clone())),
        }
    }
}

fn run(bin: &str, args: &[&str]) -> Result<()> {
    tracing::info!(bin, ?args, "exec");
    let status = Command::new(bin).args(args).status().with_context(|| format!("running {bin}"))?;
    anyhow::ensure!(status.success(), "{bin} {} exited with {status}", args.join(" "));
    Ok(())
}
