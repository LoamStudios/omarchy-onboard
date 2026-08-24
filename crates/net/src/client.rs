use crate::code::PairingCode;
use crate::protocol::{CheckInfo, Request, Response, read_msg, write_msg};
use crate::ALPN;
use anyhow::{Context, Result};
use iroh::endpoint::{Connection, presets};
use iroh::{Endpoint, RelayMode};
use iroh_mdns_address_lookup::{DiscoveryEvent, MdnsAddressLookup};
use n0_future::StreamExt;
use omarchy_onboard_core::{Discovery, FileKind, FileRef, Platform};
use omarchy_onboard_target::FileSource;
use std::path::Path;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Synchronous handle to a paired source. Owns its own tokio runtime so the
/// CLI and executor stay plain blocking code.
pub struct Client {
    rt: Runtime,
    endpoint: Endpoint,
    conn: Connection,
    pub host: String,
    pub platform: Platform,
}

impl Client {
    /// Scan the LAN for a source advertising `code`, connect, and prove the code.
    pub fn pair(code: &PairingCode, timeout: Duration) -> Result<Self> {
        let rt = Runtime::new()?;
        let (endpoint, conn, host, platform) = rt.block_on(pair_async(code, timeout))?;
        Ok(Self { rt, endpoint, conn, host, platform })
    }

    pub fn list_checks(&self) -> Result<Vec<CheckInfo>> {
        match self.call(Request::ListChecks)? {
            Response::Checks(c) => Ok(c),
            other => anyhow::bail!("unexpected response: {other:?}"),
        }
    }

    pub fn discover(&self, only: &[String]) -> Result<Discovery> {
        match self.call(Request::RunChecks { only: only.to_vec() })? {
            Response::Discovery(d) => Ok(*d),
            other => anyhow::bail!("unexpected response: {other:?}"),
        }
    }

    fn call(&self, req: Request) -> Result<Response> {
        self.rt.block_on(async {
            let (mut send, mut recv) = self.conn.open_bi().await?;
            write_msg(&mut send, &req).await?;
            send.finish()?;
            let resp: Response = read_msg(&mut recv).await?;
            if let Response::Error(e) = &resp {
                anyhow::bail!("source error: {e}");
            }
            Ok(resp)
        })
    }

    pub fn close(self) {
        self.rt.block_on(async {
            self.conn.close(0u32.into(), b"done");
            self.endpoint.close().await;
        });
    }
}

impl FileSource for Client {
    fn fetch(&mut self, item: &FileRef, dest: &Path) -> Result<()> {
        self.rt.block_on(async {
            let (mut send, mut recv) = self.conn.open_bi().await?;
            write_msg(&mut send, &Request::GetFile { item: item.clone() }).await?;
            send.finish()?;
            match read_msg::<Response>(&mut recv).await? {
                Response::Ok => {}
                Response::Error(e) => anyhow::bail!("source error: {e}"),
                other => anyhow::bail!("unexpected response: {other:?}"),
            }
            // Pump the tar stream to a blocking unpacker.
            let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(16);
            let dest = dest.to_path_buf();
            let kind = item.kind;
            let unpack = tokio::task::spawn_blocking(move || -> Result<()> {
                let reader = ChannelReader { rx, buf: Vec::new(), pos: 0 };
                let mut ar = tar::Archive::new(reader);
                // The archive has one top-level entry named after the source's
                // file name; unpack it *as* `dest`.
                let staging = tempdir_beside(&dest)?;
                ar.unpack(&staging)?;
                let mut entries = std::fs::read_dir(&staging)?;
                let first = entries.next().context("empty archive")??.path();
                if dest.exists() {
                    match kind {
                        FileKind::File => std::fs::remove_file(&dest)?,
                        FileKind::Directory => std::fs::remove_dir_all(&dest)?,
                    }
                }
                std::fs::rename(&first, &dest)?;
                std::fs::remove_dir_all(&staging)?;
                Ok(())
            });
            let mut buf = vec![0u8; 64 * 1024];
            while let Some(n) = recv.read(&mut buf).await? {
                if tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
            drop(tx);
            unpack.await??;
            Ok(())
        })
    }
}

fn tempdir_beside(dest: &Path) -> Result<std::path::PathBuf> {
    let parent = dest.parent().context("dest has no parent")?;
    std::fs::create_dir_all(parent)?;
    let name = format!(".omarchy-onboard-{}", std::process::id());
    let p = parent.join(name);
    std::fs::create_dir_all(&p)?;
    Ok(p)
}

struct ChannelReader {
    rx: std::sync::mpsc::Receiver<Vec<u8>>,
    buf: Vec<u8>,
    pos: usize,
}

impl std::io::Read for ChannelReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.buf.len() {
            match self.rx.recv() {
                Ok(chunk) => {
                    self.buf = chunk;
                    self.pos = 0;
                }
                Err(_) => return Ok(0),
            }
        }
        let n = (self.buf.len() - self.pos).min(out.len());
        out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

async fn pair_async(code: &PairingCode, timeout: Duration) -> Result<(Endpoint, Connection, String, Platform)> {
    let endpoint = Endpoint::builder(presets::Minimal)
        .relay_mode(RelayMode::Disabled)
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .context("binding endpoint")?;
    let mdns = MdnsAddressLookup::builder().advertise(false).build(endpoint.id())?;
    endpoint.address_lookup()?.add(mdns.clone());

    match pair_on(&endpoint, &mdns, code, timeout).await {
        Ok((conn, host, platform)) => Ok((endpoint, conn, host, platform)),
        Err(e) => {
            endpoint.close().await;
            Err(e)
        }
    }
}

async fn pair_on(
    endpoint: &Endpoint,
    mdns: &MdnsAddressLookup,
    code: &PairingCode,
    timeout: Duration,
) -> Result<(Connection, String, Platform)> {
    let tag = code.discovery_tag();
    let mut events = mdns.subscribe().await;
    let found = tokio::time::timeout(timeout, async {
        while let Some(ev) = events.next().await {
            if let DiscoveryEvent::Discovered { endpoint_info, .. } = ev
                && endpoint_info.data.user_data().map(|u| u.as_ref() == tag.as_str()).unwrap_or(false)
            {
                return Some(endpoint_info);
            }
        }
        None
    })
    .await
    .map_err(|_| anyhow::anyhow!("no source with that pairing code found on the local network within {}s", timeout.as_secs()))?
    .context("discovery ended")?;

    tracing::info!(id = %found.endpoint_id, "found source");
    let conn = endpoint.connect(found.into_endpoint_addr(), ALPN).await.context("connecting to source")?;

    let proof = code.proof(&conn)?;
    let (mut send, mut recv) = conn.open_bi().await?;
    write_msg(&mut send, &Request::Hello { proof }).await?;
    send.finish()?;
    match read_msg::<Response>(&mut recv).await? {
        Response::Hello { host, platform } => Ok((conn, host, platform)),
        Response::Error(e) => anyhow::bail!("pairing rejected: {e}"),
        other => anyhow::bail!("unexpected response: {other:?}"),
    }
}
