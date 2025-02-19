use anchor_lang::prelude::*;
mod instructions;

use instructions::*;

declare_id!("ELsxP11QYREU4RDadUZS5DXEPPjaqwa4csursWD6aeL6");

#[program]
pub mod shogun_task {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn proxy_initialize(
        ctx: Context<ProxyInitialize>,
        sqrt_price_x64: u128,
        open_time: u64,
    ) -> Result<()> {
        instructions::proxy_initialize(ctx, sqrt_price_x64, open_time)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
