use anchor_client::{Client, Cluster};
use anchor_lang::prelude::AccountMeta;
use anyhow::Result;
use raydium_amm_v3::accounts as raydium_accounts;
use raydium_amm_v3::instruction as raydium_instruction;
use raydium_amm_v3::libraries::liquidity_math;
use raydium_amm_v3::libraries::tick_math;
use raydium_amm_v3::states::POSITION_SEED;
use raydium_amm_v3::states::TICK_ARRAY_SEED;
use solana_client::rpc_client::RpcClient;
use solana_sdk::system_program;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;

use crate::utils::amount_with_slippage;
use crate::utils::deserialize_anchor_account;
use crate::utils::get_all_nft_and_position_by_owner;
use crate::utils::get_pool_mints_transfer_fee;
use crate::utils::get_tick_array_bitmap;
use crate::utils::tick_with_spacing;
use crate::{
    config::Config,
    rpc::send_txn,
    utils::{price_to_sqrt_price_x64, read_keypair_file},
};

pub fn decrease_liquidity(
    config: &Config,
    tick_lower_price: f64,
    tick_upper_price: f64,
    liquidity: Option<u128>,
    pool_pubkey: Pubkey,
    slippage: f64,
) -> Result<()> {
    let payer = read_keypair_file(&config.global.payer_path).unwrap();

    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, &payer);
    let rpc_client = RpcClient::new(config.global.http_url.to_string());

    let program = client.program(config.global.raydium_v3_program.parse().unwrap())?;
    let program_pubkey = program.id();

    // load pool to get observation
    let pool: raydium_amm_v3::states::PoolState = program.account(pool_pubkey)?;

    let mint0 = pool.token_mint_0;
    let mint1 = pool.token_mint_1;
    let amm_config = pool.amm_config;

    let tick_lower_price_x64 =
        price_to_sqrt_price_x64(tick_lower_price, pool.mint_decimals_0, pool.mint_decimals_1);
    let tick_upper_price_x64 =
        price_to_sqrt_price_x64(tick_upper_price, pool.mint_decimals_0, pool.mint_decimals_1);
    let tick_lower_index = tick_with_spacing(
        tick_math::get_tick_at_sqrt_price(tick_lower_price_x64)?,
        pool.tick_spacing.into(),
    );
    let tick_upper_index = tick_with_spacing(
        tick_math::get_tick_at_sqrt_price(tick_upper_price_x64)?,
        pool.tick_spacing.into(),
    );

    let tick_array_lower_start_index =
        raydium_amm_v3::states::TickArrayState::get_array_start_index(
            tick_lower_index,
            pool.tick_spacing.into(),
        );
    let tick_array_upper_start_index =
        raydium_amm_v3::states::TickArrayState::get_array_start_index(
            tick_upper_index,
            pool.tick_spacing.into(),
        );
    // load position
    let position_nft_infos =
        get_all_nft_and_position_by_owner(&rpc_client, &payer.pubkey(), &program_pubkey);

    let positions: Vec<Pubkey> = position_nft_infos
        .iter()
        .map(|item| item.position)
        .collect();
    let rsps = rpc_client.get_multiple_accounts(&positions)?;
    let mut user_positions = Vec::new();
    for rsp in rsps {
        match rsp {
            None => continue,
            Some(rsp) => {
                let position = deserialize_anchor_account::<
                    raydium_amm_v3::states::PersonalPositionState,
                >(&rsp)?;
                user_positions.push(position);
            }
        }
    }
    let mut find_position = raydium_amm_v3::states::PersonalPositionState::default();
    for position in user_positions {
        if position.pool_id == pool_pubkey
            && position.tick_lower_index == tick_lower_index
            && position.tick_upper_index == tick_upper_index
        {
            find_position = position.clone();
            println!("liquidity:{:?}", find_position);
        }
    }

    let tickarray_bitmap_extension = get_tick_array_bitmap(
        &amm_config,
        &mint0,
        &mint1,
        &config.global.raydium_v3_program.parse().unwrap(),
    );

    if find_position.nft_mint != Pubkey::default() && find_position.pool_id == pool_pubkey {
        let user_nft_token_info = position_nft_infos
            .iter()
            .find(|&nft_info| nft_info.mint == find_position.nft_mint)
            .unwrap();
        let mut reward_vault_with_user_vault: Vec<Pubkey> = Vec::new();
        for item in pool.reward_infos.into_iter() {
            if item.token_mint != Pubkey::default() {
                reward_vault_with_user_vault.push(item.token_vault);
                reward_vault_with_user_vault.push(get_associated_token_address(
                    &payer.pubkey(),
                    &item.token_mint,
                ));
                reward_vault_with_user_vault.push(item.token_mint);
            }
        }
        let liquidity = if let Some(liquidity) = liquidity {
            liquidity
        } else {
            find_position.liquidity
        };
        let (amount_0, amount_1) = liquidity_math::get_delta_amounts_signed(
            pool.tick_current,
            pool.sqrt_price_x64,
            tick_lower_index,
            tick_upper_index,
            -(liquidity as i128),
        )?;
        let amount_0_with_slippage = amount_with_slippage(amount_0, slippage, false);
        let amount_1_with_slippage = amount_with_slippage(amount_1, slippage, false);
        let transfer_fee = get_pool_mints_transfer_fee(
            &rpc_client,
            pool.token_mint_0,
            pool.token_mint_1,
            amount_0_with_slippage,
            amount_1_with_slippage,
        );
        let amount_0_min = amount_0_with_slippage
            .checked_sub(transfer_fee.0.transfer_fee)
            .unwrap();
        let amount_1_min = amount_1_with_slippage
            .checked_sub(transfer_fee.1.transfer_fee)
            .unwrap();

        let mut remaining_accounts = Vec::new();
        remaining_accounts.push(AccountMeta::new(tickarray_bitmap_extension, false));

        let mut accounts = reward_vault_with_user_vault
            .into_iter()
            .map(|item| AccountMeta::new(item, false))
            .collect();
        remaining_accounts.append(&mut accounts);
        // personal position exist
        let mut decrease_instr = decrease_liquidity_instr(
            &config,
            &payer,
            pool_pubkey,
            pool.token_vault_0,
            pool.token_vault_1,
            pool.token_mint_0,
            pool.token_mint_1,
            find_position.nft_mint,
            user_nft_token_info.key,
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &payer.pubkey(),
                &mint0,
                &transfer_fee.0.owner,
            ),
            spl_associated_token_account::get_associated_token_address_with_program_id(
                &payer.pubkey(),
                &mint1,
                &transfer_fee.1.owner,
            ),
            remaining_accounts,
            liquidity,
            amount_0_min,
            amount_1_min,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
        )?;
        if liquidity == find_position.liquidity {
            let close_position_instr = close_personal_position_instr(
                &config,
                &payer,
                find_position.nft_mint,
                user_nft_token_info.key,
                user_nft_token_info.program,
            )?;
            decrease_instr.extend(close_position_instr);
        }
        // send
        let signers = vec![&payer];
        let recent_hash = rpc_client.get_latest_blockhash()?;
        let txn = Transaction::new_signed_with_payer(
            &decrease_instr,
            Some(&payer.pubkey()),
            &signers,
            recent_hash,
        );
        let signature = send_txn(&rpc_client, &txn, true)?;
        println!("{}", signature);
    } else {
        println!("Position doesn't exist");
    }

    Ok(())
}

