use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::{EnhancedEscrowState, CommitmentStorage, PrivacyLevel};

#[derive(Accounts)]
#[instruction(commitment_hash: [u8; 32])]
pub struct InitializeEnhanced<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + EnhancedEscrowState::INIT_SPACE,
        seeds = [
            b"escrow",
            initializer.key().as_ref(),
            commitment_hash.as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, EnhancedEscrowState>,
    
    #[account(
        init,
        payer = initializer,
        space = 8 + CommitmentStorage::INIT_SPACE,
        seeds = [
            b"commitment",
            initializer.key().as_ref(),
            commitment_hash.as_ref()
        ],
        bump
    )]
    pub commitment_storage: Account<'info, CommitmentStorage>,
    
    #[account(mut)]
    pub initializer: Signer<'info>,
    
    #[account(
        mut,
        constraint = initializer_deposit_token_account.owner == initializer.key(),
        constraint = initializer_deposit_token_account.mint == deposit_mint.key()
    )]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    
    pub initializer_receive_token_account: Account<'info, TokenAccount>,
    
    pub deposit_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_enhanced(
    ctx: Context<InitializeEnhanced>,
    commitment_hash: [u8; 32],
    taker_amount: u64,
    privacy_level: PrivacyLevel,
) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;
    let commitment_storage = &mut ctx.accounts.commitment_storage;
    
    // Original escrow initialization
    escrow_account.initializer_key = ctx.accounts.initializer.key();
    escrow_account.initializer_deposit_token_account = ctx.accounts.initializer_deposit_token_account.key();
    escrow_account.initializer_receive_token_account = ctx.accounts.initializer_receive_token_account.key();
    escrow_account.taker_amount = taker_amount;
    escrow_account.bump = ctx.bumps.escrow_account;
    escrow_account.initializer_amount = 0; // Set during reveal
    
    // Privacy-preserving integration fields
    escrow_account.commitment_hash = commitment_hash;
    escrow_account.zk_proof_verified = false;
    escrow_account.privacy_level = privacy_level;
    escrow_account.execution_timestamp = Clock::get()?.unix_timestamp;
    escrow_account.routing_proof_hash = [0u8; 32];
    escrow_account.settlement_proof_hash = [0u8; 32];
    escrow_account.optimal_venue_id = 0;
    
    // Initialize commitment storage
    commitment_storage.commitment_hash = commitment_hash;
    commitment_storage.user = ctx.accounts.initializer.key();
    commitment_storage.timestamp = Clock::get()?.unix_timestamp;
    commitment_storage.revealed = false;
    commitment_storage.escrow_pda = escrow_account.key();
    
    msg!("Enhanced escrow initialized with commitment: {:?}", commitment_hash);
    Ok(())
}