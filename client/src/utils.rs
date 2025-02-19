use std::env;

use anyhow::{anyhow, Result};

use solana_sdk::signature::Keypair;

pub fn read_keypair_file(s: &str) -> Result<Keypair> {
    let expanded = if s.starts_with("~") {
        let home = env::var("HOME").map_err(|_| anyhow!("HOME environment variable is not set"))?;
        s.replacen("~", &home, 1)
    } else {
        s.to_string()
    };
    solana_sdk::signature::read_keypair_file(expanded)
        .map_err(|e| anyhow!("failed to read keypair from {}", e))
}
