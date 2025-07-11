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
        escrow.amount_refunded = 0;
        escrow.milestone_index = 0;
        escrow.revision_requests = 0;
        escrow.deliverable_hash = [0u8; 32];
        escrow.dispute_evidence_uri_hash = [0u8; 32];
        escrow.completed_at = 0;


    // //  Transfer SOL to escrow
    //     system_program::transfer(
    //         CpiContext::new(
    //             ctx.accounts.system_program.to_account_info(),
    //             system_program::Transfer {
    //                 from: ctx.accounts.maker.to_account_info(),
    //                 to: ctx.accounts.vault.to_account_info(),
    //             },
    //         ),
    //         amount,
    //     )?;

     // Calculate rent for vault account
    let rent = Rent::get()?;
    let vault_rent = rent.minimum_balance(0); // 0 space
    
    // Create vault account via CPI
    let binding = ctx.accounts.maker.key();
    let vault_seeds = &[
        b"vault",
        binding.as_ref(),
        &escrow_id.to_le_bytes(),
        &[ctx.bumps.vault],
    ];
    
    system_program::create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: ctx.accounts.maker.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        )
        .with_signer(&[vault_seeds]),
        vault_rent + amount, // Rent + initial deposit
        0,                   // Space
        &System::id(),       // Owned by System Program
    )?;


        Ok(())
    }

    pub fn submit_work(
        ctx: Context<SubmitWork>,
        deliverable_hash: [u8; 32],
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;
        
        require!(escrow.status == EscrowStatus::Active as u8, EscrowError::InvalidState);
        require!(*ctx.accounts.taker.key == escrow.taker, EscrowError::Unauthorized);
        require!(clock.unix_timestamp <= escrow.deadline, EscrowError::DeadlinePassed);
        
        escrow.deliverable_hash = deliverable_hash;
        escrow.status = EscrowStatus::Submitted as u8;
        
        Ok(())
    }

 pub fn approve_work(ctx: Context<ApproveWork>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        
        require!(escrow.status == EscrowStatus::Submitted as u8, EscrowError::InvalidState);
        require!(*ctx.accounts.maker.key == escrow.maker, EscrowError::Unauthorized);
        
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
        
        // Transfer from vault to taker
        // let cpi_ctx = CpiContext::new(
        //     ctx.accounts.system_program.to_account_info(),
        //     system_program::Transfer {
        //         from: ctx.accounts.escrow_vault.to_account_info(),
        //         to: ctx.accounts.taker.to_account_info(),
        //     },
        // ).with_signer(signer_seeds);
        
        // system_program::transfer(cpi_ctx, amount)?;
        
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
    //  // SOL vault PDA
    // /// CHECK: This is safe because we validate the PDA
    // #[account(
    //     init,
    //     payer = maker,
    //     space = 0, // No additional space needed for vault
    //     seeds = [b"vault", maker.key().as_ref(), &escrow_id.to_le_bytes()],
    //     bump
    // )]
    // pub vault: AccountInfo<'info>,
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