pub fn decrease_liquidity_instr(
    config: &Config,
    payer: &Keypair,
    pool_account_key: Pubkey,
    token_vault_0: Pubkey,
    token_vault_1: Pubkey,
    token_mint_0: Pubkey,
    token_mint_1: Pubkey,
    nft_mint_key: Pubkey,
    nft_token_key: Pubkey,
    user_token_account_0: Pubkey,
    user_token_account_1: Pubkey,
    remaining_accounts: Vec<AccountMeta>,
    liquidity: u128,
    amount_0_min: u64,
    amount_1_min: u64,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(config.global.raydium_v3_program.parse().unwrap())?;
    let (personal_position_key, __bump) = Pubkey::find_program_address(
        &[POSITION_SEED.as_bytes(), nft_mint_key.to_bytes().as_ref()],
        &program.id(),
    );
    let (protocol_position_key, __bump) = Pubkey::find_program_address(
        &[
            POSITION_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_lower_index.to_be_bytes(),
            &tick_upper_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let (tick_array_lower, __bump) = Pubkey::find_program_address(
        &[
            TICK_ARRAY_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_array_lower_start_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let (tick_array_upper, __bump) = Pubkey::find_program_address(
        &[
            TICK_ARRAY_SEED.as_bytes(),
            pool_account_key.to_bytes().as_ref(),
            &tick_array_upper_start_index.to_be_bytes(),
        ],
        &program.id(),
    );
    let instructions = program
        .request()
        .accounts(raydium_accounts::DecreaseLiquidityV2 {
            nft_owner: program.payer(),
            nft_account: nft_token_key,
            personal_position: personal_position_key,
            pool_state: pool_account_key,
            protocol_position: protocol_position_key,
            token_vault_0,
            token_vault_1,
            tick_array_lower,
            tick_array_upper,
            recipient_token_account_0: user_token_account_0,
            recipient_token_account_1: user_token_account_1,
            token_program: spl_token::id(),
            token_program_2022: spl_token_2022::id(),
            memo_program: spl_memo::id(),
            vault_0_mint: token_mint_0,
            vault_1_mint: token_mint_1,
        })
        .accounts(remaining_accounts)
        .args(raydium_instruction::DecreaseLiquidityV2 {
            liquidity,
            amount_0_min,
            amount_1_min,
        })
        .instructions()?;
    Ok(instructions)
}

pub fn close_personal_position_instr(
    config: &Config,
    payer: &Keypair,
    nft_mint_key: Pubkey,
    nft_token_key: Pubkey,
    nft_token_program: Pubkey,
) -> Result<Vec<Instruction>> {
    let url = Cluster::Custom(config.global.http_url.clone(), config.global.ws_url.clone());
    let client = Client::new(url, payer);

    let program = client.program(config.global.raydium_v3_program.parse().unwrap())?;
    let (personal_position_key, __bump) = Pubkey::find_program_address(
        &[POSITION_SEED.as_bytes(), nft_mint_key.to_bytes().as_ref()],
        &program.id(),
    );
    let instructions = program
        .request()
        .accounts(raydium_accounts::ClosePosition {
            nft_owner: program.payer(),
            position_nft_mint: nft_mint_key,
            position_nft_account: nft_token_key,
            personal_position: personal_position_key,
            system_program: system_program::id(),
            token_program: nft_token_program,
        })
        .args(raydium_instruction::ClosePosition)
        .instructions()?;
    Ok(instructions)
}
