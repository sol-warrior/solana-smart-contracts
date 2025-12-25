use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("2t4tTkX9nEKuTHdZbVYYXwq188GBQQq35fh46xV1qBuw");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        pool.authority = ctx.accounts.authority.key();
        pool.mint = ctx.accounts.mint.key();
        pool.vault = ctx.accounts.pool_vault.key();
        pool.bump = ctx.bumps.pool;
        pool.reward_rate = 1; // 1 point per USDC per day
        pool.total_staked = 0;

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // 1. if user stakes again, compute points earned before updating
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
            user_stake.user = ctx.accounts.user.key();
        }

        // 2. transfer USDC from user ATA -> pool vault
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        // 3. update state
        user_stake.amount += amount;
        pool.total_staked += amount;

        Ok(())
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

    pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // If user never staked or has no amount, do nothing
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

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;

        require!(user_stake.amount >= amount, CustomError::InsufficientStake);

        // 1. compute earned points since last claim
        let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
        if elapsed_secs > 0 {
            let earned = ((elapsed_secs as u64) / 86400) * user_stake.amount * pool.reward_rate;
            user_stake.points += earned;
        }

        let old_amount = user_stake.amount;

        // 2. proportional reduction of points
        let deducted = user_stake.points * amount / old_amount;
        user_stake.points -= deducted;

        // 3. update accounting BEFORE transfer
        user_stake.amount -= amount;
        pool.total_staked -= amount;
        user_stake.last_claim = clock.unix_timestamp;

        // 4. token transfer back to user
        let pool_key = &pool.key();
        let bump = pool.bump;
        let seeds: &[&[u8]] = &[b"pool", pool.authority.as_ref(), &[bump]];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn unstake_all(ctx: Context<UnstakeAll>) -> Result<()> {
        let clock = Clock::get()?;
        let pool = &mut ctx.accounts.pool;
        let user_stake = &mut ctx.accounts.user_stake;

        let amount = user_stake.amount;
        require!(amount > 0, CustomError::InsufficientStake);

        // calculate earned points before exit
        let elapsed_secs = clock.unix_timestamp - user_stake.last_claim;
        if elapsed_secs > 0 {
            let earned = ((elapsed_secs as u64) / 86400) * amount * pool.reward_rate;
            user_stake.points += earned;
        }

        // update pool total
        pool.total_staked -= amount;

        // reset user stake data
        user_stake.amount = 0;
        user_stake.points = 0;
        user_stake.last_claim = clock.unix_timestamp;

        // transfer tokens back
        let seeds: &[&[u8]] = &[b"pool", pool.authority.as_ref(), &[pool.bump]];

        let signer_seeds = &[seeds];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: pool.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, amount)?;

        // close user stake account, rent refunded to user
        **ctx.accounts.user.try_borrow_mut_lamports()? += user_stake.to_account_info().lamports();
        **user_stake.to_account_info().try_borrow_mut_lamports()? = 0;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[account]
pub struct Pool {
    pub authority: Pubkey, // admin of pool (creator)
    pub mint: Pubkey,      // USDC mint
    pub vault: Pubkey,     // Token vault account address
    pub bump: u8,          // PDA bump
    pub reward_rate: u64,  // how many pts per USDC per day
    pub total_staked: u64, // total amount staked by everyone
}

#[account]
pub struct UserStake {
    pub user: Pubkey,    // wallet owner
    pub amount: u64,     // total staked amount
    pub staked_at: i64,  // first time staked
    pub last_claim: i64, // last time points were updated
    pub points: u64,     // stored points earned so far
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>, // pool creator

    pub mint: Account<'info, Mint>, // USDC Mint

    #[account(
        init,
        payer = authority,
        seeds = [b"pool", authority.key().as_ref()],
        bump,
        space = 8 + 32 + 32 + 32 + 1 + 8 + 8
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = pool
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump,
        space = 8 + 32 + 8 + 8 + 8 + 8
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>, // user USDC ATA

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct GetPoints<'info> {
    pub pool: Account<'info, Pool>,
    /// CHECK:
    pub user: UncheckedAccount<'info>,
    #[account(
        seeds = [b"user-stake", user_stake.user.as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>, // receive tokens back

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnstakeAll<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // wallet claiming points

    pub pool: Account<'info, Pool>, // global staking pool config

    #[account(
        mut,
        seeds = [b"user-stake", user.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>, // user-specific staking record
}

#[error_code]
pub enum CustomError {
    #[msg("User does not have enough stake to unstake this amount")]
    InsufficientStake,
}
