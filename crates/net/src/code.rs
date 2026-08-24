use iroh::endpoint::Connection;
use sha2::{Digest, Sha256};
use std::fmt;

/// Unambiguous alphabet (no 0/O, 1/I/L).
const ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";
const LEN: usize = 8;

/// Short code shown by the source and typed on the target, e.g. `K7QT-3MZP`.
#[derive(Clone, PartialEq, Eq)]
pub struct PairingCode(String);

impl PairingCode {
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let s: String = (0..LEN)
            .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
            .collect();
        Self(s)
    }

    /// Accepts any case, with or without the dash.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let s: String = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        anyhow::ensure!(s.len() == LEN, "pairing code must be {LEN} characters");
        anyhow::ensure!(
            s.bytes().all(|b| ALPHABET.contains(&b)),
            "pairing code has invalid characters"
        );
        Ok(Self(s))
    }

    /// Public tag broadcast over mDNS so the target can find the right source.
    pub fn discovery_tag(&self) -> String {
        let h = Sha256::new_with_prefix(b"omarchy-onboard/tag/")
            .chain_update(self.0.as_bytes())
            .finalize();
        format!("oo1-{}", hex(&h[..8]))
    }

    /// Session-bound proof of knowledge of the code.
    pub fn proof(&self, conn: &Connection) -> anyhow::Result<Vec<u8>> {
        let mut ekm = [0u8; 32];
        conn.export_keying_material(&mut ekm, b"omarchy-onboard/pair", &[])
            .map_err(|e| anyhow::anyhow!("export keying material: {e:?}"))?;
        Ok(Sha256::new_with_prefix(b"omarchy-onboard/proof/")
            .chain_update(self.0.as_bytes())
            .chain_update(ekm)
            .finalize()
            .to_vec())
    }
}

impl fmt::Display for PairingCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", &self.0[..4], &self.0[4..])
    }
}

impl fmt::Debug for PairingCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PairingCode(****)")
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
