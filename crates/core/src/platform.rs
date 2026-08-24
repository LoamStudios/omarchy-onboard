use serde::{Deserialize, Serialize};
use std::fmt;

/// Operating system a check can run on (source) or an executor targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    MacOs,
    Windows,
    Linux,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

/// User-facing grouping. Findings and proposals carry a group so the UI can
/// present "Packages (12)", "Editors (3)", and accept/skip a whole group at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Shell,
    Packages,
    Applications,
    Editors,
    Terminal,
    Git,
    Keys,
    Input,
    Appearance,
    Fonts,
    Files,
}

impl Group {
    pub const ALL: &[Group] = &[
        Group::Shell,
        Group::Packages,
        Group::Applications,
        Group::Editors,
        Group::Terminal,
        Group::Git,
        Group::Keys,
        Group::Input,
        Group::Appearance,
        Group::Fonts,
        Group::Files,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Group::Shell => "Shell",
            Group::Packages => "Packages",
            Group::Applications => "Applications",
            Group::Editors => "Editors",
            Group::Terminal => "Terminal",
            Group::Git => "Git",
            Group::Keys => "SSH & keys",
            Group::Input => "Keyboard & input",
            Group::Appearance => "Appearance & theme",
            Group::Fonts => "Fonts",
            Group::Files => "Files & directories",
        }
    }
}

impl fmt::Display for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}
