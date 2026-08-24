//! Platform-agnostic model for the three phases of a migration:
//!
//! 1. **Discover** — [`Check`]s run on the *source* machine and emit [`Finding`]s (facts).
//! 2. **Propose** — [`Rule`]s run on the *target* machine, turning findings into
//!    [`Proposal`]s whose [`Operation`] is the *semantically equivalent* thing to do
//!    (install the package, not copy its files).
//! 3. **Migrate** — an [`Executor`] applies accepted operations on the target.
//!
//! Nothing here knows about macOS, Windows, or Omarchy specifically; those live in
//! `omarchy-onboard-checks` (source side) and `omarchy-onboard-rules` (target side).

pub mod check;
pub mod discovery;
pub mod finding;
pub mod operation;
pub mod plan;
pub mod platform;
pub mod proposal;
pub mod rule;

pub use check::{Check, CheckMeta, SourceContext};
pub use discovery::Discovery;
pub use finding::{FileKind, FileRef, Finding};
pub use operation::{ConfigMode, Operation, Package, PackageSource};
pub use plan::{Decision, Plan};
pub use platform::{Group, Platform};
pub use proposal::Proposal;
pub use rule::{NoIndex, PackageIndex, Rule, TargetContext};
