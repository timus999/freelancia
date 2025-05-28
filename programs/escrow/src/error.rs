use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode{
    #[msg("Account already initialized")]
    AlreadyInitialized,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
}