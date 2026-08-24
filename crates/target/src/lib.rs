//! Target-side "how": package index and executor for Omarchy (Arch).
//! Rules decide *what*; this crate knows pacman, yay and `omarchy-*` helpers.

pub mod executor;
pub mod pacman;

pub use executor::{Executor, FileSource, Outcome};
pub use pacman::{ListIndex, PacmanIndex};
