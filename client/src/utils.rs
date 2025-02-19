use std::env;

use anyhow::{anyhow, Result};

use solana_sdk::signature::Keypair;

const Q64: u128 = (u64::MAX as u128) + 1; // 2^64

pub fn multipler(decimals: u8) -> f64 {
    (10_i32).checked_pow(decimals.try_into().unwrap()).unwrap() as f64
}

pub fn price_to_x64(price: f64) -> u128 {
    (price * Q64 as f64) as u128
}

pub fn price_to_sqrt_price_x64(price: f64, decimals_0: u8, decimals_1: u8) -> u128 {
    let price_with_decimals = price * multipler(decimals_1) / multipler(decimals_0);
    price_to_x64(price_with_decimals.sqrt())
}

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
