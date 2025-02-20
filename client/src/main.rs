use std::str::FromStr;

use client::{self, config::Config};

use clap::{Parser, Subcommand};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};

/// Top-level struct for parsing command-line arguments.
///
/// The `Args` struct holds global options (like the configuration file)
/// and a subcommand which groups specific commands (e.g., Raydium or Solend).
#[derive(Debug, Parser)]
#[command(author, version, about = "CLI for managing Raydium and Solend operations", long_about = None)]
pub struct Args {
    /// Global configuration file path. This option allows you to specify
    /// a TOML file that contains configuration details.
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Choose a subcommand to execute. The subcommands are grouped into different
    /// categories such as `raydium` for Raydium-related commands and `solend` for Solend-related commands.
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

/// Subcommands grouping for the CLI.
///
/// You can extend this enum with additional groups as needed.
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    /// Raydium-related operations.
    #[command(subcommand, name = "raydium")]
    RaydiumSubcommands(RaydiumSubcommands),

    /// Solend-related operations.
    #[command(subcommand, name = "solend")]
    SolendSubcommands(SolendSubcommands),
}

/// Subcommands under the Raydium category.
///
/// Each variant represents a specific operation. The `--help` flag will
/// show descriptions for each command and its parameters.
#[derive(Debug, Subcommand)]
pub enum RaydiumSubcommands {
    /// Mint a new token.
    MintToken,

    /// Create a token account for the specified mint.
    CreateTokenAccount {
        /// The public key of the mint for which to create an account.
        mint: Pubkey,
    },

    /// Mint tokens to an existing token account.
    MintToTokenAccount {
        /// The public key of the mint.
        mint: Pubkey,
        /// The target token account's public key.
        token_account: Pubkey,
        /// The amount of tokens to mint.
        amount: u64,
    },

    /// Increase liquidity in a pool by specifying the price range and input amount.
    IncreaseLiquidity {
        /// Lower bound of the tick price.
        tick_lower_price: f64,
        /// Upper bound of the tick price.
        tick_upper_price: f64,
        /// Input amount used for liquidity.
        input_amount: u64,
        /// The public key of the liquidity pool.
        pool_pubkey: Pubkey,
        /// Allowed slippage when adding liquidity.
        slippage: f64,
    },

    /// Decrease liquidity from a pool by specifying the price range and liquidity.
    DecreaseLiquidity {
        /// Lower bound of the tick price.
        tick_lower_price: f64,
        /// Upper bound of the tick price.
        tick_upper_price: f64,
        /// The public key of the liquidity pool.
        pool_pubkey: Pubkey,
        /// Allowed slippage when removing liquidity.
        slippage: f64,
        /// Optional liquidity parameter to remove. If not provided, all liquidity is removed.
        liquidity: Option<u128>,
    },

    /// Create a new pool using the provided parameters.
    CreatePool {
        /// Configuration index for the pool.
        config_index: u16,
        /// Initial price for the pool.
        price: f64,
        /// The public key of the first token's mint.
        mint0: Pubkey,
        /// The public key of the second token's mint.
        mint1: Pubkey,
        /// Open time for the pool (optional, defaults to 0).
        #[arg(short, long, default_value_t = 0)]
        open_time: u64,
    },
}

/// Subcommands under the Solend category.
///
/// This enum can be extended as additional Solend operations become available.
#[derive(Debug, Subcommand)]
pub enum SolendSubcommands {
    /// A test command for Solend operations.
    Test,
}

/// The main entry point of the CLI application.
fn main() {
    // Parse the command line arguments using Clap.
    let args = Args::parse();

    // Load configuration from the specified config file.
    // This file should be in TOML format and contain the necessary settings.
    let config = client::config::Config::from_file("config.toml").unwrap();

    // Dispatch subcommands based on user input.
    match args.subcommand {
        Subcommands::RaydiumSubcommands(subcommand) => {
            process_raydium_subcommands(subcommand, &config);
        }
        Subcommands::SolendSubcommands(subcommand) => {
            process_solend_subcommands(subcommand, &config);
        }
    }
}

/// Processes Solend-specific subcommands.
fn process_solend_subcommands(subcommand: SolendSubcommands, config: &Config) {
    match subcommand {
        SolendSubcommands::Test => {
            // let solend = Pubkey::from_str("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx").unwrap();
            let lending_market =
                Pubkey::from_str("6xSkPYdpga1SgjZUemGNzDFkwbV68GSdySrP7XYShhE5").unwrap();

            let rpc = RpcClient::new(config.global.http_url.to_string());

            let account = rpc.get_account_data(&lending_market);

            let lending_market_account =
                solend_sdk::state::LendingMarket::unpack(&account.unwrap()).unwrap();

            println!("Lending Market: {:#?}", lending_market_account);
        }
    }
}

/// Processes Raydium-specific subcommands.
fn process_raydium_subcommands(subcommand: RaydiumSubcommands, config: &Config) {
    match subcommand {
        RaydiumSubcommands::MintToken => {
            // Create a new mint using the client module.
            let mint = client::create_mint(&config).unwrap();
            println!("Mint: {}", mint);
        }
        RaydiumSubcommands::CreateTokenAccount { mint } => {
            // Create a token account for the provided mint.
            let token_account = client::create_token_account(&config, &mint).unwrap();
            println!("Token Account: {}", token_account);
        }
        RaydiumSubcommands::MintToTokenAccount {
            mint,
            token_account,
            amount,
        } => {
            // Mint tokens to the specified token account.
            client::mint_to_token_account(&config, &mint, &token_account, amount).unwrap();
            println!("Minted {} tokens to account: {}", amount, token_account);
        }
        RaydiumSubcommands::IncreaseLiquidity {
            tick_lower_price,
            tick_upper_price,
            input_amount,
            pool_pubkey,
            slippage,
        } => {
            // Increase liquidity in the pool with the specified parameters.
            client::increase_liquidity(
                &config,
                tick_lower_price,
                tick_upper_price,
                true, // This example assumes an additional flag (e.g., for direction).
                input_amount,
                pool_pubkey,
                slippage,
            )
            .unwrap();
            println!("Increased liquidity in pool: {}", pool_pubkey);
        }
        RaydiumSubcommands::DecreaseLiquidity {
            tick_lower_price,
            tick_upper_price,
            liquidity,
            pool_pubkey,
            slippage,
        } => {
            // Decrease liquidity in the pool with the provided parameters.
            client::decrease_liquidity(
                &config,
                tick_lower_price,
                tick_upper_price,
                liquidity,
                pool_pubkey,
                slippage,
            )
            .unwrap();
            println!("Decreased liquidity in pool: {}", pool_pubkey);
        }
        RaydiumSubcommands::CreatePool {
            config_index,
            price,
            mint0,
            mint1,
            open_time,
        } => {
            // Create a new pool with the provided configuration and parameters.
            let pool =
                client::create_pool(&config, config_index, price, mint0, mint1, open_time).unwrap();
            println!("Pool created: {}", pool);
        }
    }
}
