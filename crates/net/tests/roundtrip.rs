//! Pair over real sockets on this machine, discover, pull a directory.
//! Needs a working local network stack (mDNS multicast); on macOS the spawning
//! app must have the Local Network permission.

use omarchy_onboard_core::{Discovery, FileKind, FileRef, Finding, Group, Platform};
use omarchy_onboard_net::protocol::TopicInfo;
use omarchy_onboard_net::server::Source;
use omarchy_onboard_net::{Client, PairingCode};
use omarchy_onboard_target::FileSource;
use std::sync::Arc;
use std::time::Duration;

struct FakeSource;
impl Source for FakeSource {
    fn host(&self) -> String {
        "fake".into()
    }
    fn platform(&self) -> Platform {
        Platform::MacOs
    }
    fn topics(&self) -> Vec<TopicInfo> {
        vec![TopicInfo {
            id: "t".into(),
            group: Group::Shell,
            title: "T".into(),
            description: "".into(),
        }]
    }
    fn discover(&self, _only: &[String]) -> anyhow::Result<Discovery> {
        let mut d = Discovery::new(Platform::MacOs, "fake");
        d.findings
            .push(Finding::new("t", Group::Shell, "k", "hello"));
        Ok(d)
    }
}

#[test]
#[ignore = "uses real sockets and mDNS; run with --ignored"]
fn pair_discover_and_pull() {
    let src = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(src.path().join("d/sub")).unwrap();
    std::fs::write(src.path().join("d/sub/a.txt"), "hello").unwrap();
    let dst = tempfile::tempdir().unwrap();

    let code = PairingCode::generate();
    let server_code = code.clone();
    let server = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(omarchy_onboard_net::serve(
            server_code,
            Arc::new(FakeSource),
        ))
        .unwrap();
    });

    let mut client = Client::pair(&code, Duration::from_secs(30)).expect("pair");
    assert_eq!(client.host, "fake");
    assert_eq!(client.list_topics().unwrap()[0].id, "t");
    assert_eq!(client.discover(&[]).unwrap().findings[0].title, "hello");

    let item = FileRef {
        path: src.path().join("d"),
        kind: FileKind::Directory,
        size: 5,
    };
    client.fetch(&item, &dst.path().join("d")).unwrap();
    assert_eq!(
        std::fs::read_to_string(dst.path().join("d/sub/a.txt")).unwrap(),
        "hello"
    );

    client.close();
    server.join().unwrap();
}
