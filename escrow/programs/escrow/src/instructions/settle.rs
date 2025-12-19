use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, CloseAccount, Transfer};

use crate::errors::EscrowError;
use crate::instructions::deposit_want::DepositWant;

use crate::state::escrow::Escrow;

pub fn settle_internal<'a>(ctx: &Context<'a, 'a, 'a, 'a, DepositWant<'a>>) -> Result<()> {
    // ------------------------------------------------
    // Escrow (typed, safe – NO AccountInfo conversion)
    // ------------------------------------------------
    let escrow = &ctx.accounts.escrow.to_account_info();
    // Dynamically clone all remaining accounts into a Vec<AccountInfo>
    let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().cloned().collect();

    // require!(
    //     escrow.give_deposited && escrow.want_deposited,
    //     EscrowError::NotReady
    // );
    let maker = remaining[0].clone();
    let taker = remaining[1].clone();
    // ------------------------------------------------
    // Remaining accounts (DYNAMIC, RAW)
    // ------------------------------------------------
    // let remaining: Vec<AccountInfo> = ctx.remaining_accounts.iter().map(|a| a.clone()).collect();

    // require!(remaining.len() == 8, EscrowError::InvalidAccounts);

    // let maker = remaining[0].clone();
    // let taker = remaining[1].clone();

    let give_vault = remaining[2].clone();
    let taker_receive = remaining[3].clone();

    let want_vault = remaining[4].clone();
    let maker_receive = remaining[5].clone();

    let token_program = remaining[6].clone();
    let system_program = remaining[7].clone();

    let escrow_key = escrow.key();

    // =========================
    // 1. GIVE → TAKER
    // =========================
    if ctx.accounts.escrow.give_is_sol() {
        let ix = system_instruction::transfer(
            give_vault.key,
            taker.key,
            ctx.accounts.escrow.give_amount,
        );

        let seeds = &[
            b"sol_vault",
            escrow_key.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[give_vault.clone(), taker.clone(), system_program.clone()],
            signer,
        )?;
    } else {
        let cpi_accounts = Transfer {
            from: give_vault.clone(),
            to: taker_receive.clone(),
            authority: escrow.to_account_info(),
        };

        let seeds = &[
            b"escrow",
            ctx.accounts.escrow.maker.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer);

        token::transfer(cpi_ctx, ctx.accounts.escrow.give_amount)?;
    }

    // =========================
    // 2. WANT → MAKER
    // =========================
    if ctx.accounts.escrow.want_is_sol() {
        let ix = system_instruction::transfer(
            want_vault.key,
            maker.key,
            ctx.accounts.escrow.want_amount,
        );

        let seeds = &[
            b"sol_vault",
            escrow_key.as_ref(),
            b"want",
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        anchor_lang::solana_program::program::invoke_signed(
            &ix,
            &[want_vault.clone(), maker.clone(), system_program.clone()],
            signer,
        )?;
    } else {
        let cpi_accounts = Transfer {
            from: want_vault.clone(),
            to: maker_receive.clone(),
            authority: escrow.to_account_info(),
        };

        let seeds = &[
            b"escrow",
            ctx.accounts.escrow.maker.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer);

        token::transfer(cpi_ctx, ctx.accounts.escrow.want_amount)?;
    }

    // =========================
    // 3. CLOSE SPL VAULTS
    // =========================
    if !ctx.accounts.escrow.give_is_sol() {
        let close = CloseAccount {
            account: give_vault.clone(),
            destination: maker.clone(),
            authority: escrow.to_account_info(),
        };

        let seeds = &[
            b"escrow",
            ctx.accounts.escrow.maker.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), close, signer);

        token::close_account(cpi_ctx)?;
    }

    if !ctx.accounts.escrow.want_is_sol() {
        let close = CloseAccount {
            account: want_vault.clone(),
            destination: maker.clone(),
            authority: escrow.to_account_info(),
        };

        let seeds = &[
            b"escrow",
            ctx.accounts.escrow.maker.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), close, signer);

        token::close_account(cpi_ctx)?;
    }

    Ok(())
}
