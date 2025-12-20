use anchor_lang::prelude::*;
mod errors;
mod instructions;
mod state;
use instructions::*;
declare_id!("7ttjDw7MEBJqCaxNc33GozzyH4EHmisaenM8QoyyU4c9");

#[program]
pub mod escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn maker(
        ctx: Context<Maker>,
        seed: u64,
        token_mint_n_expected: u64,
        amount: u64,
    ) -> Result<()> {
        maker::handler(ctx, seed, token_mint_n_expected, amount)
    }

    #[instruction(discriminator = 1)]
    pub fn taker(ctx: Context<Taker>) -> Result<()> {
        taker::handler(ctx)
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        refund::handler(ctx)
    }
}
