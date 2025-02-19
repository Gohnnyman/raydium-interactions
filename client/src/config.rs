use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Global {
    pub http_url: String,
    pub ws_url: String,
    pub payer_path: String,
    pub admin_path: String,
    pub raydium_v3_program: String,
    pub slippage: f64,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub global: Global,
}

impl Config {
    /// Loads and parses the configuration from a TOML file at the given path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
