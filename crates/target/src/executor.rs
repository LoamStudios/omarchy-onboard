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
    /// Apply every operation of a proposal in order; stop at the first failure.
    /// Manual instructions are collected and returned, not executed.
    pub fn apply_all(&self, ops: &[Operation], files: &mut dyn FileSource) -> Result<Outcome> {
        if self.dry_run {
            return Ok(Outcome::Skipped("dry run".into()));
        }
        let mut manual = Vec::new();
        for op in ops {
            if let Outcome::Manual(text) = self.apply(op, files)? {
                manual.push(text);
            }
        }
        Ok(if manual.is_empty() {
            Outcome::Done
        } else {
            Outcome::Manual(manual.join("\n"))
        })
    }

    pub fn apply(&self, op: &Operation, files: &mut dyn FileSource) -> Result<Outcome> {
        match op {
            Operation::InstallPackages { packages } => {
                let pacman: Vec<&str> = packages
                    .iter()
                    .filter(|p| p.source == PackageSource::Pacman)
                    .map(|p| p.name.as_str())
                    .collect();
                let aur: Vec<&str> = packages
                    .iter()
                    .filter(|p| p.source == PackageSource::Aur)
                    .map(|p| p.name.as_str())
                    .collect();
                if !pacman.is_empty() {
                    run(
                        "sudo",
                        &[&["pacman", "-S", "--needed", "--noconfirm"][..], &pacman].concat(),
                    )?;
                }
                if !aur.is_empty() {
                    run(
                        "yay",
                        &[&["-S", "--needed", "--noconfirm"][..], &aur].concat(),
                    )?;
                }
                for p in packages
                    .iter()
                    .filter(|p| p.source == PackageSource::DistroInstaller)
                {
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
            Operation::PullFiles { items, dest, mode } => {
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
                    if let Some(mode) = mode {
                        set_mode(&target, *mode)?;
                    }
                }
                Ok(Outcome::Done)
            }
            Operation::WriteConfig {
                path,
                content,
                mode,
            } => {
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
    let status = Command::new(bin)
        .args(args)
        .status()
        .with_context(|| format!("running {bin}"))?;
    anyhow::ensure!(
        status.success(),
        "{bin} {} exited with {status}",
        args.join(" ")
    );
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    if path.is_dir() {
        for e in walkdir(path)? {
            std::fs::set_permissions(&e, std::fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            out.extend(walkdir(&p)?);
        }
        out.push(p);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use omarchy_onboard_core::FileKind;

    /// Copies from local disk; stands in for the paired client.
    struct LocalFiles;
    impl FileSource for LocalFiles {
        fn fetch(&mut self, item: &FileRef, dest: &Path) -> Result<()> {
            std::fs::copy(&item.path, dest)?;
            Ok(())
        }
    }

    #[test]
    fn write_config_append_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("rc");
        let op = Operation::WriteConfig {
            path: path.clone(),
            content: "export A=1\n".into(),
            mode: ConfigMode::Append,
        };
        let ex = Executor { dry_run: false };
        ex.apply(&op, &mut LocalFiles).unwrap();
        ex.apply(&op, &mut LocalFiles).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "export A=1\n");
    }

    #[test]
    fn pull_files_applies_mode() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("key");
        std::fs::write(&src, "secret").unwrap();
        let dest = dir.path().join("out").join("key");
        let op = Operation::PullFiles {
            items: vec![FileRef {
                path: src,
                kind: FileKind::File,
                size: 6,
            }],
            dest: dest.clone(),
            mode: Some(0o600),
        };
        Executor { dry_run: false }
            .apply(&op, &mut LocalFiles)
            .unwrap();
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "secret");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn dry_run_touches_nothing_and_manual_is_collected() {
        let ex = Executor { dry_run: true };
        let ops = [Operation::Manual {
            instructions: "do it".into(),
        }];
        assert!(matches!(
            ex.apply_all(&ops, &mut LocalFiles).unwrap(),
            Outcome::Skipped(_)
        ));
        let ex = Executor { dry_run: false };
        assert!(
            matches!(ex.apply_all(&ops, &mut LocalFiles).unwrap(), Outcome::Manual(m) if m == "do it")
        );
    }
}
