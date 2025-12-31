use crate::{
    errors::StakingError,
    state::{Pool, UserStake},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GetPoints<'info> {
    #[account()]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump= user_stake.bump,
        has_one= user @ StakingError::InvalidAuthority,
    )]
    pub user_stake: Account<'info, UserStake>,
    // The `system_program` account is not required here because none of the logic in `get_points` uses
    //or invokes the system program. No account creations, transfers,
    //or CPI calls involving the system program are performed in this instruction, so it can be safely omitted.
}

pub fn get_points(ctx: Context<GetPoints>) -> Result<u64> {
    let clock = Clock::get()?;
    let pool = &ctx.accounts.pool;
    let user_stake = &ctx.accounts.user_stake;

    if user_stake.amount == 0 {
        return Ok(user_stake.points);
    }

    let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
    let new_points = if elapsed_secs > 0 {
        ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate
    } else {
        0
    };

    Ok(user_stake.points + new_points)
}
