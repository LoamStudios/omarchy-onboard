//! Platform-agnostic model for the three phases of a migration:
//!
//! 1. **Discover** — each [`Topic`] runs `discover` on the *source* machine and
//!    emits [`Finding`]s (facts).
//! 2. **Propose** — each topic runs `propose` on the *target* machine, turning
//!    findings into [`Proposal`]s composed of [`Operation`] primitives — the
//!    *semantically equivalent* thing (install the package, not copy its files).
//! 3. **Migrate** — an executor applies accepted operations on the target.
//!
//! A topic is the unit of authorship: one row that knows how to look on each
//! source platform and what to propose. Nothing here knows about macOS,
//! Windows, or Omarchy; topics live in `omarchy-onboard-topics`, executors in
//! `omarchy-onboard-target`.

pub mod context;
pub mod discovery;
pub mod finding;
pub mod operation;
pub mod plan;
pub mod platform;
pub mod proposal;
pub mod topic;

pub use context::{NoIndex, PackageIndex, SourceContext, TargetContext};
pub use discovery::Discovery;
pub use finding::{FileKind, FileRef, Finding};
pub use operation::{ConfigMode, Operation, Package, PackageSource};
pub use plan::{Decision, Plan};
pub use platform::{Group, Platform};
pub use proposal::Proposal;
pub use topic::{Topic, TopicMeta};
