use crate::{
    errors::StakingError,
    state::{Pool, UserStake},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump,
        has_one = authority @ StakingError::InvalidAuthority,
        has_one = mint @ StakingError::InvalidMint,
    )]
    pub pool: Account<'info, Pool>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = pool,
        associated_token::token_program = token_program
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump,
        space = UserStake::DISCRIMINATOR.len() + UserStake::INIT_SPACE
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>, // user USDC ATA

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Stake<'info> {
    fn populate_user(&mut self, user_stake_bump: u8) -> Result<()> {
        let clock = Clock::get()?;

        let pool = &mut self.pool;
        let user_stake = &mut self.user_stake;

        // if user stakes again, compute points earned before updating
        if user_stake.amount > 0 {
            let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
            if elapsed_secs > 0 {
                let earned = ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate;
                user_stake.points += earned;
            }
        } else {
            // first time staking
            user_stake.staked_at = clock.unix_timestamp;
            user_stake.last_claim = clock.unix_timestamp;
            user_stake.points = 0;
            user_stake.user = self.user.key();
            user_stake.user_vault_ata = self.user_ata.key();
            user_stake.bump = user_stake_bump
        }
        Ok(())
    }

    ///  Deposit usdc tokens from user -> pool_vault
    fn deposit_tokens(&self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.user_ata.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.pool_vault.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            amount,
            self.mint.decimals,
        )?;

        Ok(())
    }
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::InvalidStakingAmount);
    require!(
        amount <= ctx.accounts.user_ata.amount,
        StakingError::InsufficientTokenBalance
    );

    ctx.accounts.populate_user(ctx.bumps.user_stake)?;
    ctx.accounts.deposit_tokens(amount)?;

    // //update state
    ctx.accounts.user_stake.amount += amount;
    ctx.accounts.pool.total_staked += amount;

    Ok(())
}
