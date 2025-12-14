use crate::error::UserVaultError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
    // Ensure user does not already have a vault account
    if ctx.accounts.user_vault.total_deposit != 0
        || ctx.accounts.user_vault_lamports.lamports() != 0
    {
        return err!(UserVaultError::VaultAlreadyInitialized);
    }

    let user_vault_acc = &mut ctx.accounts.user_vault;
    user_vault_acc.user = ctx.accounts.user.key();
    user_vault_acc.user_vault_bump = ctx.bumps.user_vault;
    user_vault_acc.user_vault_lamports_bump = ctx.bumps.user_vault_lamports;
    user_vault_acc.total_deposit = 0;
    user_vault_acc.user_vault_lamports = ctx.accounts.user_vault_lamports.key();

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(
        init,
        seeds=[b"vault",user.key().as_ref()],
        bump,
        payer=user,
        space= 8 + UserVault::INIT_SPACE
    )]
    user_vault: Account<'info, UserVault>,

    #[account(
        mut,
        seeds=[b"user_lamports",user.key().as_ref(),user_vault.key().as_ref()],
        bump
    )]
    user_vault_lamports: SystemAccount<'info>,

    system_program: Program<'info, System>,
}
