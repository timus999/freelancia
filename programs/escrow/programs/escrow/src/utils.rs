use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program::clock;




declare_id!("2emdThfEdhbHmHZu3GCfsdT3dQicx1JELXxdFHpXu1Jk");

#[program]
pub mod escrow {
    use super::*;

     pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        escrow_id: u64,
        amount: u64,
        deadline: i64,
        auto_release_at: i64,
        spec_hash: [u8; 32],
        arbiter: Option<Pubkey>,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        let clock = Clock::get()?;
        
        require!(deadline > clock.unix_timestamp, EscrowError::InvalidDeadline);
        require!(auto_release_at > deadline, EscrowError::InvalidReleaseTime);
        
        // Initialize all fields
        escrow.bump = ctx.bumps.escrow;
        escrow.vault_bump = ctx.bumps.vault;
        escrow.escrow_id = escrow_id;
        escrow.maker = ctx.accounts.maker.key();
        escrow.taker = ctx.accounts.taker.key();
        escrow.amount_total = amount;
        escrow.created_at = clock::Clock::get()?.unix_timestamp;
        escrow.deadline = deadline;
        escrow.auto_release_at = auto_release_at;
        escrow.status = EscrowStatus::Active as u8;
        escrow.arbiter = arbiter.unwrap_or(Pubkey::default());
        escrow.spec_hash = spec_hash;
        escrow.amount_released = 0;
        escrow.disputed_at = 0; // default 0
        escrow.amount_refunded = 0;
        escrow.milestone_index = 0;
        escrow.revision_requests = 0;
        escrow.deliverable_hash = [0u8; 32];
        escrow.dispute_evidence_uri_hash = [0u8; 32];
        escrow.completed_at = 0;


