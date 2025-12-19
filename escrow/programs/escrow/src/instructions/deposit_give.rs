use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::EscrowError;
use crate::state::escrow::Escrow;

#[derive(Accounts)]
pub struct DepositGive<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", maker.key().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // ===== SPL PATH (passed but only used if give_mint != SOL) =====
    #[account(mut)]
    pub maker_give_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = maker,
        token::mint = give_mint,
        token::authority = escrow,
        seeds = [b"vault", escrow.key().as_ref()],
        bump
    )]
    pub give_token_vault: Account<'info, TokenAccount>,

    pub give_mint: Account<'info, Mint>,

    // ===== SOL PATH =====
    /// CHECK: PDA system account for SOL vault
    #[account(
        mut,
        seeds = [b"sol_vault", escrow.key().as_ref()],
        bump
    )]
    pub give_sol_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositGive>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;

    require!(!escrow.give_deposited, EscrowError::AlreadyDeposited);

    if escrow.give_is_sol() {
        deposit_sol(&ctx)?;
    } else {
        deposit_spl(&ctx)?;
    }

    ctx.accounts.escrow.give_deposited = true;
    Ok(())
}

fn deposit_sol(ctx: &Context<DepositGive>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let vault = &ctx.accounts.give_sol_vault;

    // Create vault if empty
    if vault.lamports() == 0 {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);

        let escrow_key = escrow.key();
        let seeds = &[
            b"sol_vault",
            escrow_key.as_ref(),
            &[ctx.bumps.give_sol_vault],
        ];
        let signer = &[&seeds[..]];

        let ix = system_instruction::create_account(
            &ctx.accounts.maker.key(),
            vault.key,
            lamports,
            0,
            ctx.program_id,
        );

        invoke_signed(
            &ix,
            &[
                ctx.accounts.maker.to_account_info(),
                vault.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer,
        )?;
    }

    let ix = system_instruction::transfer(&ctx.accounts.maker.key(), vault.key, escrow.give_amount);

    invoke(
        &ix,
        &[
            ctx.accounts.maker.to_account_info(),
            vault.clone(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    Ok(())
}

fn deposit_spl(ctx: &Context<DepositGive>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;

    require!(
        ctx.accounts.maker_give_ata.mint == escrow.give_mint,
        EscrowError::InvalidMint
    );
    require!(
        ctx.accounts.maker_give_ata.owner == ctx.accounts.maker.key(),
        EscrowError::Unauthorized
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.maker_give_ata.to_account_info(),
        to: ctx.accounts.give_token_vault.to_account_info(),
        authority: ctx.accounts.maker.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    token::transfer(cpi_ctx, escrow.give_amount)?;

    Ok(())
}
