use anchor_lang::prelude::*;

#[error_code]
pub enum UserVaultError {
    #[msg("Cannot initialize, vault is already initialized")]
    VaultAlreadyInitialized,

    #[msg("Cannot deposit, you are not owner of this vault")]
    NotVaultOwner,

    #[msg("Cannot deposit, you are not authorised for this wallet account")]
    NotAuthorised,
 
    #[msg("Math calculation overflow/underflow error")]
    MathOverflow,

    #[msg("Invalid deposit amount")]
    InvalidAmount,

    #[msg("Insufficient funds in the vault")]
    InsufficientFunds,
}