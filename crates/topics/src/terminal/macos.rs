//! Discover on macOS: read each emulator's config from its macOS location(s)
//! and pull out the font family/size with a light per-format parse.

use super::{Emulator, ID, TerminalConfig};
use anyhow::Result;
use omarchy_onboard_core::{Finding, Group, SourceContext};

pub fn discover(ctx: &SourceContext) -> Result<Vec<Finding>> {
    let h = &ctx.home;
    let candidates = [
        (
            Emulator::Ghostty,
            vec![
                h.join(".config/ghostty/config"),
                h.join("Library/Application Support/com.mitchellh.ghostty/config"),
            ],
        ),
        (
            Emulator::Alacritty,
            vec![
                h.join(".config/alacritty/alacritty.toml"),
                h.join(".alacritty.toml"),
            ],
        ),
        (Emulator::Kitty, vec![h.join(".config/kitty/kitty.conf")]),
        (
            Emulator::WezTerm,
            vec![
                h.join(".config/wezterm/wezterm.lua"),
                h.join(".wezterm.lua"),
            ],
        ),
    ];
    let mut out = Vec::new();
    for (emu, paths) in candidates {
        let Some(content) = paths.iter().find_map(|p| std::fs::read_to_string(p).ok()) else {
            continue;
        };
        let (font_family, font_size) = parse_font(emu, &content);
        let title = format!(
            "{} config{}",
            emu.name(),
            font_family
                .as_deref()
                .map(|f| format!(
                    " — {f}{}",
                    font_size.map(|s| format!(" {s}")).unwrap_or_default()
                ))
                .unwrap_or_default()
        );
        out.push(
            Finding::new(
                ID,
                Group::Terminal,
                format!("{:?}", emu).to_lowercase(),
                title,
            )
            .with_value(TerminalConfig {
                emulator: emu,
                content,
                font_family,
                font_size,
            }),
        );
    }
    if h.join("Library/Preferences/com.googlecode.iterm2.plist")
        .exists()
    {
        out.push(
            Finding::new(
                ID,
                Group::Terminal,
                "iterm2",
                "iTerm2 preferences (macOS only)",
            )
            .with_value(TerminalConfig {
                emulator: Emulator::ITerm2,
                content: String::new(),
                font_family: None,
                font_size: None,
            }),
        );
    }
    Ok(out)
}

/// Best-effort: enough to name the font, not a full config parser.
pub fn parse_font(emu: Emulator, content: &str) -> (Option<String>, Option<f32>) {
    let mut family = None;
    let mut size = None;
    for line in content.lines() {
        let t = line.trim();
        match emu {
            Emulator::Ghostty => {
                if let Some(v) = kv(t, "font-family") {
                    family.get_or_insert(v);
                } else if let Some(v) = kv(t, "font-size") {
                    size = v.parse().ok();
                }
            }
            Emulator::Kitty => {
                if let Some(v) = t.strip_prefix("font_family ") {
                    family = Some(v.trim().to_string());
                } else if let Some(v) = t.strip_prefix("font_size ") {
                    size = v.trim().parse().ok();
                }
            }
            Emulator::Alacritty => {
                if let Some(v) = kv(t, "family") {
                    family.get_or_insert(v);
                } else if let Some(v) = kv(t, "size") {
                    size = v.parse().ok();
                }
            }
            Emulator::WezTerm => {
                if let Some(i) = t.find("wezterm.font(") {
                    let rest = &t[i + 13..];
                    let q: Vec<&str> = rest.split(['"', '\'']).collect();
                    if q.len() > 1 {
                        family = Some(q[1].to_string());
                    }
                } else if let Some(v) = kv(t, "config.font_size") {
                    size = v.trim_end_matches(',').parse().ok();
                }
            }
            Emulator::ITerm2 => {}
        }
    }
    (family, size)
}

/// `key = "value"` / `key = value` → value without quotes.
fn kv(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    Some(rest.trim_matches(|c| c == '"' || c == '\'').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ghostty_font() {
        let (f, s) = parse_font(
            Emulator::Ghostty,
            "theme = bamboo\nfont-family = \"Berkeley Mono\"\nfont-size = 18\n",
        );
        assert_eq!(f.as_deref(), Some("Berkeley Mono"));
        assert_eq!(s, Some(18.0));
    }

    #[test]
    fn parses_wezterm_font() {
        let (f, s) = parse_font(
            Emulator::WezTerm,
            "config.font = wezterm.font('Fira Code')\nconfig.font_size = 14.5,\n",
        );
        assert_eq!(f.as_deref(), Some("Fira Code"));
        assert_eq!(s, Some(14.5));
    }
}
