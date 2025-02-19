use anchor_client::{Client, Cluster};
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::{config::Config, rpc::send_txn, utils::read_keypair_file};

pub fn create_token_account(config: &Config, mint: &Pubkey) -> Result<Pubkey> {
    let payer = read_keypair_file(&config.global.payer_path).unwrap();
    let rpc_client = RpcClient::new(config.global.http_url.to_string());

    let create_ata_instr = create_ata_token_account_instr(&config, &payer, mint, &payer.pubkey())?;

    let signers = vec![&payer];
    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &create_ata_instr,
        Some(&payer.pubkey()),
        &signers,
        recent_hash,
    );

    let _ = send_txn(&rpc_client, &txn, true)?;

    let token_account =
        get_associated_token_address_with_program_id(&payer.pubkey(), &mint, &spl_token_2022::id());

    Ok(token_account)
}

pub fn create_ata_token_account_instr(
    config: &Config,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(spl_token_2022::id())?;
    let instructions = program
        .request()
        .instruction(
            spl_associated_token_account::instruction::create_associated_token_account(
                &program.payer(),
                owner,
                mint,
                &spl_token_2022::id(),
            ),
        )
        .instructions()?;
    Ok(instructions)
}
