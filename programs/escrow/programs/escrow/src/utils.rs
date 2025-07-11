 // }
    //    pub fn request_revision(ctx: Context<RequestRevision>) -> Result<()> {
    //     let escrow = &mut ctx.accounts.escrow;
        
    //     require!(escrow.status == EscrowStatus::Submitted as u8, EscrowError::InvalidState);
    //     require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
        
    //     escrow.status = EscrowStatus::Active as u8;
    //     escrow.revision_requests = escrow.revision_requests.checked_add(1).ok_or(EscrowError::Overflow)?;
        
    //     Ok(())
    // }

    // pub fn raise_dispute(
    //     ctx: Context<RaiseDispute>,
    //     evidence_uri_hash: [u8; 32],
    // ) -> Result<()> {
    //     let escrow = &mut ctx.accounts.escrow;
    //     let caller = ctx.accounts.caller.key();
        
    //     require!(
    //         escrow.status == EscrowStatus::Active as u8 || 
    //         escrow.status == EscrowStatus::Submitted as u8,
    //         EscrowError::InvalidState
    //     );
    //     require!(
    //         caller == escrow.maker || caller == escrow.taker,
    //         EscrowError::Unauthorized
    //     );
        
    //     escrow.status = EscrowStatus::Disputed as u8;
    //     escrow.dispute_evidence_uri_hash = evidence_uri_hash;
        
    //     Ok(())
    // }

    // pub fn arbiter_resolve(
    //     ctx: Context<ArbiterResolve>,
    //     taker_amount: u64,
    //     maker_amount: u64,
    // ) -> Result<()> {
    //     let escrow = &mut ctx.accounts.escrow;
    //     let total = taker_amount.checked_add(maker_amount).ok_or(EscrowError::Overflow)?;
    //     let available = escrow.amount_total - escrow.amount_released - escrow.amount_refunded;
        
    //     require!(escrow.status == EscrowStatus::Disputed as u8, EscrowError::InvalidState);
    //     require!(*ctx.accounts.arbiter.key == escrow.arbiter, EscrowError::Unauthorized);
    //     require!(total <= available, EscrowError::InvalidAmount);
        
    //     if taker_amount > 0 {
    //         Self::transfer_from_escrow(
    //             escrow,
    //             ctx.accounts.escrow.to_account_info(),
    //             ctx.accounts.taker.to_account_info(),
    //             taker_amount,
    //             ctx.accounts.system_program.to_account_info(),
    //         )?;
    //         escrow.amount_released = escrow.amount_released.checked_add(taker_amount).unwrap();
    //     }
        
    //     if maker_amount > 0 {
    //         Self::transfer_from_escrow(
    //             escrow,
    //             ctx.accounts.escrow.to_account_info(),
    //             ctx.accounts.maker.to_account_info(),
    //             maker_amount,
    //             ctx.accounts.system_program.to_account_info(),
    //         )?;
    //         escrow.amount_refunded = escrow.amount_refunded.checked_add(maker_amount).unwrap();
    //     }
        
    //     escrow.status = EscrowStatus::Completed as u8;
    //     escrow.completed_at = Clock::get()?.unix_timestamp;
        
    //     Ok(())
    // }

    // pub fn cancel_before_start(ctx: Context<CancelBeforeStart>) -> Result<()> {
    //     let escrow = &mut ctx.accounts.escrow;
        
    //     require!(escrow.status == EscrowStatus::Active as u8, EscrowError::InvalidState);
    //     require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
    //     require!(escrow.amount_released == 0, EscrowError::FundsAlreadyReleased);
        
    //     let amount = escrow.amount_total - escrow.amount_refunded;
    //     require!(amount > 0, EscrowError::NoFundsAvailable);
        
    //     Self::transfer_from_escrow(
    //         escrow,
    //         ctx.accounts.escrow.to_account_info(),
    //         ctx.accounts.maker.to_account_info(),
    //         amount,
    //         ctx.accounts.system_program.to_account_info(),
    //     )?;
        
    //     escrow.amount_refunded += amount;
    //     escrow.status = EscrowStatus::Cancelled as u8;
        
    //     Ok(())
    // }

    // pub fn claim_timeout(ctx: Context<ClaimTimeout>) -> Result<()> {
    //     let escrow = &mut ctx.accounts.escrow;
    //     let clock = Clock::get()?;
    //     let current_time = clock.unix_timestamp;
        
    //     match escrow.status {
    //         s if s == EscrowStatus::Active as u8 && current_time > escrow.deadline => {
    //             require!(
    //                 *ctx.accounts.claimant.key == escrow.maker,
    //                 EscrowError::Unauthorized
    //             );
                
    //             let amount = escrow.amount_total - escrow.amount_refunded;
    //             require!(amount > 0, EscrowError::NoFundsAvailable);
                
    //             Self::transfer_from_escrow(
    //                 escrow,
    //                 ctx.accounts.escrow.to_account_info(),
    //                 ctx.accounts.claimant.to_account_info(),
    //                 amount,
    //                 ctx.accounts.system_program.to_account_info(),
    //             )?;
                
    //             escrow.amount_refunded += amount;
    //             escrow.status = EscrowStatus::Cancelled as u8;
    //         }
    //         s if s == EscrowStatus::Submitted as u8 && current_time > escrow.auto_release_at => {
    //             require!(
    //                 *ctx.accounts.claimant.key == escrow.taker,
    //                 EscrowError::Unauthorized
    //             );
                
    //             let amount = escrow.amount_total - escrow.amount_released;
    //             require!(amount > 0, EscrowError::NoFundsAvailable);
                
    //             Self::transfer_from_escrow(
    //                 escrow,
    //                 ctx.accounts.escrow.to_account_info(),
    //                 ctx.accounts.claimant.to_account_info(),
    //                 amount,
    //                 ctx.accounts.system_program.to_account_info(),
    //             )?;
                
    //             escrow.amount_released += amount;
    //             escrow.status = EscrowStatus::Completed as u8;
    //             escrow.completed_at = current_time;
    //         }
    //         _ => return Err(EscrowError::ClaimNotAvailable.into()),
    //     }
        
    //     Ok(())
    // }


#[derive(Accounts)]
pub struct RequestRevision<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct RaiseDispute<'info> {
    pub caller: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct ArbiterResolve<'info> {
    pub arbiter: Signer<'info>,
    /// CHECK: Maker account for SOL transfer
    #[account(mut)]
    pub maker: AccountInfo<'info>,
    /// CHECK: Taker account for SOL transfer
    #[account(mut)]
    pub taker: AccountInfo<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelBeforeStart<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTimeout<'info> {
    pub claimant: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
    pub system_program: Program<'info, System>,
}
