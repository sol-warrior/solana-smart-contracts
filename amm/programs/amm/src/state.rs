use anchor_lang::prelude::*;

#[account(discriminator = 1)]
#[derive(InitSpace)]
pub struct Config {
    pub seed: u64,
    pub authority: Pubkey,
    pub mint_m: Pubkey, // Mint address for token M
    pub mint_n: Pubkey, // Mint address for token N
    pub fee: u16,       // Fee charged on swaps, in basis points (1/100th of a percent)
    pub locked: bool,
    pub bump_lp: u8,
    pub bump: u8,
}
