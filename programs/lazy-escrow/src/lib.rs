use anchor_lang::prelude::*;

declare_id!("G12PpJUib62fdSGLveZxVTZnr9wnmF2bXsfY7TFPDAxX");

mod state;
mod instructions;

use instructions::*;

#[program]
pub mod lazy_escrow {
    use super::*;

    pub fn make(ctx: Context<Maker>, amount_a: u64, amount_b: u64) -> Result<()> {
        ctx.accounts.init_escrow(amount_a, amount_b, &ctx.bumps)?;
        ctx.accounts.transfer_token_a()
    }

    pub fn take(ctx: Context<Taker>) -> Result<()> {
        ctx.accounts.transfer_token_b()?;
        ctx.accounts.transfer_token_a()?;
        ctx.accounts.close_vault()
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_to_maker()?;
        ctx.accounts.close_vault()
    }
}
