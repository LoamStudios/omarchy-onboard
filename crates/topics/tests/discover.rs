//! Discover-side tests against a fabricated home directory.

use omarchy_onboard_core::{Platform, SourceContext, Topic};
use omarchy_onboard_topics::ssh;

#[test]
fn ssh_discover_finds_keys_config_and_never_leaks_key_material() {
    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".ssh");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(
        dir.join("id_ed25519"),
        "-----BEGIN OPENSSH PRIVATE KEY-----\nSECRETSECRET\n-----END OPENSSH PRIVATE KEY-----\n",
    )
    .unwrap();
    std::fs::write(dir.join("id_ed25519.pub"), "ssh-ed25519 AAAA me@host\n").unwrap();
    std::fs::write(dir.join("config"), "Host gh\n  HostName github.com\n").unwrap();
    std::fs::write(dir.join("known_hosts"), "github.com ssh-ed25519 AAAA\n").unwrap();
    std::fs::write(dir.join("notes.txt"), "not a key").unwrap();

    let ctx = SourceContext {
        platform: Platform::MacOs,
        home: home.path().to_path_buf(),
    };
    let findings = ssh::Ssh.discover(&ctx).unwrap();
    let mut keys: Vec<_> = findings.iter().map(|f| f.key.as_str()).collect();
    keys.sort();
    assert_eq!(keys, ["config", "key/id_ed25519", "known_hosts"]);

    let json = serde_json::to_string(&findings).unwrap();
    assert!(
        !json.contains("SECRETSECRET"),
        "private key bytes leaked into findings"
    );

    let key = findings.iter().find(|f| f.key == "key/id_ed25519").unwrap();
    assert_eq!(key.files.len(), 2);
    assert!(key.title.contains("ssh-ed25519"));
}
