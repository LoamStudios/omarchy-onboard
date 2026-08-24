//! Discover on macOS: `~/Library/Fonts` (user fonts only; `/Library/Fonts` and
//! `/System/Library/Fonts` ship with the OS). Files are grouped by the family
//! name in the font's own name table, falling back to the filename.

use super::{Family, ID};
use crate::fs::file_ref;
use anyhow::Result;
use omarchy_onboard_core::{FileRef, Finding, Group, SourceContext};
use std::collections::BTreeMap;
use std::path::Path;

const EXTS: &[&str] = &["ttf", "otf", "ttc"];

pub fn discover(ctx: &SourceContext) -> Result<Vec<Finding>> {
    let dir = ctx.home.join("Library/Fonts");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(vec![]);
    };
    let mut families: BTreeMap<String, Vec<FileRef>> = BTreeMap::new();
    for e in entries.filter_map(Result::ok) {
        let path = e.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        if !ext.map(|x| EXTS.contains(&x.as_str())).unwrap_or(false) {
            continue;
        }
        let Some(fr) = file_ref(&path) else { continue };
        families.entry(family_name(&path)).or_default().push(fr);
    }
    Ok(families
        .into_iter()
        .map(|(name, files)| {
            let mut f = Finding::new(
                ID,
                Group::Fonts,
                format!("family/{}", super::normalise(&name)),
                format!("{name} ({} files)", files.len()),
            )
            .with_value(Family {
                name: name.clone(),
                file_count: files.len(),
            });
            for fr in files {
                f = f.with_file(fr);
            }
            f
        })
        .collect())
}

/// Typographic family (name id 16) if present, else legacy family (id 1),
/// else a guess from the filename.
pub fn family_name(path: &Path) -> String {
    std::fs::read(path)
        .ok()
        .and_then(|data| {
            let face = ttf_parser::Face::parse(&data, 0).ok()?;
            let pick = |id: u16| {
                face.names()
                    .into_iter()
                    .find(|n| n.name_id == id && n.is_unicode())
                    .and_then(|n| n.to_string())
            };
            pick(ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
                .or_else(|| pick(ttf_parser::name_id::FAMILY))
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            family_of(
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown"),
            )
        })
}

/// Filename fallback: "BerkeleyMono-Bold-Oblique" → "Berkeley Mono".
pub fn family_of(stem: &str) -> String {
    let base = stem.split(['-', '_', '[']).next().unwrap_or(stem).trim();
    let mut out = String::new();
    let chars: Vec<char> = base.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && c.is_ascii_uppercase() && chars[i - 1].is_ascii_lowercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn filename_fallback_splits_family_names() {
        assert_eq!(
            super::family_of("BerkeleyMono-Bold-Oblique"),
            "Berkeley Mono"
        );
        assert_eq!(super::family_of("Geist[wght]"), "Geist");
    }
}
