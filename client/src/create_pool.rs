use anchor_client::{Client, Cluster};
use anyhow::Result;
use raydium_amm_v3::accounts as raydium_accounts;
use raydium_amm_v3::instruction as raydium_instruction;
use raydium_amm_v3::{
    libraries::tick_math,
    states::{OBSERVATION_SEED, POOL_SEED, POOL_VAULT_SEED},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey, signature::Keypair,
    signer::Signer, transaction::Transaction,
};
use solana_sdk::{system_program, sysvar};

use crate::utils::get_tick_array_bitmap;
use crate::{
    config::Config,
    rpc::send_txn,
    utils::{price_to_sqrt_price_x64, read_keypair_file},
};

pub fn create_pool(
    config: &Config,
    config_index: u16,
    price: f64,
    mint0: Pubkey,
    mint1: Pubkey,
    open_time: u64,
) -> Result<Pubkey> {
    let payer = read_keypair_file(&config.global.payer_path).unwrap();
    let rpc_client = RpcClient::new(config.global.http_url.to_string());

    let raydium_v3_program = config.global.raydium_v3_program.parse().unwrap();

    let mut price = price;
    let mut mint0 = mint0;
    let mut mint1 = mint1;
    if mint0 > mint1 {
        std::mem::swap(&mut mint0, &mut mint1);
        price = 1.0 / price;
    }
    let load_pubkeys = vec![mint0, mint1];
    let rsps = rpc_client.get_multiple_accounts(&load_pubkeys)?;
    let mint0_owner = rsps[0].clone().unwrap().owner;
    let mint1_owner = rsps[1].clone().unwrap().owner;
    let mint0_account =
        spl_token_2022::state::Mint::unpack(&rsps[0].as_ref().unwrap().data).unwrap();
    let mint1_account =
        spl_token_2022::state::Mint::unpack(&rsps[1].as_ref().unwrap().data).unwrap();

    let sqrt_price_x64 =
        price_to_sqrt_price_x64(price, mint0_account.decimals, mint1_account.decimals);

    let (amm_config_key, __bump) = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::AMM_CONFIG_SEED.as_bytes(),
            &config_index.to_be_bytes(),
        ],
        &raydium_v3_program,
    );
    let tick = tick_math::get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
    println!(
        "tick:{}, price:{}, sqrt_price_x64:{}, amm_config_key:{}",
        tick, price, sqrt_price_x64, amm_config_key
    );

    let create_pool_instr = create_pool_instr(
        &config,
        &payer,
        amm_config_key,
        mint0,
        mint1,
        mint0_owner,
        mint1_owner,
        sqrt_price_x64,
        open_time,
    )?;

    // send
    let signers = vec![&payer];
    let recent_hash = rpc_client.get_latest_blockhash()?;
    let txn = Transaction::new_signed_with_payer(
        &create_pool_instr,
        Some(&payer.pubkey()),
        &signers,
        recent_hash,
    );
    let signature = send_txn(&rpc_client, &txn, true)?;
    println!("{}", signature);

    let (pool, _) = Pubkey::find_program_address(
        &[
            POOL_SEED.as_bytes(),
            amm_config_key.to_bytes().as_ref(),
            mint0.to_bytes().as_ref(),
            mint1.to_bytes().as_ref(),
        ],
        &raydium_v3_program,
    );

    Ok(pool)
}

pub fn create_pool_instr(
    config: &Config,
    payer: &Keypair,
    amm_config: Pubkey,
    token_mint_0: Pubkey,
    token_mint_1: Pubkey,
    token_program_0: Pubkey,
    token_program_1: Pubkey,
    sqrt_price_x64: u128,
    open_time: u64,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(config.global.raydium_v3_program.parse().unwrap())?;

    let (pool_account_key, __bump) = Pubkey::find_program_address(
        &[
            POOL_SEED.as_bytes(),
            amm_config.to_bytes().as_ref(),
            token_mint_0.to_bytes().as_ref(),
            token_mint_1.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    let (token_vault_0, __bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_mint_0.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    let (token_vault_1, __bump) = Pubkey::find_program_address(
        &[
            POOL_VAULT_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            token_mint_1.to_bytes().as_ref(),
        ],
        &program.id(),
    );
    let (observation_key, __bump) = Pubkey::find_program_address(
        &[
            OBSERVATION_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
        ],
        &program.id(),
    );

    let tick_array_bitmap = get_tick_array_bitmap(
        &amm_config,
        &token_mint_0,
        &token_mint_1,
        &config.global.raydium_v3_program.parse().unwrap(),
    );

    let instructions = program
        .request()
        .accounts(raydium_accounts::CreatePool {
            pool_creator: program.payer(),
            amm_config,
            pool_state: pool_account_key,
            token_mint_0,
            token_mint_1,
            token_vault_0,
            token_vault_1,
            observation_state: observation_key,
            tick_array_bitmap,
            token_program_0,
            token_program_1,
            system_program: system_program::id(),
            rent: sysvar::rent::id(),
        })
        .args(raydium_instruction::CreatePool {
            sqrt_price_x64,
            open_time,
        })
        .instructions()?;
    Ok(instructions)
}
