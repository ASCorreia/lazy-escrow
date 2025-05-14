pub use anchor_lang::prelude::*;
pub use anchor_lang::accounts::lazy_account::LazyAccount;
pub use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        Mint, 
        Token, 
        TokenAccount, 
        Transfer, 
        transfer
    },
};

use crate::state::{
    Escrow, 
    LazyEscrow
};

#[derive(Accounts)]
pub struct Maker<'info>{
    #[account(mut)]
    pub maker: Signer<'info>,
    pub token_a: Account<'info, Mint>,
    pub token_b: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = maker,
    )]
    pub ata_maker_token_a: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
    )]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref()],
        bump,
    )]
    pub escrow: LazyAccount<'info, Escrow>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Maker<'info> {

    pub fn init_escrow(&mut self, amount_a: u64, amount_b: u64, bumps: &MakerBumps) -> Result<()> {

        let mut my_escrow = self.escrow.load_mut()?;

        my_escrow.maker = self.maker.key();
        my_escrow.token_a = self.token_a.key();
        my_escrow.token_b = self.token_b.key();
        my_escrow.amount_a = amount_a;
        my_escrow.amount_b = amount_b;
        my_escrow.bump = bumps.escrow;

        Ok(())
    }

    pub fn transfer_token_a(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.ata_maker_token_a.to_account_info(),
            to: self.vault_token_a.to_account_info(),
           
            authority: self.maker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let amount_a = self.escrow.load_amount_a()?;

        transfer(cpi_ctx, *amount_a)?;

        Ok(())
    }

}