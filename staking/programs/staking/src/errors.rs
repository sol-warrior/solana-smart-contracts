use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("User does not have enough stake to unstake this amount")]
    InsufficientStake,

    #[msg("Creator does not have enough sol for initialize the pool")]
    InsufficientSolBalance,

    #[msg("Invalid authority")]
    InvalidAuthority,

    #[msg("Invalid mint address")]
    InvalidMint,

    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,

    #[msg("Invalid staking amount")]
    InvalidStakingAmount,

    #[msg("Invalid unstaking amount")]
    InvalidUnStakingAmount,
}
