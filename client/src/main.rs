use client;

use clap::Parser;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Parser)]

pub struct Args {
    #[clap(subcommand)]
    pub command: CommandsName,

    #[clap(short, long, default_value = "config.toml")]
    pub config: String,
}

#[derive(Debug, Parser)]
pub enum CommandsName {
    MintToken,
    CreateTokenAccount {
        mint: Pubkey,
    },
    MintToTokenAccount {
        mint: Pubkey,
        token_account: Pubkey,
        amount: u64,
    },
    IncreaseLiquidity {
        tick_lower_price: f64,
        tick_upper_price: f64,
        input_amount: u64,
        pool_pubkey: Pubkey,
        slippage: f64,
    },
    CreatePool {
        config_index: u16,
        price: f64,
        mint0: Pubkey,
        mint1: Pubkey,
        #[arg(short, long, default_value_t = 0)]
        open_time: u64,
    },
    Test,
}

fn main() {
    let args = Args::parse();

    let config = client::config::Config::from_file("config.toml").unwrap();

    match args.command {
        CommandsName::MintToken => {
            let mint = client::create_mint(&config).unwrap();

            println!("Mint: {}", mint);
        }
        CommandsName::CreateTokenAccount { mint } => {
            let token_account = client::create_token_account(&config, &mint).unwrap();

            println!("Token Account: {}", token_account);
        }
        CommandsName::MintToTokenAccount {
            mint,
            token_account,
            amount,
        } => {
            client::mint_to_token_account(&config, &mint, &token_account, amount).unwrap();
        }
        CommandsName::IncreaseLiquidity {
            tick_lower_price,
            tick_upper_price,
            input_amount,
            pool_pubkey,
            slippage,
        } => {
            client::increase_liquidity(
                &config,
                tick_lower_price,
                tick_upper_price,
                true,
                input_amount,
                pool_pubkey,
                slippage,
            )
            .unwrap();
        }
        CommandsName::CreatePool {
            config_index,
            price,
            mint0,
            mint1,
            open_time,
        } => {
            let pool =
                client::create_pool(&config, config_index, price, mint0, mint1, open_time).unwrap();

            println!("Pool: {}", pool);
        }
        CommandsName::Test => {
            let mint1 = client::create_mint(&config).unwrap();
            let token_account1 = client::create_token_account(&config, &mint1).unwrap();

            let mint2 = client::create_mint(&config).unwrap();
            let token_account2 = client::create_token_account(&config, &mint2).unwrap();

            println!("Mint1: {}", mint1);
            println!("Token Account1: {}", token_account1);

            println!("Mint2: {}", mint2);
            println!("Token Account2: {}", token_account2);

            client::mint_to_token_account(&config, &mint1, &token_account1, 100_000).unwrap();
            client::mint_to_token_account(&config, &mint2, &token_account2, 100_000).unwrap();
        }
    }
}
