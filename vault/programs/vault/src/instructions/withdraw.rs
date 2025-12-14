use crate::error::UserVaultError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

pub fn withdraw_vault(ctx: Context<WithdrawVault>, amount: u64) -> Result<()> {
    let from = ctx.accounts.user_vault_lamports.to_account_info();
    let to = &ctx.accounts.user.to_account_info();
    let to_vault_acc = &ctx.accounts.user_vault;

    require_keys_eq!(
        to_vault_acc.user.key(),
        to.key(),
        UserVaultError::NotVaultOwner
    );
    require_keys_eq!(
        to_vault_acc.user_vault_lamports,
        from.key(),
        UserVaultError::NotAuthorised
    );
    require!(from.lamports() >= amount, UserVaultError::InsufficientFunds);
    let program_id = ctx.accounts.system_program.to_account_info();

    let user_key = to.key();
    let vault_key = to_vault_acc.key();
    let bump_seed = to_vault_acc.user_vault_lamports_bump;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"user_lamports",
        user_key.as_ref(),
        vault_key.as_ref(),
        &[bump_seed],
    ]];

    let cpi_context = CpiContext::new(
        program_id,
        Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
        },
    )
    .with_signer(signer_seeds);

    transfer(cpi_context, amount)?;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawVault<'info> {
    #[account(mut)]
    user: Signer<'info>,

    #[account(
        mut,
        seeds=[b"vault",user.key().as_ref()],
        bump= user_vault.user_vault_bump,
    )]
    user_vault: Account<'info, UserVault>,

    #[account(
        mut,
        seeds=[b"user_lamports",user.key().as_ref(),user_vault.key().as_ref()],
        bump= user_vault.user_vault_lamports_bump
    )]
    user_vault_lamports: SystemAccount<'info>,

    system_program: Program<'info, System>,
}
