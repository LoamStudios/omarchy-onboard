use anyhow::{Context, Result};
use iroh::endpoint::{RecvStream, SendStream};
use omarchy_onboard_core::{Discovery, FileRef, Group, Platform};
use serde::{Deserialize, Serialize};

const MAX_MSG: usize = 64 * 1024 * 1024;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Must be the first request on a connection.
    Hello {
        proof: Vec<u8>,
    },
    ListTopics,
    /// Run topics' discover; empty = all.
    Discover {
        only: Vec<String>,
    },
    /// Stream `item` as a tar archive after the `Ok` response.
    GetFile {
        item: FileRef,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Hello {
        host: String,
        platform: Platform,
    },
    Topics(Vec<TopicInfo>),
    Discovery(Box<Discovery>),
    /// For `GetFile`: tar bytes follow on the same stream.
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub id: String,
    pub group: Group,
    pub title: String,
    pub description: String,
}

pub async fn write_msg<T: Serialize>(send: &mut SendStream, msg: &T) -> Result<()> {
    let bytes = serde_json::to_vec(msg)?;
    send.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
    send.write_all(&bytes).await?;
    Ok(())
}

pub async fn read_msg<T: for<'de> Deserialize<'de>>(recv: &mut RecvStream) -> Result<T> {
    let mut len = [0u8; 4];
    recv.read_exact(&mut len)
        .await
        .context("reading message length")?;
    let len = u32::from_be_bytes(len) as usize;
    anyhow::ensure!(len <= MAX_MSG, "message too large ({len} bytes)");
    let mut buf = vec![0u8; len];
    recv.read_exact(&mut buf)
        .await
        .context("reading message body")?;
    Ok(serde_json::from_slice(&buf)?)
}
