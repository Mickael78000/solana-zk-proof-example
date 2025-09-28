use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};

declare_id!("6SVZnwSz6xkgK8AnK3JWNgj5Yn5fqC7tjZM1qwit7rER");

// Initialize global state
#[derive(Accounts)]
pub struct InitializeState<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 32 + 1,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Commit context with validation
#[derive(Accounts)]
#[instruction(commitment_index: u64)]
pub struct CommitOrder<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 1 + 32 + 1 + 8,
        seeds = [b"commitment", user.key().as_ref(), &commitment_index.to_le_bytes()],
        bump
    )]
    pub commitment: Account<'info, Commitment>,
    #[account(
        constraint = state.commit_deadline > Clock::get()?.unix_timestamp @ CommitRevealError::CommitPhaseClosed,
        constraint = !state.paused @ CommitRevealError::ProgramPaused
    )]
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Reveal context with validation
#[derive(Accounts)]
pub struct RevealOrder<'info> {
    #[account(
        mut,
        constraint = commitment.user == *user.key,
        constraint = !commitment.revealed @ CommitRevealError::AlreadyRevealed,
        constraint = Clock::get()?.unix_timestamp > state.commit_deadline @ CommitRevealError::RevealNotStarted,
        constraint = Clock::get()?.unix_timestamp < state.reveal_deadline @ CommitRevealError::RevealPhaseClosed
    )]
    pub commitment: Account<'info, Commitment>,
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

// Initialize global state
#[derive(Accounts)]
pub struct InitializeState<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 32 + 1,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Commit context with validation
#[derive(Accounts)]
#[instruction(commitment_index: u64)]
pub struct CommitOrder<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 1 + 32 + 1 + 8,
        seeds = [b"commitment", user.key().as_ref(), &commitment_index.to_le_bytes()],
        bump
    )]
    pub commitment: Account<'info, Commitment>,
    #[account(
        constraint = state.commit_deadline > Clock::get()?.unix_timestamp @ CommitRevealError::CommitPhaseClosed,
        constraint = !state.paused @ CommitRevealError::ProgramPaused
    )]
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Reveal context with validation
#[derive(Accounts)]
pub struct RevealOrder<'info> {
    #[account(
        mut,
        constraint = commitment.user == *user.key,
        constraint = !commitment.revealed @ CommitRevealError::AlreadyRevealed,
        constraint = Clock::get()?.unix_timestamp > state.commit_deadline @ CommitRevealError::RevealNotStarted,
        constraint = Clock::get()?.unix_timestamp < state.reveal_deadline @ CommitRevealError::RevealPhaseClosed
    )]
    pub commitment: Account<'info, Commitment>,
    pub state: Account<'info, CommitRevealState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

// Global state account
#[account]
pub struct CommitRevealState {
    pub commit_deadline: i64,    // Unix timestamp
    pub reveal_deadline: i64,    // Unix timestamp  
    pub authority: Pubkey,       // Program authority
    pub paused: bool,           // Emergency pause flag
}

// Individual commitment account (PDA-based)
#[account]
pub struct Commitment {
    pub commitment_hash: [u8; 32],    // SHA256 hash
    pub commit_time: i64,             // Commit timestamp
    pub revealed: bool,               // Reveal status
    pub user: Pubkey,                 // Owner
    pub zk_proof_verified: bool,      // ZK proof status
    pub commitment_index: u64,        // Sequential index
}

// Custom error types
#[error_code]
pub enum CommitRevealError {
    #[msg("Commit phase has ended")]
    CommitPhaseClosed,
    #[msg("Reveal phase has not started")]
    RevealNotStarted,
    #[msg("Reveal phase has ended")]
    RevealPhaseClosed,
    #[msg("Already revealed")]
    AlreadyRevealed,
    #[msg("Invalid reveal - hash mismatch")]
    InvalidReveal,
    #[msg("Unauthorized action")]
    UnauthorizedAction,
    #[msg("Program is paused")]
    ProgramPaused,
    #[msg("ZK proof verification failed")]
    ZkProofFailed,
}

// Events for tracking program activity
#[event]
pub struct StateInitialized {
    pub authority: Pubkey,
    pub commit_deadline: i64,
    pub reveal_deadline: i64,
}

#[event]
pub struct OrderCommitted {
    pub user: Pubkey,
    pub commitment_hash: [u8; 32],
    pub timestamp: i64,
    pub commitment_account: Pubkey,
    pub commitment_index: u64,
}

#[event]
pub struct OrderRevealed {
    pub user: Pubkey,
    pub commitment_account: Pubkey,
    pub revealed_data: Vec<u8>,
    pub timestamp: i64,
}
