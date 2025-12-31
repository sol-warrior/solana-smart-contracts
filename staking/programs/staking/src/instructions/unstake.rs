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
pub struct Unstake<'info> {
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
        mut,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump= user_stake.bump,
        has_one= user @ StakingError::InvalidAuthority,
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

impl<'info> Unstake<'info> {
    fn update_user_state(&mut self, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &self.pool;
        let user_stake = &mut self.user_stake;

        // compute earned points since last claim
        let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
        if elapsed_secs > 0 {
            let earned = ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate;
            user_stake.points += earned;
        }

        let old_amount = user_stake.amount;

        // 2. proportional reduction of points
        let deducted = user_stake.points * amount / old_amount;
        user_stake.points -= deducted;

        user_stake.last_claim = clock.unix_timestamp;

        Ok(())
    }

    ///  Deposit usdc tokens from  pool_vault -> user
    fn deposit_tokens(&self, amount: u64) -> Result<()> {
        let bump = self.pool.bump;
        let seeds: &[&[u8]] = &[b"pool", self.pool.authority.as_ref(), &[bump]];
        let signer_seeds = &[seeds];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.pool_vault.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.user_ata.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
            self.mint.decimals,
        )?;

        Ok(())
    }
}

pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::InvalidUnStakingAmount);
    require!(
        amount <= ctx.accounts.user_stake.amount,
        StakingError::InsufficientTokenBalance
    );

    ctx.accounts.update_user_state(amount)?;
    ctx.accounts.deposit_tokens(amount)?;

    //update state
    ctx.accounts.user_stake.amount -= amount;
    ctx.accounts.pool.total_staked -= amount;

    Ok(())
}
