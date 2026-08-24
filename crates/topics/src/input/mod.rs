//! Keyboard and pointer settings → Hyprland `input` block.

mod macos;

use anyhow::Result;
use omarchy_onboard_core::{
    ConfigMode, Decision, Discovery, Finding, Group, Operation, Platform, Proposal, SourceContext,
    TargetContext, Topic, TopicMeta,
};
use serde::{Deserialize, Serialize};

pub const ID: &str = "input";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsLock {
    Control,
    Escape,
}

/// Normalised, platform-neutral input settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputSettings {
    /// Delay before a held key starts repeating.
    pub repeat_delay_ms: Option<u32>,
    /// Repeats per second once repeating.
    pub repeat_rate_hz: Option<u32>,
    pub natural_scroll: Option<bool>,
    pub tap_to_click: Option<bool>,
    pub caps_lock: Option<CapsLock>,
}

pub struct Input;

static META: TopicMeta = TopicMeta {
    id: ID,
    group: Group::Input,
    title: "Keyboard & pointer",
    description: "Key repeat, scroll direction, tap to click, Caps Lock remap",
    sources: &[Platform::MacOs],
};

impl Topic for Input {
    fn meta(&self) -> &TopicMeta {
        &META
    }

    fn discover(&self, ctx: &SourceContext) -> Result<Vec<Finding>> {
        match ctx.platform {
            Platform::MacOs => macos::discover(),
            _ => Ok(vec![]),
        }
    }

    fn propose(&self, mine: &[&Finding], _all: &Discovery, ctx: &TargetContext) -> Vec<Proposal> {
        let Some(f) = mine.iter().find(|f| f.key == "settings") else {
            return vec![];
        };
        let Ok(s) = serde_json::from_value::<InputSettings>(f.value.clone()) else {
            return vec![];
        };
        let (block, lines) = hyprland_block(&s);
        if lines.is_empty() {
            return vec![];
        }
        vec![Proposal {
            id: "input/hyprland".into(),
            group: Group::Input,
            title: "Match keyboard & pointer settings in Hyprland".into(),
            rationale: format!(
                "Appends to ~/.config/hypr/input.conf: {}.",
                lines.join(", ")
            ),
            findings: vec![f.id()],
            operations: vec![Operation::WriteConfig {
                path: ctx.home.join(".config/hypr/input.conf"),
                content: block,
                mode: ConfigMode::Append,
            }],
            default: Decision::Accept,
        }]
    }
}

/// Render the settings as a Hyprland `input` block. Returns the block and a
/// human summary of each setting it sets.
fn hyprland_block(s: &InputSettings) -> (String, Vec<String>) {
    let mut lines = Vec::new();
    let mut body = Vec::new();
    let mut touchpad = Vec::new();
    if let Some(d) = s.repeat_delay_ms {
        body.push(format!("    repeat_delay = {d}"));
        lines.push(format!("repeat delay {d} ms"));
    }
    if let Some(r) = s.repeat_rate_hz {
        body.push(format!("    repeat_rate = {r}"));
        lines.push(format!("repeat rate {r}/s"));
    }
    if let Some(c) = s.caps_lock {
        let opt = match c {
            CapsLock::Control => "caps:ctrl_modifier",
            CapsLock::Escape => "caps:escape",
        };
        body.push(format!("    kb_options = {opt}"));
        lines.push(format!(
            "Caps Lock → {}",
            match c {
                CapsLock::Control => "Control",
                CapsLock::Escape => "Escape",
            }
        ));
    }
    if let Some(n) = s.natural_scroll {
        touchpad.push(format!("        natural_scroll = {n}"));
        lines.push(format!("natural scroll {}", if n { "on" } else { "off" }));
    }
    if let Some(t) = s.tap_to_click {
        touchpad.push(format!("        tap-to-click = {t}"));
        lines.push(format!("tap to click {}", if t { "on" } else { "off" }));
    }
    let mut out = String::from("\n# From omarchy-onboard (macOS settings)\ninput {\n");
    for l in &body {
        out.push_str(l);
        out.push('\n');
    }
    if !touchpad.is_empty() {
        out.push_str("    touchpad {\n");
        for l in &touchpad {
            out.push_str(l);
            out.push('\n');
        }
        out.push_str("    }\n");
    }
    out.push_str("}\n");
    (out, lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_hyprland_block() {
        let s = InputSettings {
            repeat_delay_ms: Some(150),
            repeat_rate_hz: Some(66),
            natural_scroll: Some(true),
            tap_to_click: None,
            caps_lock: Some(CapsLock::Escape),
        };
        let (block, lines) = hyprland_block(&s);
        assert!(block.contains("repeat_delay = 150"));
        assert!(block.contains("kb_options = caps:escape"));
        assert!(block.contains("natural_scroll = true"));
        assert!(!block.contains("tap-to-click"));
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn empty_settings_render_nothing() {
        let (_, lines) = hyprland_block(&InputSettings::default());
        assert!(lines.is_empty());
    }
}
