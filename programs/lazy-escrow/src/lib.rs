use anchor_lang::prelude::*;

declare_id!("G12PpJUib62fdSGLveZxVTZnr9wnmF2bXsfY7TFPDAxX");

mod state;
mod instructions;

#[program]
pub mod lazy_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
