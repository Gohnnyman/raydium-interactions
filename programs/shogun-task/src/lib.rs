use anchor_lang::prelude::*;
mod instructions;

use instructions::*;

declare_id!("ELsxP11QYREU4RDadUZS5DXEPPjaqwa4csursWD6aeL6");

#[program]
pub mod shogun_task {
    use super::*;
}

#[derive(Accounts)]
pub struct Initialize {}
