use anchor_lang::prelude::*;
// use sha2::{_Sha256, _Digest};

declare_id!("6SVZnwSz6xkgK8AnK3JWNgj5Yn5fqC7tjZM1qwit7rER");

#[program]
pub mod commit_reveal_dapp {
    use super::*;

    pub fn initialize_state(ctx: Context<InitializeState>, commit_deadline: i64, reveal_deadline: i64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.commit_deadline = commit_deadline;
        state.reveal_deadline = reveal_deadline;
        state.authority = ctx.accounts.authority.key();
        state.paused = false;
        emit!(StateInitialized { authority: state.authority, commit_deadline, reveal_deadline });
        Ok(())
    }

    pub fn commit_order(
        ctx: Context<CommitOrder>,
        commitment_hash: Vec<u8>,
        commitment_index: u64,
        _zk_proof: Option<Vec<u8>>,
    ) -> Result<()> {
        require_eq!(commitment_hash.len(), 32, CommitRevealError::InvalidReveal);
        let mut hash_arr = [0u8; 32];
        hash_arr.copy_from_slice(&commitment_hash);

        let commitment = &mut ctx.accounts.commitment;
        commitment.commitment_hash = hash_arr;
        commitment.commit_time = Clock::get()?.unix_timestamp;
        commitment.revealed = false;
        commitment.user = ctx.accounts.user.key();
        commitment.zk_proof_verified = false;
        commitment.commitment_index = commitment_index;

        emit!(OrderCommitted {
            user: commitment.user,
            commitment_hash: commitment.commitment_hash,
            timestamp: commitment.commit_time,
            commitment_account: ctx.accounts.commitment.key(),
            commitment_index,
        });
        Ok(())
    }

    pub fn reveal_order(ctx: Context<RevealOrder>, order_data: Vec<u8>, secret: Vec<u8>) -> Result<()> {
        let mut data = order_data.clone();
        data.extend_from_slice(&secret);
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(&data);
        let computed = hasher.finalize();
        require!(computed.as_slice() == ctx.accounts.commitment.commitment_hash, CommitRevealError::InvalidReveal);

        let commitment = &mut ctx.accounts.commitment;
        commitment.revealed = true;
        emit!(OrderRevealed {
            user: commitment.user,
            commitment_account: ctx.accounts.commitment.key(),
            revealed_data: order_data,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
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
