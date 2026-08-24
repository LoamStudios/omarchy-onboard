//! Propose-side tests: canned findings in, proposals out. No network, no real machine.

use omarchy_onboard_core::{
    Decision, Discovery, FileKind, FileRef, Finding, Group, Operation, PackageIndex, PackageSource,
    Platform, TargetContext,
};
use omarchy_onboard_topics::{homebrew, ssh};
use std::path::PathBuf;
use std::sync::Arc;

struct FakeIndex(Vec<&'static str>);
impl PackageIndex for FakeIndex {
    fn lookup(&self, name: &str) -> Option<PackageSource> {
        self.0.contains(&name).then_some(PackageSource::Pacman)
    }
}

fn ctx(index: FakeIndex) -> TargetContext {
    TargetContext {
        platform: Platform::Linux,
        home: PathBuf::from("/home/u"),
        packages: Arc::new(index),
    }
}

fn brew(name: &str, requested: bool, cask: bool) -> Finding {
    let key = format!("{}/{name}", if cask { "cask" } else { "formula" });
    Finding::new(
        homebrew::ID,
        if cask {
            Group::Applications
        } else {
            Group::Packages
        },
        key,
        name,
    )
    .with_value(homebrew::BrewPackage {
        name: name.into(),
        version: "1".into(),
        requested,
        cask,
    })
}

fn plan_for(findings: Vec<Finding>, index: FakeIndex) -> omarchy_onboard_core::Plan {
    let mut d = Discovery::new(Platform::MacOs, "mac");
    d.findings = findings;
    omarchy_onboard_topics::propose(&d, &ctx(index))
}

#[test]
fn homebrew_maps_known_skips_dependencies_and_verifies_unknown_against_index() {
    let plan = plan_for(
        vec![
            brew("ripgrep", true, false),   // mapped
            brew("libpng", false, false),   // dependency → nothing
            brew("hugo", true, false),      // mapped
            brew("caddy", true, false),     // unmapped, exists on target
            brew("mas", true, false),       // mapped as manual
            brew("weirdtool", true, false), // unmapped, absent → manual
            brew("slack", true, true),      // cask → AUR
        ],
        FakeIndex(vec!["caddy"]),
    );
    let by_id = |id: &str| {
        plan.proposals
            .iter()
            .find(|p| p.id == id)
            .unwrap_or_else(|| panic!("missing {id}"))
    };

    assert!(plan.proposals.iter().all(|p| p.id != "packages/libpng"));

    let rg = by_id("packages/ripgrep");
    assert_eq!(rg.default, Decision::Accept);
    assert!(
        matches!(&rg.operations[..], [Operation::InstallPackages { packages }] if packages[0].name == "ripgrep")
    );

    let caddy = by_id("packages/caddy");
    assert!(matches!(
        &caddy.operations[..],
        [Operation::InstallPackages { .. }]
    ));

    let mas = by_id("packages/mas");
    assert_eq!(mas.default, Decision::Skip);
    assert!(!mas.is_action());

    let weird = by_id("packages/weirdtool");
    assert!(!weird.is_action());

    let slack = by_id("apps/slack");
    assert_eq!(slack.group, Group::Applications);
    assert!(
        matches!(&slack.operations[..], [Operation::InstallPackages { packages }] if packages[0].source == PackageSource::Aur)
    );
}

#[test]
fn ssh_keys_are_pulled_with_0600_and_config_is_rewritten() {
    let key = Finding::new(ssh::ID, Group::Keys, "key/id_ed25519", "key")
        .with_file(FileRef {
            path: "/Users/u/.ssh/id_ed25519".into(),
            kind: FileKind::File,
            size: 400,
        })
        .with_file(FileRef {
            path: "/Users/u/.ssh/id_ed25519.pub".into(),
            kind: FileKind::File,
            size: 100,
        });
    let lone = Finding::new(ssh::ID, Group::Keys, "key/id_rsa", "key").with_file(FileRef {
        path: "/Users/u/.ssh/id_rsa".into(),
        kind: FileKind::File,
        size: 400,
    });
    let config =
        Finding::new(ssh::ID, Group::Keys, "config", "config").with_value(ssh::SshConfig {
            content: "Host *\n  UseKeychain yes\nHost gh\n  HostName github.com\n".into(),
            hosts: vec!["gh".into()],
        });
    let plan = plan_for(vec![key, lone, config], FakeIndex(vec![]));

    let pair = plan
        .proposals
        .iter()
        .find(|p| p.id == "ssh/key/id_ed25519")
        .unwrap();
    assert!(matches!(&pair.operations[..],
        [Operation::PullFiles { items, dest, mode: Some(0o600) }] if items.len() == 2 && dest == &PathBuf::from("/home/u/.ssh")));

    let lone = plan
        .proposals
        .iter()
        .find(|p| p.id == "ssh/key/id_rsa")
        .unwrap();
    assert!(matches!(&lone.operations[..],
        [Operation::PullFiles { dest, .. }] if dest == &PathBuf::from("/home/u/.ssh/id_rsa")));

    let cfg = plan
        .proposals
        .iter()
        .find(|p| p.id == "ssh/config")
        .unwrap();
    assert!(matches!(&cfg.operations[..],
        [Operation::WriteConfig { content, .. }] if !content.contains("UseKeychain") && content.contains("HostName github.com")));
    assert!(cfg.rationale.contains("usekeychain"));
}

#[test]
fn every_topic_has_a_unique_id_and_at_least_one_source() {
    let topics = omarchy_onboard_topics::all();
    let mut ids: Vec<_> = topics.iter().map(|t| t.meta().id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), topics.len(), "duplicate topic ids");
    assert!(topics.iter().all(|t| !t.meta().sources.is_empty()));
}

