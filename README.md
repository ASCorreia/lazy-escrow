# Lazy Escrow

This program implements an Escrow contract to demonstrate the use of new Anchor 0.31.0 features, namely:
- **Custom discriminators**: Discriminators of 1-byte are used to override the default 8-byte discriminators.
- **LazyAccount**: Experimental account type that allows deserialization of individual fields.

The Escrow is a solana program which will hold on to the assets until a condition is met. There will be a user (`maker`) who defines the agreement conditions for the transaction: initiating the escrow and depositing a given amount of a given token (in this case, `amount_a` of `token_a`) to the vault owned by our program in exchange for an amount of tokens (in this case, `amount_b` of `token_b`). Now any user (`taker`) can take up their offer and deposit the amount expected by the maker and receive the tokens from the vault to their account atomically. So this is how we achieve a trustless conditional transfer.

---

## Let's walk through the architecture:

For this program, we will have the Escrow state account that consists of:

```rust
#[account(discriminator = 1)]
pub struct Escrow {
    pub maker: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub bump: u8,
}
```

### In this state account, we will store:

- `maker`: The user that will initiate the escrow.

- `token_a`: The token that the maker is trading with.

- `token_b`: The token that the maker is trading for.

- `amount_a`: The amount of token_a that the maker is trading.

- `amount_b`: The amount of token_b that the maker wants to receive.

- `bump`: Since our Escrow account will be a PDA (Program Derived Address), we will store the bump of the account.

The discriminator for this state account will be customized using the attribute macro  `#[account(discriminator = 1)]`, overriding the default 8-byte discriminator, to save resources and use only 1-byte. The discriminator needs to be unique for each type implemented, in this case only one is defined.

---

### The maker will be able to define the deal conditions. For that, we create the following context:

```rust
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
```

Let´s have a closer look at the accounts that we are passing in this context:

- `maker`: The person that is creating the escrow. Will be a signer of the transaction, and we mark this account as mutable as we will be deducting lamports from this account.

- `token_a`: The Mint Account that represents the asset to be sent by the maker (and received by the taker).

- `token_b`: The Mint Account that represents the asset to be received by the maker (and sent by the taker).

- `ata_maker_token_a`:  The Associated Token Account that holds the token_a of the maker. This will be mutable as the assets are being transferred from this account.

- `vault_token_a`: This is an Associated Token Account to hold the token_a transferred from the maker, and hold by the escrow agent, until the agreement is completed and the assets are either sent to the taker or refunded back to the maker. This account will be created and payed by the maker, but the escrow will hold the authority on the funds.

- `escrow`: The Escrow account will hold the state of the exchange agreement that we will initialize and that will be payed by the maker. We derive the Escrow PDA from the byte representation of the word "escrow" and the reference of the user public key. Anchor will calculate the canonical bump (the first bump that throws that address out of the ed25519 eliptic curve) and save it for us in a struct. In this example, we will be using the `LazyAccount` as the account type.

- `token_program`: The associated token program.

- `associated_token_program`: The token program.

- `system_program`: The system program. Program responsible for the initialization of any new account.

### We then implement some functionality for our Make context:

```rust
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
```

In the `init_escrow` function, we initialize the escrow account. In this case, as we are using the `LazyAccount`, we need to use the `load_mut` method to deserialize the account and set the data content.

In the `transfer_token_a` function, we transfer tokens from the maker's associated token account to the vault account. In this case, for reading a individual data field in the escrow account, we use the `load_<field>` method of the `LazyAccount`. The lazy account allow us to not load the entire account, saving us some memory and compute units.

---

### The taker can then take the open offer and deposit the amount expected by the maker and receive the tokens from the vault to their account. For that, we create the following context:

```rust
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
```

In this context, we are passing all the accounts that we need to transfer the token_b from the taker to the maker, transfer the token_a from the vault to the taker and close accounts:

- `taker`: The account of the person that is accepting the exchange proposed by the maker in the escrow.

- `maker`: The account of the person that initialized the escrow.

- `token_a`: The mint of the token the maker is depositing and the taker is receiving.

