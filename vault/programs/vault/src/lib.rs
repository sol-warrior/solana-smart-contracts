use anchor_lang::prelude::*;

declare_id!("DJktBCt1vV8jdYmyxqnH8oNVa5PMLWgkc7WuT4KB1Q8o");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
