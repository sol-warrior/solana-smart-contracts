use anchor_lang::prelude::*;
#[account]
#[derive(InitSpace)]
pub struct UserVault {
    pub user: Pubkey,
    pub user_vault_bump: u8,
    pub user_vault_lamports_bump: u8,
    pub total_deposit: u64,
    pub user_vault_lamports: Pubkey,
}
