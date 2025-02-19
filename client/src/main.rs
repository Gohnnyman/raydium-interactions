use client;

use clap::Parser;

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
        mint: String,
    },
    MintToTokenAccount {
        mint: String,
        token_account: String,
        amount: u64,
    },
    Test,
}

fn main() {
    let args = Args::parse();

    let config = client::config::Config::from_file("config.toml").unwrap();

    println!("CONFIG: {:#?}", config);

    match args.command {
        CommandsName::MintToken => {
            let mint = client::create_mint(&config).unwrap();

            println!("Mint: {}", mint);
        }
        CommandsName::CreateTokenAccount { mint } => {
            let mint = mint.parse().unwrap();
            let token_account = client::create_token_account(&config, &mint).unwrap();

            println!("Token Account: {}", token_account);
        }
        CommandsName::MintToTokenAccount {
            mint,
            token_account,
            amount,
        } => {
            let mint = mint.parse().unwrap();
            let token_account = token_account.parse().unwrap();

            client::mint_to_token_account(&config, &mint, &token_account, amount).unwrap();
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
