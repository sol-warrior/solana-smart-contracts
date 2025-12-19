use anchor_lang::prelude::*;

declare_id!("7ttjDw7MEBJqCaxNc33GozzyH4EHmisaenM8QoyyU4c9");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
