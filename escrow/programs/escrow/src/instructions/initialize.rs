use crate::{errors::EscrowError, state::escrow::Escrow};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    #[account(
        init,
        payer = maker,
        space = Escrow::LEN,
        seeds = [b"escrow", maker.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub maker: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeEscrow>,
    give_mint: Pubkey,
    give_amount: u64,
    want_mint: Pubkey,
    want_amount: u64,
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    require!(give_amount > 0, EscrowError::InvalidMint);
    require!(want_amount > 0, EscrowError::InvalidMint);
    require!(give_mint != want_mint, EscrowError::InvalidMint);

    escrow.maker = ctx.accounts.maker.key();
    escrow.taker = None;

    escrow.give_mint = give_mint;
    escrow.give_amount = give_amount;

    escrow.want_mint = want_mint;
    escrow.want_amount = want_amount;

    escrow.give_deposited = false;
    escrow.want_deposited = false;

    escrow.bump = ctx.bumps.escrow;

    Ok(())
}
