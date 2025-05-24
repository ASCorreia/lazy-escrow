pub use anchor_lang::prelude::*;

#[account(discriminator = 1)]
pub struct Escrow {
    pub maker: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub bump: u8,
}

impl Space for Escrow {
    const INIT_SPACE: usize = 1 + 32 + 32 + 32 + 8 + 8 + 1;
}