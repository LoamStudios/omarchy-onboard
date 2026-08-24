use crate::code::PairingCode;
use crate::protocol::{CheckInfo, Request, Response, read_msg, write_msg};
use crate::ALPN;
use anyhow::{Context, Result};
use iroh::endpoint::{Connection, presets};
use iroh::{Endpoint, RelayMode};
use iroh_mdns_address_lookup::MdnsAddressLookup;
use omarchy_onboard_core::{Discovery, FileKind, FileRef};
use std::sync::Arc;

/// Runs checks on this machine. Injected so `net` doesn't depend on `checks`.
pub trait Source: Send + Sync + 'static {
    fn host(&self) -> String;
    fn platform(&self) -> omarchy_onboard_core::Platform;
    fn checks(&self) -> Vec<CheckInfo>;
    fn discover(&self, only: &[String]) -> Result<Discovery>;
}

/// Advertise on the LAN and answer one paired target at a time. Returns when
/// the paired target disconnects.
pub async fn serve(code: PairingCode, source: Arc<dyn Source>) -> Result<()> {
    let user_data: iroh::address_lookup::UserData = code.discovery_tag().parse().map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let endpoint = Endpoint::builder(presets::Minimal)
        .relay_mode(RelayMode::Disabled)
        .alpns(vec![ALPN.to_vec()])
        .user_data_for_address_lookup(user_data)
        .address_lookup(MdnsAddressLookup::builder().advertise(true))
        .bind()
        .await
        .context("binding endpoint")?;
    tracing::info!(id = %endpoint.id(), "serving");

    loop {
        let Some(incoming) = endpoint.accept().await else { break };
        let conn = match incoming.await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("incoming connection failed: {e}");
                continue;
            }
        };
        match handle_connection(conn, &code, source.clone()).await {
            Ok(true) => break,
            Ok(false) => continue,
            Err(e) => tracing::warn!("connection error: {e:#}"),
        }
    }
    endpoint.close().await;
    Ok(())
}

/// Returns `Ok(true)` once a paired session completes.
async fn handle_connection(conn: Connection, code: &PairingCode, source: Arc<dyn Source>) -> Result<bool> {
    // First stream must carry a valid Hello.
    let (mut send, mut recv) = conn.accept_bi().await?;
    let Request::Hello { proof } = read_msg::<Request>(&mut recv).await? else {
        write_msg(&mut send, &Response::Error("expected Hello".into())).await?;
        return Ok(false);
    };
    if proof != code.proof(&conn)? {
        tracing::warn!(remote = %conn.remote_id(), "rejected: wrong pairing code");
        write_msg(&mut send, &Response::Error("wrong pairing code".into())).await?;
        send.finish()?;
        conn.close(1u32.into(), b"bad code");
        return Ok(false);
    }
    write_msg(&mut send, &Response::Hello { host: source.host(), platform: source.platform() }).await?;
    send.finish()?;
    eprintln!("Paired with {}", conn.remote_id().fmt_short());

    loop {
        let (mut send, mut recv) = match conn.accept_bi().await {
            Ok(s) => s,
            Err(_) => return Ok(true), // remote closed
        };
        let req: Request = read_msg(&mut recv).await?;
        let result = handle_request(req, &source, &mut send).await;
        if let Err(e) = result {
            tracing::warn!("request failed: {e:#}");
            let _ = write_msg(&mut send, &Response::Error(format!("{e:#}"))).await;
        }
        let _ = send.finish();
    }
}

async fn handle_request(req: Request, source: &Arc<dyn Source>, send: &mut iroh::endpoint::SendStream) -> Result<()> {
    match req {
        Request::Hello { .. } => anyhow::bail!("duplicate Hello"),
        Request::ListChecks => write_msg(send, &Response::Checks(source.checks())).await,
        Request::RunChecks { only } => {
            let src = source.clone();
            let d = tokio::task::spawn_blocking(move || src.discover(&only)).await??;
            write_msg(send, &Response::Discovery(Box::new(d))).await
        }
        Request::GetFile { item } => {
            write_msg(send, &Response::Ok).await?;
            stream_tar(item, send).await
        }
    }
}

/// Tar `item` on a blocking thread, forwarding chunks to the QUIC stream.
async fn stream_tar(item: FileRef, send: &mut iroh::endpoint::SendStream) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(16);
    let producer = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut ar = tar::Builder::new(ChannelWriter(tx));
        ar.follow_symlinks(false);
        let name = item.path.file_name().context("file has no name")?;
        match item.kind {
            FileKind::File => ar.append_path_with_name(&item.path, name)?,
            FileKind::Directory => ar.append_dir_all(name, &item.path)?,
        }
        ar.finish()?;
        Ok(())
    });
    while let Some(chunk) = rx.recv().await {
        send.write_all(&chunk).await?;
    }
    producer.await??;
    Ok(())
}

struct ChannelWriter(tokio::sync::mpsc::Sender<Vec<u8>>);

impl std::io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.blocking_send(buf.to_vec()).map_err(|_| std::io::Error::other("receiver gone"))?;
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
