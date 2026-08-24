//! Pairing and transport between source (`serve`) and target (`Client`).
//!
//! - **Discovery**: the source advertises over mDNS with a user-data tag derived
//!   from the pairing code; the target scans for that tag. Nothing secret is
//!   broadcast.
//! - **Authentication**: iroh gives an encrypted QUIC connection between two
//!   keypairs. The target proves it knows the code by sending
//!   `SHA256(code ‖ EKM)` where EKM is TLS exported keying material for this
//!   connection (RFC 5705) — so the proof is bound to this session and useless
//!   to a MITM.
//! - **Protocol**: one bidirectional stream per request; length-prefixed JSON.
//!   `GetFile` is followed by a raw tar stream.

pub mod client;
pub mod code;
pub mod protocol;
pub mod server;

pub use client::Client;
pub use code::PairingCode;
pub use server::serve;

pub const ALPN: &[u8] = b"omarchy-onboard/0";
