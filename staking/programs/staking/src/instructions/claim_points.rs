use crate::{
    errors::StakingError,
    state::{Pool, UserStake},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    #[account()]
    pub user: Signer<'info>, // wallet claiming points

    #[account(
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump= user_stake.bump,
        has_one= user @ StakingError::InvalidAuthority,
    )]
    pub user_stake: Account<'info, UserStake>,
}

pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
    let clock = Clock::get()?;
    let pool = &ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;

    if user_stake.amount == 0 {
        return Ok(());
    }

    // Calculate earned since last claim
    let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
    if elapsed_secs > 0 {
        let earned = ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate;
        user_stake.points += earned;
    }

    // Reset claim window
    user_stake.last_claim = clock.unix_timestamp;

    Ok(())
}
