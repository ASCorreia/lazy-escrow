pub use anchor_lang::prelude::*;
pub use anchor_lang::accounts::lazy_account::LazyAccount;
pub use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{Mint, Token, TokenAccount, Transfer, transfer},
};

use crate::state::{Escrow, LazyEscrow};

#[derive(Accounts)]
pub struct Taker<'info>{
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: Signer<'info>,
    pub token_a: Account<'info, Mint>,
    pub token_b: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
    )]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = token_b,
        associated_token::authority = maker,
    )]
    pub ata_maker_token_b: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_a,
        associated_token::authority = taker,
    )]
    pub ata_taker_token_a: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_b,
        associated_token::authority = taker,
    )]
    pub ata_taker_token_b: Account<'info, TokenAccount>,
    #[account(
        mut,
        //has_one = 
    )]
    pub escrow: LazyAccount<'info, Escrow>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Taker<'info> {

    pub fn transfer_token_b(&mut self) -> Result<()> {

        //let cpi_program = self.

        //let cpi_accounts = 

        Ok(())
    }

}