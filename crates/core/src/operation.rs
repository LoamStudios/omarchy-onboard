use crate::finding::FileRef;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Something the target machine can do. Operations are declarative; the
/// target's `Executor` decides how (pacman vs yay, which config file, …).
///
/// Guiding principle: an operation expresses the *semantic* equivalent of
/// what the user had, never "copy these bytes" when a proper install exists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    /// Install packages via the target's package manager(s).
    InstallPackages { packages: Vec<Package> },

    /// Install an editor extension through the editor's own mechanism.
    InstallEditorExtension { editor: String, extension: String },

    /// Copy files/directories from the source. Only for genuinely user-owned
    /// data (documents, dotfiles) — not for anything installable.
    PullFiles {
        items: Vec<FileRef>,
        dest: PathBuf,
        /// Unix permission bits to apply to pulled files (e.g. `0o600` for secrets).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<u32>,
    },

    /// Write or merge a config file on the target.
    WriteConfig {
        path: PathBuf,
        content: String,
        mode: ConfigMode,
    },

    /// Apply a named target theme.
    SetTheme { name: String },

    /// Run a target-provided command (e.g. an `omarchy-*` helper).
    RunCommand { argv: Vec<String> },

    /// No automated equivalent — show the user what to do by hand.
    Manual { instructions: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigMode {
    /// Overwrite the file.
    Replace,
    /// Append if the content isn't already present.
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub source: PackageSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
    /// Official repos (`pacman`).
    Pacman,
    /// AUR (`yay`).
    Aur,
    /// A distro-provided installer script, e.g. `omarchy-install-*`.
    DistroInstaller,
}

impl Operation {
    /// Whether applying this needs files from the source machine.
    pub fn needs_source_files(&self) -> bool {
        matches!(self, Operation::PullFiles { .. })
    }

    pub fn is_manual(&self) -> bool {
        matches!(self, Operation::Manual { .. })
    }
}
