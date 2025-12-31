use anchor_lang::prelude::*;

declare_id!("2t4tTkX9nEKuTHdZbVYYXwq188GBQQq35fh46xV1qBuw");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

#[program]
pub mod staking {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        instructions::initialize_pool(ctx)
    }

    #[instruction(discriminator = 1)]
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake(ctx, amount)
    }

    #[instruction(discriminator = 2)]
    pub fn get_points(ctx: Context<GetPoints>) -> Result<u64> {
        instructions::get_points(ctx)
    }

    #[instruction(discriminator = 3)]
    pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
        instructions::claim_points(ctx)
    }

    #[instruction(discriminator = 4)]
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::unstake(ctx, amount)
    }

    #[instruction(discriminator = 5)]
    pub fn unstake_all(ctx: Context<UnstakeAll>) -> Result<()> {
        instructions::unstake_all(ctx)
    }
}