#[test]
fn vscode_extensions_are_installed_not_copied_and_keybindings_rewritten() {
    use omarchy_onboard_topics::vscode;
    let ext = |id: &str| {
        Finding::new(vscode::ID, Group::Editors, format!("extension/{id}"), id).with_value(
            vscode::Extension {
                id: id.into(),
                version: None,
            },
        )
    };
    let kb = Finding::new(vscode::ID, Group::Editors, "keybindings", "kb").with_value(
        vscode::ConfigFile {
            content: "[{\"key\": \"cmd+p\"}]".into(),
        },
    );
    let plan = plan_for(vec![ext("a.b"), ext("c.d"), kb], FakeIndex(vec![]));

    let exts = plan
        .proposals
        .iter()
        .find(|p| p.id == "vscode/extensions")
        .unwrap();
    assert_eq!(exts.operations.len(), 2);
    assert!(exts.operations.iter().all(
        |o| matches!(o, Operation::InstallEditorExtension { editor, .. } if editor == "vscode")
    ));
    assert_eq!(exts.findings.len(), 2);

    let kb = plan
        .proposals
        .iter()
        .find(|p| p.id == "vscode/keybindings")
        .unwrap();
    assert!(
        matches!(&kb.operations[..], [Operation::WriteConfig { content, .. }] if content.contains("ctrl+p"))
    );
}

#[test]
fn input_settings_become_one_hyprland_append() {
    use omarchy_onboard_topics::input;
    let f =
        Finding::new(input::ID, Group::Input, "settings", "s").with_value(input::InputSettings {
            repeat_delay_ms: Some(150),
            repeat_rate_hz: Some(66),
            natural_scroll: Some(false),
            tap_to_click: Some(true),
            caps_lock: Some(input::CapsLock::Control),
        });
    let plan = plan_for(vec![f], FakeIndex(vec![]));
    assert_eq!(plan.proposals.len(), 1);
    let p = &plan.proposals[0];
    assert!(matches!(&p.operations[..],
        [Operation::WriteConfig { path, content, mode: omarchy_onboard_core::ConfigMode::Append }]
            if path.ends_with(".config/hypr/input.conf") && content.contains("caps:ctrl_modifier") && content.contains("natural_scroll = false")));
}
