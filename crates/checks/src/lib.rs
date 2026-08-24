//! Source-side checks. Each check lives in its own module and is registered in
//! [`all`]. Checks are platform-gated by `CheckMeta::platforms`, not by
//! `cfg`, so the full catalogue is visible (and testable) everywhere; only
//! `run` needs the real OS.

use omarchy_onboard_core::{Check, Platform};

pub mod fs;
pub mod macos;

/// Every known check, on every platform.
pub fn all() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(macos::homebrew::Homebrew),
        Box::new(macos::shell::ShellDotfiles),
    ]
}

/// Checks that can run on `platform`.
pub fn for_platform(platform: Platform) -> Vec<Box<dyn Check>> {
    all().into_iter().filter(|c| c.supports(platform)).collect()
}
