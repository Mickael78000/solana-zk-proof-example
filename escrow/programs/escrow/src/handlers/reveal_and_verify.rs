use anchor_lang::prelude::*;
use crate::state::{EnhancedEscrowState, CommitmentStorage, OrderDetails};
use crate::error::EscrowError;

#[derive(Accounts)]
pub struct RevealAndVerify<'info> {
    #[account(
        mut,
        constraint = commitment_storage.user == initializer.key() @ EscrowError::InvalidInitializer,
        constraint = !commitment_storage.revealed @ EscrowError::AlreadyRevealed
    )]
    pub commitment_storage: Account<'info, CommitmentStorage>,
    
    #[account(
        mut,
        constraint = escrow_account.commitment_hash == commitment_storage.commitment_hash @ EscrowError::CommitmentMismatch
    )]
    pub escrow_account: Account<'info, EnhancedEscrowState>,
    
    pub initializer: Signer<'info>,
}

pub fn reveal_and_verify(
    ctx: Context<RevealAndVerify>,
    order_details: OrderDetails,
    nonce: [u8; 32],
) -> Result<()> {
    let commitment_storage = &mut ctx.accounts.commitment_storage;
    let escrow_account = &mut ctx.accounts.escrow_account;
    
    // Verify commitment using Blake2b hash
    let computed_hash = compute_commitment_hash(&order_details, &nonce)?;
    require!(
        computed_hash == commitment_storage.commitment_hash,
        EscrowError::InvalidCommitmentReveal
    );
    
    // Update escrow with revealed order details
    escrow_account.initializer_amount = order_details.token_amount;
    escrow_account.optimal_venue_id = order_details.preferred_venue;
    
    // Mark as revealed
    commitment_storage.revealed = true;
    
    msg!("Order revealed and verified successfully");
    msg!("Token amount: {}, Min receive: {}, Venue: {}", 
        order_details.token_amount,
        order_details.min_receive_amount,
        order_details.preferred_venue
    );
    
    Ok(())
}

fn compute_commitment_hash(order: &OrderDetails, nonce: &[u8; 32]) -> Result<[u8; 32]> {
    use anchor_lang::solana_program::hash::hashv;
    
    let order_bytes = anchor_lang::AnchorSerialize::try_to_vec(order)
        .map_err(|_| EscrowError::SerializationError)?;
    
    let hash = hashv(&[&order_bytes, nonce]);
    Ok(hash.to_bytes())
}