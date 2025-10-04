use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{EnhancedEscrowState, CommitmentStorage, ProofBatch};
use crate::error::EscrowError;

#[derive(Accounts)]
pub struct ExecuteAtomicSwap<'info> {
    #[account(
        mut,
        constraint = escrow_account.zk_proof_verified @ EscrowError::ProofsNotVerified,
        constraint = commitment_storage.revealed @ EscrowError::CommitmentNotRevealed,
        seeds = [
            b"escrow",
            escrow_account.initializer_key.as_ref(),
            escrow_account.commitment_hash.as_ref()
        ],
        bump = escrow_account.bump,
        close = initializer
    )]
    pub escrow_account: Account<'info, EnhancedEscrowState>,
    
    #[account(
        constraint = commitment_storage.escrow_pda == escrow_account.key() @ EscrowError::EscrowMismatch
    )]
    pub commitment_storage: Account<'info, CommitmentStorage>,
    
    #[account(
        constraint = proof_batch.verified @ EscrowError::ProofsNotVerified
    )]
    pub proof_batch: Account<'info, ProofBatch>,
    
    /// CHECK: Initializer account
    #[account(mut)]
    pub initializer: AccountInfo<'info>,
    
    #[account(mut)]
    pub taker: Signer<'info>,
    
    #[account(mut)]
    pub taker_deposit_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub taker_receive_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub initializer_deposit_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub initializer_receive_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"vault", escrow_account.key().as_ref()],
        bump
    )]
    pub vault_account: Account<'info, TokenAccount>,
    
    /// CHECK: Vault authority PDA
    #[account(
        seeds = [b"vault_authority", escrow_account.key().as_ref()],
        bump
    )]
    pub vault_authority: AccountInfo<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn execute_atomic_swap(ctx: Context<ExecuteAtomicSwap>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow_account;
    let clock = Clock::get()?;
    
    // Generate settlement proof hash for audit trail
    let settlement_data = format!(
        "taker:{},initializer:{},venue:{},time:{}",
        escrow_account.taker_amount,
        escrow_account.initializer_amount,
        escrow_account.optimal_venue_id,
        clock.unix_timestamp
    );
    
    let settlement_hash = anchor_lang::solana_program::keccak::hash(settlement_data.as_bytes());
    escrow_account.settlement_proof_hash = settlement_hash.to_bytes();
    escrow_account.execution_timestamp = clock.unix_timestamp;
    
    // Execute atomic token swaps
    // 1. Taker sends tokens to initializer
    let cpi_accounts_taker = Transfer {
        from: ctx.accounts.taker_deposit_token_account.to_account_info(),
        to: ctx.accounts.initializer_receive_token_account.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx_taker = CpiContext::new(cpi_program.clone(), cpi_accounts_taker);
    token::transfer(cpi_ctx_taker, escrow_account.taker_amount)?;
    
    // 2. Vault sends initializer's tokens to taker
    let binding = escrow_account.key();
    let seeds = &[
        b"vault_authority",
        binding.as_ref(),
        &[ctx.bumps.vault_authority]
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts_vault = Transfer {
        from: ctx.accounts.vault_account.to_account_info(),
        to: ctx.accounts.taker_receive_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let cpi_ctx_vault = CpiContext::new_with_signer(
        cpi_program,
        cpi_accounts_vault,
        signer
    );
    token::transfer(cpi_ctx_vault, escrow_account.initializer_amount)?;
    
    msg!("Atomic swap executed successfully");
    msg!("Settlement hash: {:?}", settlement_hash.to_bytes());
    msg!("Venue ID: {}", escrow_account.optimal_venue_id);
    
    Ok(())
}