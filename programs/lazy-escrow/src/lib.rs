use anchor_lang::prelude::*;

declare_id!("G12PpJUib62fdSGLveZxVTZnr9wnmF2bXsfY7TFPDAxX");

mod state;
mod instructions;

use state::*;
use instructions::*;

#[program]
pub mod lazy_escrow {
    use super::*;

    pub fn make(ctx: Context<Maker>) -> Result<()> {
        
        //let mut my_escrow = ctx.accounts.escrow.load_mut()?;


        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_to_maker()?;
        ctx.accounts.close_vault()
    }
}
