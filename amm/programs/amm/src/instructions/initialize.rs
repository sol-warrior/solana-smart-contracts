use anchor_lang::prelude::*;
use anchor_spl::token_2022::{Token2022, spl_token_2022};
use anchor_spl::token_interface::{Mint as InterfaceMint, TokenAccount as InterfaceTokenAccount, transfer, Transfer};
use anchor_spl::token::{Mint, TokenAccount, Token, mint_to, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use crate::errors::AmmError;
use crate::state::Config;
use crate::constants::MINIMUM_LIQUIDITY;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint_m: Box<InterfaceAccount<'info, InterfaceMint>>,
    pub mint_n: Box<InterfaceAccount<'info, InterfaceMint>>,
    #[account(
        init,
        seeds = [b"liquiditypool", config.key.as_ref()],
        payer = initializer,
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_m,
        associated_token::authority = config,
    )]
    pub vault_m: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_n,
        associated_token::authority = config,
    )]
    pub vault_n: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_m,
        associated_token::authority = initializer,
    )]
    pub initializer_m: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_n,
        associated_token::authority = initializer,
    )]
    pub initializer_n: Box<InterfaceAccount<'info, InterfaceTokenAccount>>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint = mint_lp,
        associated_token::authority = initializer,
    )]
    pub initializer_lp: Box<Account<'info, TokenAccount>>,
    #[account(
        init, 
        payer = initializer, 
        seeds = [b"config", seed.to_le_bytes().as_ref(), mint_m.key().as_ref(), mint_n.key().as_ref()], 
        bump,
        space =  Config::DISCRIMINATOR.len() + Config::INIT_SPACE
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Option<Program<'info, Token2022>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_config(
        &mut self,
        seed: u64,
        authority: Pubkey,
        fee: u16,
        bumps: &InitializeBumps
    ) -> Result<()> {
        self.config.set_inner(
            Config {
                seed,
                authority,
                mint_m: self.mint_m.key(),
                mint_n: self.mint_n.key(),
                fee,
                locked: false,
                bump_lp: bumps.mint_lp,
                bump: bumps.config
            }
        );

        Ok(())
    }

    pub fn deposit_tokens(
        &self,
        is_x: bool,
        amount: u64,
    ) -> Result<()> {
        let (mint, from, to) = match is_x {
            true => (
                self.mint_m.to_account_info(),
                self.initializer_m.to_account_info(),
                self.vault_m.to_account_info(),
            ),
            false => (
                self.mint_n.to_account_info(),
                self.initializer_n.to_account_info(),
                self.vault_n.to_account_info(),
            )
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.initializer.to_account_info(),
        };

        let program = if *mint.owner == spl_token_2022::ID {
            self.token_2022_program.as_ref().ok_or(AmmError::InvalidToken)?.to_account_info()
        } else {
            self.token_program.to_account_info()
        };

        transfer(CpiContext::new(program, cpi_accounts), amount)
    }

    pub fn mint_initial_lp(
        &self,
        liquidity: u64,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        let seed_bytes = self.config.seed.to_le_bytes();
        let mint_m_bytes = self.mint_m.key().to_bytes();
        let mint_n_bytes = self.mint_n.key().to_bytes();

        let seeds: &[&[u8]] = &[
            b"config".as_ref(),
            seed_bytes.as_ref(),
            mint_m_bytes.as_ref(),
            mint_n_bytes.as_ref(),
            &[bumps.config],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.initializer_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            ),
            liquidity,
        )
    }
}

/// Computes the integer square root of a number using Newton's approximation method.
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

pub fn initialize(
    ctx: Context<Initialize>,
    seed: u64,
    authority: Pubkey,
    fee: u16,
    init_m_amount: u64,
    init_n_amount: u64,
) -> Result<()> {
    require!(fee <= 10000, AmmError::InvalidFee);

    require!(init_m_amount > 0, AmmError::InvalidAmount);
    require!(init_n_amount > 0, AmmError::InvalidAmount);

    // Set up the pool configuration before proceeding (this is required for generating signer seeds)
    ctx.accounts.initialize_config(seed, authority, fee, &ctx.bumps)?;

    // Transfer initial liquidity funds from the initializer to each vault (token M and token N)
    // This will move init_m_amount of token M and init_n_amount of token N from the user to the pool's vaults
    // Deposits must succeed before proceeding to LP minting logic
    ctx.accounts.deposit_tokens(true, init_m_amount)?;
    ctx.accounts.deposit_tokens(false, init_n_amount)?;

    // Compute the amount of LP tokens to mint for the initial liquidity provider.
    // Formula: liquidity = sqrt(init_m_amount * init_n_amount)
    // Initial LP tokens minted: liquidity - MINIMUM_LIQUIDITY
    let product = (init_m_amount as u128)
        .checked_mul(init_n_amount as u128)
        .ok_or(AmmError::Overflow)?;
    let liquidity = integer_sqrt(product) as u64;

    // Ensure the pool has at least MINIMUM_LIQUIDITY to be locked for fairness
    require!(liquidity > MINIMUM_LIQUIDITY, AmmError::InsufficientInitialLiquidity);

    let lp_to_mint = liquidity
        .checked_sub(MINIMUM_LIQUIDITY)
        .ok_or(AmmError::Overflow)?;

    // Mint LP tokens to the user who provided the initial liquidity.
    // A small, fixed amount (MINIMUM_LIQUIDITY) is permanently locked to make sure the pool stays balanced and fair,
    // so the user receives the rest of the tokens.
    ctx.accounts.mint_initial_lp(lp_to_mint, &ctx.bumps)?;

    msg!("Initialized pool with total liquidity: {}, user minted: {}, protocol locked: {}", 
         liquidity, lp_to_mint, MINIMUM_LIQUIDITY);

    Ok(())
}