use anchor_lang::prelude::*;

declare_id!("9qXFP6JkCQrTMaGBsMEEitFvaoGYqL4VK4mEYb5WFypi");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
