use anchor_lang::prelude::*;

pub mod error;

use error::ErrorCode;

declare_id!("63GkjuQiomySLzktPFgnzKH843N5vvpSBWMoUQDhZr5p");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()>{
        require!(amount > 0, ErrorCode::InvalidAmount);
        require!(!ctx.accounts.escrow_account.is_initialized, ErrorCode::AlreadyInitialized);

        let escrow_account = &mut ctx.accounts.escrow_account;
        escrow_account.amount = amount;
        escrow_account.is_initialized = true;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 1)]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct EscrowAccount{
    pub amount: u64,
    pub is_initialized: bool,
}