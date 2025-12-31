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
pub struct UnstakeAll<'info> {
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
        close= user,
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

impl<'info> UnstakeAll<'info> {
    fn withdraw_all(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &self.pool;
        let user_stake = &mut self.user_stake;
        let amount = user_stake.amount;

        //calculate user's stake points
        let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
        if elapsed_secs > 0 {
            let earned = ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate;
            user_stake.points += earned;
        }

        //Transfer token back to user
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

        //update state
        user_stake.amount = 0;
        user_stake.last_claim = clock.unix_timestamp;
        user_stake.points = 0;

        self.pool.total_staked -= amount;

        Ok(())
    }
}
pub fn unstake_all(ctx: Context<UnstakeAll>) -> Result<()> {
    require!(
        ctx.accounts.user_stake.amount > 0,
        StakingError::InsufficientTokenBalance
    );

    ctx.accounts.withdraw_all()?;

    Ok(())
}
