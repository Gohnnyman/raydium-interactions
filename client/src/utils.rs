use std::{env, ops::Mul};

use anchor_lang::AccountDeserialize;
use anyhow::{anyhow, Result};

use raydium_amm_v3::states::POOL_TICK_ARRAY_BITMAP_SEED;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey::Pubkey, signature::Keypair};
use spl_token_2022::{
    extension::{
        transfer_fee::{TransferFeeConfig, MAX_FEE_BASIS_POINTS},
        BaseState, BaseStateWithExtensions, StateWithExtensions,
    },
    state::Mint,
};

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

pub fn get_tick_array_bitmap(
    amm_config: &Pubkey,
    mint0: &Pubkey,
    mint1: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let pool_id_account = Pubkey::find_program_address(
        &[
            raydium_amm_v3::states::POOL_SEED.as_bytes(),
            amm_config.to_bytes().as_ref(),
            mint0.to_bytes().as_ref(),
            mint1.to_bytes().as_ref(),
        ],
        program_id,
    )
    .0;

    let tickarray_bitmap_extension = Pubkey::find_program_address(
        &[
            POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
            pool_id_account.to_bytes().as_ref(),
        ],
        program_id,
    )
    .0;

    tickarray_bitmap_extension
}

pub fn deserialize_anchor_account<T: AccountDeserialize>(account: &Account) -> Result<T> {
    let mut data: &[u8] = &account.data;
    T::try_deserialize(&mut data).map_err(Into::into)
}

pub fn tick_with_spacing(tick: i32, tick_spacing: i32) -> i32 {
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        compressed -= 1; // round towards negative infinity
    }
    compressed * tick_spacing
}

pub fn amount_with_slippage(amount: u64, slippage: f64, round_up: bool) -> u64 {
    if round_up {
        (amount as f64).mul(1_f64 + slippage).ceil() as u64
    } else {
        (amount as f64).mul(1_f64 - slippage).floor() as u64
    }
}

#[derive(Debug)]
pub struct TransferFeeInfo {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub transfer_fee: u64,
}

pub fn get_pool_mints_inverse_fee(
    rpc_client: &RpcClient,
    token_mint_0: Pubkey,
    token_mint_1: Pubkey,
    post_fee_amount_0: u64,
    post_fee_amount_1: u64,
) -> (TransferFeeInfo, TransferFeeInfo) {
    let load_accounts = vec![token_mint_0, token_mint_1];
    let rsps = rpc_client.get_multiple_accounts(&load_accounts).unwrap();
    let epoch = rpc_client.get_epoch_info().unwrap().epoch;
    let mint0_account = rsps[0].clone().ok_or("load mint0 rps error!").unwrap();
    let mint1_account = rsps[1].clone().ok_or("load mint0 rps error!").unwrap();
    let mint0_state = StateWithExtensions::<Mint>::unpack(&mint0_account.data).unwrap();
    let mint1_state = StateWithExtensions::<Mint>::unpack(&mint1_account.data).unwrap();
    (
        TransferFeeInfo {
            mint: token_mint_0,
            owner: mint0_account.owner,
            transfer_fee: get_transfer_inverse_fee(&mint0_state, post_fee_amount_0, epoch),
        },
        TransferFeeInfo {
            mint: token_mint_1,
            owner: mint1_account.owner,
            transfer_fee: get_transfer_inverse_fee(&mint1_state, post_fee_amount_1, epoch),
        },
    )
}

/// Calculate the fee for output amount
pub fn get_transfer_inverse_fee<'data, S: BaseState>(
    account_state: &StateWithExtensions<'data, S>,
    epoch: u64,
    post_fee_amount: u64,
) -> u64 {
    let fee = if let Ok(transfer_fee_config) = account_state.get_extension::<TransferFeeConfig>() {
        let transfer_fee = transfer_fee_config.get_epoch_fee(epoch);
        if u16::from(transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
            u64::from(transfer_fee.maximum_fee)
        } else {
            transfer_fee_config
                .calculate_inverse_epoch_fee(epoch, post_fee_amount)
                .unwrap()
        }
    } else {
        0
    };
    fee
}
