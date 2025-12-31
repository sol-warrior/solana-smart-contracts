use crate::{errors::StakingError, state::Pool};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // pool creator

    #[account(
        init,
        payer = authority,
        seeds = [b"pool", authority.key().as_ref()],
        bump,
        space =  Pool::DISCRIMINATOR.len() + Pool::INIT_SPACE
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>, // USDC Mint

    #[account(
        init,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializePool<'info> {
    fn populate_pool(&mut self, bump: u8) -> Result<()> {
        self.pool.set_inner(Pool {
            authority: self.authority.key(),
            mint: self.mint.key(),
            vault: self.pool_vault.key(),
            bump,
            reward_rate: 1,
            total_staked: 0,
        });

        Ok(())
    }
}

pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
    let min_balance: u64 = 10_000_000; // 0.01 SOL in lamports
    let authority_balance = ctx.accounts.authority.lamports();

    require!(
        authority_balance >= min_balance,
        StakingError::InsufficientSolBalance
    );

    let bump = ctx.bumps.pool;
    ctx.accounts.populate_pool(bump)?;

    Ok(())
}