- `token_b`: The mint of the token the maker is receiving and the taker is sending.

- `vault_token_a`: The vault account that currently holds the tokens of token_a until the condition is met. Mutable because its funds are being transferred to the taker.

- `ata_maker_token_b`: The ATA of the maker for the token_b. This account may not exist yet, so the program will need to initialize it with 'init-if-needed'.

- `ata_taker_token_a`: The ATA of the taker for the token_a. This account may not exist yet, so the program will need to initialize it with 'init-if-needed'.

- `ata_taker_token_b`: The ATA of the taker from which the tokens of token_b is being transferred from. It needs to be mutable.

- `escrow`: The Escrow account that holds the state of the exchange agreement. In this example, we will be using the `LazyAccount` as the account type. Upon closure the rent is being transferred back to the maker.

- `token_program`: The associated token program.

- `associated_token_program`: The token program.

- `system_program`: The system program.

### We then implement some functionality for our Take context:

```rust
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
```

In the `transfer_token_b` function, we transfer the token_b tokens from the taker's associated token account to the maker's associated token account.

In the `transfer_token_a` function, we transfer the token_a tokens from the vault account to the taker's associated token account. Given that the authority of the vault is the escrow, we need to pass the seeds while defining the context for the CPI.

And then, in the `close_vault`, we close the vault account and rent is claimed by the maker. Since the transfer occurs from a PDA, we need to pass the seeds while defining the context for the CPI.

In these functions, we use the `load_<field>` method of the `LazyAccount` to access (read only) individual data fields of the escrow account.

---

### The maker of an escrow can be refunded of the tokens that are in the vault and close the escrow account, if the exchange did not occur yet. For that, we create the following context:

```rust
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub token_a: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = maker,
    )]
    pub ata_maker_token_a: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = escrow,
    )]
    pub vault_token_a: Account<'info, TokenAccount>,
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
```

In this context, we are passing all the accounts that we need to refund the funds and close the escrow account:

- `maker`: The account that is refunding the funds and closing the escrow account.

- `token_a`: The mint of the token the maker has deposited on the vault.

- `ata_maker_token_a`:  The Associated Token Account of the maker for the token_a. Mutable because it will receive the funds back.

- `vault_token_a`: The vault account that currently holds the tokens of token_a until the condition is either met or canceled (refunded). Mutable because its funds are being transferred back to the maker.

- `escrow`: The Escrow account that holds the state of the exchange agreement. In this example, we will be using the `LazyAccount` as the account type. Upon closure the rent is being transferred back to the maker.

- `token_program`: The associated token program.

- `associated_token_program`: The token program.

- `system_program`: The system program.

### We then implement some functionality for our Refund context:

```rust
impl<'info> Refund<'info> {
    pub fn refund_to_maker(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault_token_a.to_account_info(),
            to: self.ata_maker_token_a.to_account_info(),
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
        let close_accounts = CloseAccount {
            account: self.vault_token_a.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &[*(self.escrow.load_bump()?)],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds,
        );

        close_account(ctx)
    }
}
```

In the `refund_to_maker` function, we transfer the tokens from the vault account to the maker's associated token account.

And then, in the `close_vault`, we close the vault account and rent is claimed by the maker. Since the transfer occurs from a PDA, we need to pass the seeds while defining the context for the CPI.

In both functions, we use the `load_<field>` method of the `LazyAccount` to access (read only) individual data fields of the escrow account.

---

### The instructions of the program are also using custom discriminators as demonstrated below:

```rust
#[program]
pub mod lazy_escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Maker>, amount_a: u64, amount_b: u64) -> Result<()> {
        ctx.accounts.init_escrow(amount_a, amount_b, &ctx.bumps)?;
        ctx.accounts.transfer_token_a()
    }

    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Taker>) -> Result<()> {
        ctx.accounts.transfer_token_b()?;
        ctx.accounts.transfer_token_a()?;
        ctx.accounts.close_vault()
    }

    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_to_maker()?;
        ctx.accounts.close_vault()
    }
}
```

In this case, we will use 1-byte discriminators for the instructions in the lib.rs, overriding the default 8-byte.

