//! Discover on macOS via `defaults read -g` and `hidutil`.
//!
//! macOS units: `KeyRepeat` and `InitialKeyRepeat` are multiples of 15 ms.
//! A missing key means the system default (natural scroll on, repeat 6/25).

use super::{CapsLock, ID, InputSettings};
use anyhow::Result;
use omarchy_onboard_core::{Finding, Group};
use std::process::Command;

const TICK_MS: u32 = 15;
const CAPS_LOCK: u64 = 0x700000039;
const LEFT_CONTROL: u64 = 0x7000000E0;
const ESCAPE: u64 = 0x700000029;

pub fn discover() -> Result<Vec<Finding>> {
    let key_repeat = read_global("KeyRepeat").unwrap_or(6);
    let initial = read_global("InitialKeyRepeat").unwrap_or(25);
    let settings = InputSettings {
        repeat_delay_ms: Some(initial * TICK_MS),
        repeat_rate_hz: Some((1000 / (key_repeat.max(1) * TICK_MS)).max(1)),
        natural_scroll: Some(
            read_global("com.apple.swipescrolldirection")
                .map(|v| v != 0)
                .unwrap_or(true),
        ),
        tap_to_click: read_domain("com.apple.AppleMultitouchTrackpad", "Clicking").map(|v| v != 0),
        caps_lock: caps_lock_mapping(),
    };
    let mut parts = vec![format!(
        "key repeat {} ms / {}/s",
        settings.repeat_delay_ms.unwrap(),
        settings.repeat_rate_hz.unwrap()
    )];
    if settings.natural_scroll == Some(false) {
        parts.push("natural scroll off".into());
    }
    if settings.tap_to_click == Some(true) {
        parts.push("tap to click".into());
    }
    if let Some(c) = settings.caps_lock {
        parts.push(format!("Caps Lock → {c:?}"));
    }
    Ok(vec![
        Finding::new(ID, Group::Input, "settings", parts.join(", ")).with_value(settings),
    ])
}

fn read_global(key: &str) -> Option<u32> {
    read_domain("-g", key)
}

fn read_domain(domain: &str, key: &str) -> Option<u32> {
    let out = Command::new("defaults")
        .args(["read", domain, key])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .ok()
        .map(|v| v as u32)
}

fn caps_lock_mapping() -> Option<CapsLock> {
    let out = Command::new("hidutil")
        .args(["property", "--get", "UserKeyMapping"])
        .output()
        .ok()?;
    parse_hidutil(&String::from_utf8_lossy(&out.stdout))
}

/// Finds a `{ Dst = …; Src = caps }` entry in hidutil's plist-ish output.
fn parse_hidutil(s: &str) -> Option<CapsLock> {
    for entry in s.split('{') {
        if field(entry, "HIDKeyboardModifierMappingSrc") != Some(CAPS_LOCK) {
            continue;
        }
        return match field(entry, "HIDKeyboardModifierMappingDst")? {
            LEFT_CONTROL => Some(CapsLock::Control),
            ESCAPE => Some(CapsLock::Escape),
            _ => None,
        };
    }
    None
}

fn field(entry: &str, name: &str) -> Option<u64> {
    let i = entry.find(name)?;
    let rest = &entry[i + name.len()..];
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hidutil_caps_to_control() {
        let s = "{\n    UserKeyMapping =     (\n                {\n            HIDKeyboardModifierMappingDst = 30064771296;\n            HIDKeyboardModifierMappingSrc = 30064771129;\n        }\n    );\n}\n";
        assert_eq!(parse_hidutil(s), Some(CapsLock::Control));
        assert_eq!(parse_hidutil("(null)\n"), None);
    }
}
