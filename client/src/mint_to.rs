use anchor_client::{Client, Cluster};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use crate::{config::Config, rpc::send_txn, utils::read_keypair_file};

pub fn mint_to_token_account(
    config: &Config,
    mint: &Pubkey,
    token_account: &Pubkey,
    amount: u64,
) -> Result<()> {
    let payer = read_keypair_file(&config.global.payer_path).unwrap();
    let rpc_client = RpcClient::new(config.global.http_url.to_string());

    let mint_to_instr =
        spl_token_mint_to_instr(&config, &payer, &mint, &token_account, amount, &payer)?;

    let signers = vec![&payer];
    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &mint_to_instr,
        Some(&payer.pubkey()),
        &signers,
        recent_hash,
    );
    let _ = send_txn(&rpc_client, &txn, true)?;

    Ok(())
}

pub fn spl_token_mint_to_instr(
    config: &Config,
    payer: &Keypair,
    mint: &Pubkey,
    token_account: &Pubkey,
    amount: u64,
    mint_authority: &Keypair,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(spl_token_2022::id())?;

    let instructions = program
        .request()
        .instruction(spl_token_2022::instruction::mint_to(
            &program.id(),
            mint,
            token_account,
            &mint_authority.pubkey(),
            &[],
            amount,
        )?)
        .signer(mint_authority)
        .instructions()?;
    Ok(instructions)
}
