use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, CloseAccount, TokenAccount, Transfer};

use crate::errors::EscrowError;
use crate::state::escrow::Escrow;

#[derive(Accounts)]
pub struct Cancel<'info> {
    /// Only maker can cancel
    #[account(mut)]
    pub maker: Signer<'info>,

    /// Escrow PDA
    #[account(
        mut,
        seeds = [b"escrow", maker.key().as_ref()],
        bump = escrow.bump,
        constraint = escrow.maker == maker.key() @ EscrowError::Unauthorized,
        constraint = escrow.taker.is_none() @ EscrowError::AlreadyTaken,
    )]
    pub escrow: Account<'info, Escrow>,

    /// CHECK: SOL vault PDA (optional, may be empty)
    #[account(
        mut,
        seeds = [b"sol_vault", escrow.key().as_ref()],
        bump
    )]
    pub give_sol_vault: AccountInfo<'info>,

    /// SPL token vault (optional, may not exist)
    #[account(mut)]
    pub give_token_vault: Option<Account<'info, TokenAccount>>,

    /// Maker receive ATA (for SPL refund)
    #[account(mut)]
    pub maker_receive_ata: Option<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Cancel>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;

    // If nothing was deposited, just close escrow
    if !escrow.give_deposited {
        return Ok(());
    }

    let escrow_key = ctx.accounts.escrow.key();

    // =========================
    // REFUND GIVE ASSET
    // =========================
    if escrow.give_is_sol() {
        // Refund SOL
        let ix = system_instruction::transfer(
            ctx.accounts.give_sol_vault.key,
            ctx.accounts.maker.key,
            escrow.give_amount,
        );

        let seeds = &[b"sol_vault", escrow_key.as_ref(), &[escrow.bump]];
        let signer = &[&seeds[..]];

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.give_sol_vault.clone(),
                ctx.accounts.maker.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer,
        )?;
    } else {
        // Refund SPL
        let vault = ctx
            .accounts
            .give_token_vault
            .as_ref()
            .ok_or(EscrowError::InvalidVault)?;

        let dest = ctx
            .accounts
            .maker_receive_ata
            .as_ref()
            .ok_or(EscrowError::InvalidVault)?;

        let cpi_accounts = Transfer {
            from: vault.to_account_info(),
            to: dest.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };

        let seeds = &[b"escrow", escrow.maker.as_ref(), &[escrow.bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        token::transfer(cpi_ctx, escrow.give_amount)?;

        // Close vault
        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: vault.to_account_info(),
                destination: ctx.accounts.maker.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            signer,
        );

        token::close_account(close_ctx)?;
    }

    Ok(())
}
