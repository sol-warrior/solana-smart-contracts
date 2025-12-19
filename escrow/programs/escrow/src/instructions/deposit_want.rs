use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::EscrowError;
use crate::instructions::settle::settle_internal;
use crate::state::escrow::Escrow;

#[derive(Accounts)]
pub struct DepositWant<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow.maker.as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    // ===== SPL PATH =====
    #[account(mut)]
    pub taker_want_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        token::mint = want_mint,
        token::authority = escrow,
        seeds = [b"vault", escrow.key().as_ref(), b"want"],
        bump
    )]
    pub want_token_vault: Account<'info, TokenAccount>,

    pub want_mint: Account<'info, Mint>,

    // ===== SOL PATH =====
    /// CHECK: PDA system account for SOL vault
    #[account(
        mut,
        seeds = [b"sol_vault", escrow.key().as_ref(), b"want"],
        bump
    )]
    pub want_sol_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// pub fn handler(ctx: Context<DepositWant>) -> Result<()> {
//     // ---- VALIDATION (no mutable borrow yet)
//     require!(
//         !ctx.accounts.escrow.want_deposited,
//         EscrowError::AlreadyDeposited
//     );
//     require!(
//         ctx.accounts.escrow.taker.is_none(),
//         EscrowError::AlreadyTaken
//     );

//     let is_sol = ctx.accounts.escrow.want_is_sol();

//     // ---- ACTION
//     if is_sol {
//         deposit_sol(&ctx)?;
//     } else {
//         deposit_spl(&ctx)?;
//     }

//     // // ---- STATE UPDATE (last)
//     // ctx.accounts.escrow.want_deposited = true;
//     // ctx.accounts.escrow.taker = Some(ctx.accounts.taker.key());

//     // // ---- AUTO SETTLE
//     // // let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();
//     // let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();

//     // if ctx.accounts.escrow.give_deposited {
//     //     // settle_internal(ctx.accounts.escrow.to_account_info(), &remaining)?;
//     //     settle_internal(ctx.accounts.escrow.to_account_info(), &remaining)?;
//     // }

//     // ---- PREPARE ACCOUNTINFOS FIRST (important)
//     let escrow_ai = ctx.accounts.escrow.to_account_info();
//     let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();

//     // ---- STATE UPDATE (last mutation)
//     ctx.accounts.escrow.want_deposited = true;
//     ctx.accounts.escrow.taker = Some(ctx.accounts.taker.key());

//     // ---- AUTO SETTLE
//     if ctx.accounts.escrow.give_deposited {
//         settle_internal(escrow_ai, &remaining)?;
//     }

//     Ok(())
// }

pub fn handler<'a>(ctx: Context<'a, 'a, 'a, 'a, DepositWant<'a>>) -> Result<()> {
    // ---- PREPARE RAW ACCOUNTINFOS FIRST (VERY IMPORTANT)
    let escrow_ai = &ctx.accounts.escrow.to_account_info();
    let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();

    // ---- VALIDATION (immutable borrows only)
    require!(
        !ctx.accounts.escrow.want_deposited,
        EscrowError::AlreadyDeposited
    );
    require!(
        ctx.accounts.escrow.taker.is_none(),
        EscrowError::AlreadyTaken
    );

    let is_sol = ctx.accounts.escrow.want_is_sol();

    // ---- ACTION
    if is_sol {
        deposit_sol(&ctx)?;
    } else {
        deposit_spl(&ctx)?;
    }

    // ---- STATE UPDATE (mutable borrow LAST)
    ctx.accounts.escrow.want_deposited = true;
    ctx.accounts.escrow.taker = Some(ctx.accounts.taker.key());

    // ---- AUTO SETTLE (RAW ACCOUNTINFO WORLD)
    if ctx.accounts.escrow.give_deposited {
        // settle_internal(escrow_ai, &remaining)?;
        settle_internal(&ctx);
    }

    Ok(())
}

fn deposit_sol(ctx: &Context<DepositWant>) -> Result<()> {
    let vault = &ctx.accounts.want_sol_vault;

    if vault.lamports() == 0 {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(0);

        let escrow_key = ctx.accounts.escrow.key();

        let seeds = &[
            b"sol_vault",
            escrow_key.as_ref(),
            b"want",
            &[ctx.bumps.want_sol_vault],
        ];
        let signer = &[&seeds[..]];

        let ix = system_instruction::create_account(
            &ctx.accounts.taker.key(),
            vault.key,
            lamports,
            0,
            ctx.program_id,
        );

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.taker.to_account_info(),
                vault.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer,
        )?;
    }

    let ix = system_instruction::transfer(
        &ctx.accounts.taker.key(),
        vault.key,
        ctx.accounts.escrow.want_amount,
    );

    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.taker.to_account_info(),
            vault.clone(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    Ok(())
}

fn deposit_spl(ctx: &Context<DepositWant>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;

    require!(
        ctx.accounts.taker_want_ata.mint == escrow.want_mint,
        EscrowError::InvalidMint
    );
    require!(
        ctx.accounts.taker_want_ata.owner == ctx.accounts.taker.key(),
        EscrowError::Unauthorized
    );

    let cpi_accounts = Transfer {
        from: ctx.accounts.taker_want_ata.to_account_info(),
        to: ctx.accounts.want_token_vault.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

    token::transfer(cpi_ctx, escrow.want_amount)?;

    Ok(())
}
