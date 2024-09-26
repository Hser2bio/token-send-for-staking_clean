use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("5YFYMJ6zQyNzWDxXyDtA2nfJrPXnQRUaWkVyuAYencLw");

#[program]
pub mod token_lock {
    use super::*;

    pub fn init_admin(
        ctx: Context<InitAdmin>,
        start_date: u64,
        lock_period: u64,
        lock_amount: u64,
    ) -> Result<()> {
        ctx.accounts.admin_state.admin = *ctx.accounts.admin.key;
        ctx.accounts.admin_state.token_recipient =
            *ctx.accounts.token_recipient.to_account_info().key;
        ctx.accounts.admin_state.token_mint = ctx.accounts.token_mint.key();
        ctx.accounts.admin_state.start_date = start_date;
        ctx.accounts.admin_state.lock_period = lock_period;
        ctx.accounts.admin_state.lock_amount = lock_amount;

        Ok(())
    }

    pub fn update_admin(
        ctx: Context<UpdateAdmin>,
        start_date: u64,
        lock_period: u64,
        lock_amount: u64,
    ) -> Result<()> {
        ctx.accounts.admin_state.admin = *ctx.accounts.new_admin.key;
        ctx.accounts.admin_state.token_recipient =
            *ctx.accounts.new_token_recipient.to_account_info().key;
        ctx.accounts.admin_state.start_date = start_date;
        ctx.accounts.admin_state.lock_period = lock_period;
        ctx.accounts.admin_state.lock_amount = lock_amount;

        Ok(())
    }

    pub fn send_token(ctx: Context<SendToken>) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        let amount = ctx.accounts.admin_state.lock_amount / ctx.accounts.admin_state.lock_period
            * (current_timestamp as u64 - ctx.accounts.admin_state.start_date);

        let bump = *ctx.bumps.get("vault").unwrap();
        let token_mint_key = ctx.accounts.token_mint.key();
        let pda_sign = &[&b"vault"[..], token_mint_key.as_ref(), &[bump]];

        token::transfer(
            ctx.accounts
                .from_pda_transfer_to_user()
                .with_signer(&[pda_sign]),
            amount,
        )?;

        ctx.accounts.admin_state.start_date = current_timestamp as u64;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(start_date: u64, lock_period: u64, lock_amount: u64)]
pub struct InitAdmin<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        token::mint = token_mint,
    )]
    pub token_recipient: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    // create vault to lock tokens
    #[account(
        init,
        seeds = [b"vault".as_ref(), token_mint.key().as_ref()],
        bump,
        payer = admin,
        token::mint = token_mint,
        token::authority = vault,
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
         init,
         seeds = [b"state".as_ref(), b"admin".as_ref()],
         bump,
         payer = admin,
         space = AdminState::space()
     )]
    pub admin_state: Box<Account<'info, AdminState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(start_date: u64, lock_period: u64, lock_amount: u64)]
pub struct UpdateAdmin<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub new_admin: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        token::mint = admin_state.token_mint,
    )]
    pub new_token_recipient: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = admin_state.admin == *admin.key,
        seeds = [b"state".as_ref(), b"admin".as_ref()],
        bump
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
}

#[derive(Accounts)]
pub struct SendToken<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"state".as_ref(), b"admin".as_ref()],
        bump
    )]
    pub admin_state: Box<Account<'info, AdminState>>,
    #[account(
        mut,
        seeds = [b"vault".as_ref(), token_mint.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = admin_state.token_mint == *token_mint.to_account_info().key,
    )]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = token_mint,
    )]
    pub token_recipient: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct AdminState {
    pub admin: Pubkey,
    pub token_recipient: Pubkey, // token will be sent to this address
    pub token_mint: Pubkey,      // target token address users wants to buy
    pub start_date: u64,
    pub lock_period: u64,
    pub lock_amount: u64,
}

impl AdminState {
    pub fn space() -> usize {
        8 + 32 + 32 + 32 + 8 + 8 + 8
    }
}

impl<'info> SendToken<'info> {
    fn from_pda_transfer_to_user(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.token_recipient.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}
