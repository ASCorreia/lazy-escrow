pub use anchor_lang::prelude::*;
pub use anchor_lang::accounts::lazy_account::LazyAccount;
pub use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{
        Mint, 
        Token, 
        TokenAccount, 
        Transfer, 
        transfer, 
        CloseAccount, 
        close_account
    },
};

use crate::state::{
    Escrow, 
    LazyEscrow
};

#[derive(Accounts)]
pub struct Taker<'info>{
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
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
        payer = taker,
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
        close = maker,
        seeds = [b"escrow", maker.key().as_ref()],
        bump = *(escrow.load_bump()?),
    )]
    pub escrow: LazyAccount<'info, Escrow>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Taker<'info> {

    pub fn transfer_token_b(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.ata_taker_token_b.to_account_info(),
            to: self.ata_maker_token_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let amount_b = self.escrow.load_amount_b()?;

        transfer(cpi_ctx, *amount_b)?;

        Ok(())
    }

    pub fn transfer_token_a(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault_token_a.to_account_info(),
            to: self.ata_taker_token_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &[*(self.escrow.load_bump()?)],
        ]];  

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);        

        let amount_a = self.escrow.load_amount_a()?;

        transfer(cpi_ctx, *amount_a)?;

        Ok(())
    }

    pub fn close_vault(&mut self) -> Result<()> {

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = CloseAccount {
            account: self.vault_token_a.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &[*(self.escrow.load_bump()?)],
        ]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

        close_account(cpi_ctx)?;

        Ok(())
    }

}