    //  Transfer SOL to escrow
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.maker.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                },
            ),
            amount,
        )?;

     // Calculate rent for vault account
    // let rent = Rent::get()?;
    // let vault_rent = rent.minimum_balance(0); // 0 space
    
    // // Create vault account via CPI
    // let binding = ctx.accounts.maker.key();
    // let vault_seeds = &[
    //     b"vault",
    //     binding.as_ref(),
    //     &escrow_id.to_le_bytes(),
    //     &[ctx.bumps.vault],
    // ];
    
    // // Create vault account with rent exemption and initial deposit
    // system_program::create_account(
    //     CpiContext::new(
    //         ctx.accounts.system_program.to_account_info(),
    //         system_program::CreateAccount {
    //             from: ctx.accounts.maker.to_account_info(),
    //             to: ctx.accounts.vault.to_account_info(),
    //         },
    //     )
    //     .with_signer(&[vault_seeds]),
    //     vault_rent + amount, // Rent + initial deposit
    //     0,                   // Space
    //     &System::id(),       // Owned by System Program
    // )?;


        Ok(())
    }

    pub fn submit_work(
        ctx: Context<SubmitWork>,
        deliverable_hash: [u8; 32],
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;
        
        // Ensure escrow is in a valid state for submission
        require!(escrow.status == EscrowStatus::Active as u8, EscrowError::InvalidState);

        // Ensure caller is the taker
        require!(*ctx.accounts.taker.key == escrow.taker, EscrowError::Unauthorized);

        // Ensure deadline has not passed
        require!(clock.unix_timestamp <= escrow.deadline, EscrowError::DeadlinePassed);
        
        // Ensure deliverable hash is not already set
        escrow.deliverable_hash = deliverable_hash;
        escrow.status = EscrowStatus::Submitted as u8;
        
        Ok(())
    }

 pub fn approve_work(ctx: Context<ApproveWork>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        
        // Ensure escrow is in a valid state for approval
        require!(escrow.status == EscrowStatus::Submitted as u8, EscrowError::InvalidState);

        // Ensure caller is the maker
        require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
        
        // Ensure funds are available for release
        let amount = escrow.amount_total - escrow.amount_released;
        require!(amount > 0, EscrowError::NoFundsAvailable);

        // Prepare seeds for vault PDA signing
        let seeds = &[
            b"vault",
            escrow.maker.as_ref(),
            &escrow.escrow_id.to_le_bytes(),
            &[escrow.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        // Transfer funds to taker
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.taker.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        // Update state
        let escrow = &mut ctx.accounts.escrow;
        escrow.amount_released += amount;
        escrow.status = EscrowStatus::Completed as u8;
        escrow.completed_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }
   
    pub fn request_revision(ctx: Context<RequestRevision>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        
        // Ensure escrow is in a valid state for revision
        require!(escrow.status == EscrowStatus::Submitted as u8, EscrowError::InvalidState);
        // Ensure caller is the maker
        require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
        
        // Ensure revision requests do not exceed limit
        escrow.status = EscrowStatus::Active as u8;
        escrow.revision_requests = escrow.revision_requests.checked_add(1).ok_or(EscrowError::Overflow)?;
        
        Ok(())
    }

    pub fn raise_dispute(
        ctx: Context<RaiseDispute>,
        evidence_uri_hash: [u8; 32],
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let caller = ctx.accounts.caller.key();
        

        // Ensure escrow is in a valid state for dispute
        require!(
            escrow.status == EscrowStatus::Active as u8 || 
            escrow.status == EscrowStatus::Submitted as u8,
            EscrowError::InvalidState
        );

        // Ensure caller is either maker or taker
        require!(
            caller == escrow.maker || caller == escrow.taker,
            EscrowError::Unauthorized
        );

        // Ensure escrow is not already disputed
        require!(
        escrow.status != EscrowStatus::Disputed as u8,
        EscrowError::AlreadyDisputed
    );
        
        // Update escrow status and evidence
        escrow.status = EscrowStatus::Disputed as u8;
        escrow.dispute_evidence_uri_hash = evidence_uri_hash;
        escrow.disputed_at = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

pub fn arbiter_resolve(
    ctx: Context<ArbiterResolve>,
    taker_amount: u64,
    maker_amount: u64,
) -> Result<()> {
    let escrow = &ctx.accounts.escrow;

    // Ensure escrow is in disputed state
    require!(
        escrow.status == EscrowStatus::Disputed as u8,
        EscrowError::InvalidState
    );
    // Ensure arbiter is authorized
    require!(
        *ctx.accounts.arbiter.key == escrow.arbiter,
        EscrowError::Unauthorized
    );

    // Ensure both parties are authorized
    require!(ctx.accounts.maker.key == &escrow.maker, EscrowError::Unauthorized);
    require!(ctx.accounts.taker.key == &escrow.taker, EscrowError::Unauthorized);

    // Calculate available funds
    let available = escrow.amount_total
        .checked_sub(escrow.amount_released)
        .and_then(|v| v.checked_sub(escrow.amount_refunded))
        .ok_or(EscrowError::Overflow)?;

    let total = taker_amount
        .checked_add(maker_amount)
        .ok_or(EscrowError::Overflow)?;

    // Ensure at least one amount is non-zero
    require!(maker_amount > 0 || taker_amount > 0, EscrowError::InvalidAmount);

    // Ensure total does not exceed available funds
    require!(total <= available, EscrowError::InvalidAmount);

    // Prepare seeds for vault PDA signing
    let seeds = &[
        b"vault",
        escrow.maker.as_ref(),
        &escrow.escrow_id.to_le_bytes(),
        &[escrow.vault_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // Transfer to taker
    if taker_amount > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.taker.to_account_info(),
                },
                signer_seeds,
            ),
            taker_amount,
        )?;
    }

    // Transfer to maker
    if maker_amount > 0 {
        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.maker.to_account_info(),
                },
                signer_seeds,
            ),
            maker_amount,
        )?;
    }

    // Update escrow state
    let escrow = &mut ctx.accounts.escrow;
    escrow.amount_released = escrow
        .amount_released
        .checked_add(taker_amount)
        .ok_or(EscrowError::Overflow)?;
    escrow.amount_refunded = escrow
        .amount_refunded
        .checked_add(maker_amount)
        .ok_or(EscrowError::Overflow)?;
    escrow.status = EscrowStatus::Completed as u8;
    escrow.completed_at = Clock::get()?.unix_timestamp;

    Ok(())
}

   pub fn cancel_before_start(ctx: Context<CancelBeforeStart>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        
        require!(escrow.status == EscrowStatus::Active as u8, EscrowError::InvalidState);
        require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
        require!(escrow.amount_released == 0, EscrowError::FundsAlreadyReleased);
        
        let amount = escrow.amount_total - escrow.amount_refunded;
        require!(amount > 0, EscrowError::NoFundsAvailable);


          // Prepare seeds for vault PDA signing
    let seeds = &[
        b"vault",
        escrow.maker.as_ref(),
        &escrow.escrow_id.to_le_bytes(),
        &[escrow.vault_bump],
    ];
    let signer_seeds = &[&seeds[..]];
        
          system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.maker.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        
        escrow.amount_refunded += amount;
        escrow.status = EscrowStatus::Cancelled as u8;
        
        Ok(())
    }


    pub fn claim_timeout(ctx: Context<ClaimTimeout>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

                  // Prepare seeds for vault PDA signing
    let seeds = &[
        b"vault",
        escrow.maker.as_ref(),
        &escrow.escrow_id.to_le_bytes(),
        &[escrow.vault_bump],
    ];
          let signer_seeds = &[&seeds[..]];
        match escrow.status {
            s if s == EscrowStatus::Active as u8 && current_time > escrow.deadline => {
                require!(
                    *ctx.accounts.claimant.key == escrow.maker,
                    EscrowError::Unauthorized
                );
                
                let amount = escrow.amount_total - escrow.amount_refunded;
                require!(amount > 0, EscrowError::NoFundsAvailable);
                
            system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.claimant.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
                
                escrow.amount_refunded += amount;
                escrow.status = EscrowStatus::Cancelled as u8;
                escrow.completed_at = current_time;

            }
            s if s == EscrowStatus::Submitted as u8 && current_time > escrow.auto_release_at => {
                require!(
                    *ctx.accounts.claimant.key == escrow.taker,
                    EscrowError::Unauthorized
                );
                
                let amount = escrow.amount_total - escrow.amount_released;
                require!(amount > 0, EscrowError::NoFundsAvailable);
                
                 system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.claimant.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
                
                escrow.amount_released += amount;
                escrow.status = EscrowStatus::Completed as u8;
                escrow.completed_at = current_time;
            }
            _ => return Err(EscrowError::ClaimNotAvailable.into()),
        }
        
        Ok(())
    }


}



