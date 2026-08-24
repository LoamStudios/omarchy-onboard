//! Helpers shared by checks that reference files.

use omarchy_migrate_core::{FileKind, FileRef};
use std::path::Path;

/// Build a `FileRef` for `path` if it exists, computing recursive size for dirs.
pub fn file_ref(path: &Path) -> Option<FileRef> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if meta.is_dir() {
        let size = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum();
        Some(FileRef { path: path.to_path_buf(), kind: FileKind::Directory, size })
    } else {
        Some(FileRef { path: path.to_path_buf(), kind: FileKind::File, size: meta.len() })
    }
}
