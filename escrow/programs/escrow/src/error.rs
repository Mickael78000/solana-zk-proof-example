use anchor_lang::prelude::*;

// Original error codes for backward compatibility
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient token balance in maker's account")]
    InsufficientMakerBalance,

    #[msg("Insufficient token balance in taker's account")]
    InsufficientTakerBalance,

    #[msg("Invalid token mint - must be different from offered token")]
    InvalidTokenMint,

    #[msg("Amount must be greater than zero")]
    InvalidAmount,

    #[msg("Failed to withdraw tokens from vault")]
    FailedVaultWithdrawal,

    #[msg("Failed to close vault account")]
    FailedVaultClosure,

    #[msg("Failed to refund tokens from vault")]
    FailedRefundTransfer,

    #[msg("Failed to close vault during refund")]
    FailedRefundClosure,
}

// Enhanced privacy-preserving error codes
#[error_code]
pub enum EscrowError {
    #[msg("Invalid initializer")]
    InvalidInitializer,
    
    #[msg("Commitment has already been revealed")]
    AlreadyRevealed,
    
    #[msg("Commitment hash mismatch")]
    CommitmentMismatch,
    
    #[msg("Invalid commitment reveal")]
    InvalidCommitmentReveal,
    
    #[msg("ZK proofs have not been verified")]
    ProofsNotVerified,
    
    #[msg("Commitment has not been revealed")]
    CommitmentNotRevealed,
    
    #[msg("Escrow account mismatch")]
    EscrowMismatch,
    
    #[msg("Invalid validity proof")]
    InvalidValidityProof,
    
    #[msg("Invalid routing proof")]
    InvalidRoutingProof,
    
    #[msg("Too many public inputs")]
    TooManyPublicInputs,
    
    #[msg("Insufficient public inputs")]
    InsufficientPublicInputs,
    
    #[msg("Invalid token ratio")]
    InvalidTokenRatio,
    
    #[msg("Serialization error")]
    SerializationError,
}