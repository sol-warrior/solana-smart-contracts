use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct Pool {
    pub authority: Pubkey, // admin of pool (creator)
    pub mint: Pubkey,      // USDC mint
    pub vault: Pubkey,     // Token vault account address
    pub bump: u8,          // PDA bump
    pub reward_rate: u64,  // how many pts per USDC per day
    pub total_staked: u64, // total amount staked by everyone
}

#[derive(InitSpace)]
#[account(discriminator = 2)]
pub struct UserStake {
    pub user: Pubkey,           // wallet owner
    pub amount: u64,            // total staked amount
    pub staked_at: i64,         // first time staked
    pub last_claim: i64,        // last time points were updated
    pub points: u64,            // stored points earned so far
    pub user_vault_ata: Pubkey, // user vault token account
    pub bump: u8,
}
