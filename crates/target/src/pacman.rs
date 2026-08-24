use omarchy_onboard_core::{PackageIndex, PackageSource};
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

/// Live index: official repos via `pacman -Slq` (one shot), AUR via `yay -Si`
/// per name (cached).
pub struct PacmanIndex {
    repo: HashSet<String>,
    aur_cache: Mutex<std::collections::HashMap<String, bool>>,
    have_yay: bool,
}

impl PacmanIndex {
    pub fn load() -> anyhow::Result<Self> {
        let out = Command::new("pacman").args(["-Slq"]).output()?;
        anyhow::ensure!(out.status.success(), "pacman -Slq failed");
        let repo = String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        let have_yay = Command::new("yay")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        Ok(Self {
            repo,
            aur_cache: Mutex::new(Default::default()),
            have_yay,
        })
    }

    fn in_aur(&self, name: &str) -> bool {
        if !self.have_yay {
            return false;
        }
        if let Some(&v) = self.aur_cache.lock().unwrap().get(name) {
            return v;
        }
        let ok = Command::new("yay")
            .args(["-Si", "--aur", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        self.aur_cache.lock().unwrap().insert(name.to_string(), ok);
        ok
    }
}

impl PackageIndex for PacmanIndex {
    fn lookup(&self, name: &str) -> Option<PackageSource> {
        if self.repo.contains(name) {
            Some(PackageSource::Pacman)
        } else if self.in_aur(name) {
            Some(PackageSource::Aur)
        } else {
            None
        }
    }
}

/// Offline index from a newline-separated list (e.g. `pacman -Slq > pkgs.txt`).
/// Lets you develop and test planning on a non-Arch machine.
pub struct ListIndex(HashSet<String>);

impl ListIndex {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(Self(
            s.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect(),
        ))
    }
}

impl PackageIndex for ListIndex {
    fn lookup(&self, name: &str) -> Option<PackageSource> {
        self.0.contains(name).then_some(PackageSource::Pacman)
    }
}
