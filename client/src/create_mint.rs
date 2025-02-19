use anchor_client::{Client, Cluster};
use anyhow::Result;
use rand::rngs::OsRng;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey, signature::Keypair,
    signer::Signer, system_instruction, transaction::Transaction,
};
use spl_token_2022::state::Mint;

use crate::{config::Config, rpc::send_txn, utils::read_keypair_file};

pub fn create_mint(config: &Config) -> Result<Pubkey> {
    let payer = read_keypair_file(&config.global.payer_path).unwrap();
    let rpc_client = RpcClient::new(config.global.http_url.to_string());

    let authority = payer.pubkey();
    let mint = Keypair::generate(&mut OsRng);
    let create_and_init_instr =
        create_and_init_mint_instr(&config, &payer, &mint.pubkey(), &authority, 0)?;
    // send
    let signers = vec![&payer, &mint];
    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &create_and_init_instr,
        Some(&payer.pubkey()),
        &signers,
        recent_hash,
    );

    let _ = send_txn(&rpc_client, &txn, true)?;

    Ok(mint.pubkey())
}

pub fn create_and_init_mint_instr(
    config: &Config,
    payer: &Keypair,
    mint_key: &Pubkey,
    mint_authority: &Pubkey,
    decimals: u8,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(spl_token_2022::id())?;

    let space = Mint::LEN;

    let mut instructions = vec![system_instruction::create_account(
        &program.payer(),
        mint_key,
        program
            .rpc()
            .get_minimum_balance_for_rent_exemption(space)?,
        space as u64,
        &program.id(),
    )];

    instructions.push(spl_token_2022::instruction::initialize_mint(
        &program.id(),
        mint_key,
        mint_authority,
        None,
        decimals,
    )?);

    Ok(instructions)
}