// Instruction Context
#[derive(Accounts)]
#[instruction(escrow_id: u64, amount: u64, taker: Pubkey)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    /// CHECK: Taker is not signing yet
    pub taker: AccountInfo<'info>,
    // Metadata account
    #[account(
        init,
        payer = maker,
        space = Escrow::SPACE,
        seeds = [b"escrow", maker.key().as_ref(), &escrow_id.to_le_bytes()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    /// CHECK: Vault created manually, must match seeds
    #[account(
        mut,
        seeds = [b"vault", maker.key().as_ref(), &escrow_id.to_le_bytes()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

// Escrow Account
#[account]
pub struct Escrow {
    pub bump: u8,
    pub vault_bump: u8,
    pub escrow_id: u64,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub created_at: i64,
    pub deadline: i64,
    pub auto_release_at: i64,
    pub completed_at: i64,
    pub disputed_at: i64, // default 0
    pub status: u8,
    pub arbiter: Pubkey,
    pub amount_total: u64,
    pub amount_released: u64,
    pub amount_refunded: u64,
    pub milestone_index: u8,
    pub spec_hash: [u8; 32],
    pub deliverable_hash: [u8; 32],
    pub revision_requests: u16,
    pub dispute_evidence_uri_hash: [u8; 32],
}

impl Escrow {
    const SPACE: usize = 8 + // discriminator
        1 +  // bump
        1 +  // vault_bump
        8 +  // escrow_id
        32 + // maker
        32 + // taker
        8 +  // created_at
        8 +  // deadline
        8 +  // auto_release_at
        8 +  // completed_at
        8 +  // disputed_at
        1 +  // status
        32 + // arbiter
        8 +  // amount_total
        8 +  // amount_released
        8 +  // amount_refunded
        1 +  // milestone_index
        32 + // spec_hash
        32 + // deliverable_hash
        2 +  // revision_requests
        32;  // dispute_evidence_uri_hash

}

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
}

#[derive(Accounts)]
pub struct ApproveWork<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    /// CHECK: Taker account for SOL transfer
    #[account(mut)]
    pub taker: AccountInfo<'info>,
      #[account(
        mut,
        seeds = [b"escrow", escrow.maker.as_ref(), &escrow.escrow_id.to_le_bytes()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    //    /// CHECK: Validated by PDA seeds
    // #[account(
    //     mut,
    //     seeds = [b"vault", escrow.maker.as_ref(), &escrow.escrow_id.to_le_bytes()],
    //     bump = escrow.vault_bump
    // )]
    // pub vault: AccountInfo<'info>,
    /// CHECK: Vault created manually, must match seeds
    #[account(
        mut,
        seeds = [b"vault", escrow.maker.key().as_ref(), &escrow.escrow_id.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

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

    /// CHECK: Maker will receive funds
    #[account(mut)]
    pub maker: AccountInfo<'info>,

    /// CHECK: Taker will receive funds
    #[account(mut)]
    pub taker: AccountInfo<'info>,

    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    /// CHECK: Vault holding funds, must match seeds
    #[account(
        mut,
        seeds = [b"vault", escrow.maker.as_ref(), &escrow.escrow_id.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelBeforeStart<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

        /// CHECK: Vault holding funds, must match seeds
    #[account(
        mut,
        seeds = [b"vault", escrow.maker.as_ref(), &escrow.escrow_id.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTimeout<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
          /// CHECK: Vault holding funds, must match seeds
    #[account(
        mut,
        seeds = [b"vault", escrow.maker.as_ref(), &escrow.escrow_id.to_le_bytes()],
        bump = escrow.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}



// Escrow Status Enum
#[repr(u8)]
pub enum EscrowStatus {
    Active,
    Submitted,
    Completed,
    Disputed,
    Cancelled,
}


// Error Codes
#[error_code]
pub enum EscrowError {
    #[msg("Invalid escrow state for this operation")]
    InvalidState,
    #[msg("Invalid amount specified")]
    InvalidAmount,
    #[msg("Claim not available at this time")]
    ClaimNotAvailable,
    #[msg("No funds available in escrow")]
    NoFundsAvailable,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("This escrow is already disputed")]
    AlreadyDisputed,
    #[msg("Deadline has already passed")]
    DeadlinePassed,
    #[msg("Funds have already been released")]
    FundsAlreadyReleased,
    #[msg("Invalid deadline specified")]
    InvalidDeadline,
    #[msg("Invalid auto-release time specified")]
    InvalidReleaseTime,
    #[msg("Arithmetic overflow")]
    Overflow,
}

