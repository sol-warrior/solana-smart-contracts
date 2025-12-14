use anchor_lang::prelude::*;
pub mod error;
pub mod instructions;
pub mod state;
use crate::instructions::*;

declare_id!("DJktBCt1vV8jdYmyxqnH8oNVa5PMLWgkc7WuT4KB1Q8o");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeVault>) -> Result<()> {
        initialize_vault(ctx)
    }

    pub fn deposit(ctx: Context<DepositVault>, amount: u64) -> Result<()> {
        deposit_vault(ctx, amount)
    }

    pub fn withdraw(ctx: Context<WithdrawVault>, amount: u64) -> Result<()> {
        withdraw_vault(ctx, amount)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
