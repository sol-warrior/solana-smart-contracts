use anchor_lang::prelude::*;

declare_id!("9qXFP6JkCQrTMaGBsMEEitFvaoGYqL4VK4mEYb5WFypi");
mod constants;
mod errors;
mod instructions;
mod state;

pub use instructions::*;

#[program]
pub mod amm {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        authority: Pubkey,
        fee: u16,
        init_m_amount: u64,
        init_n_amount: u64,
    ) -> Result<()> {
        instructions::initialize(ctx, seed, authority, fee, init_m_amount, init_n_amount)
    }
}
