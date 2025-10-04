use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct EnhancedEscrowState {
    // Original escrow fields
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receive_token_account: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
    pub bump: u8,
    
    // Integration fields for privacy-preserving trading
    pub commitment_hash: [u8; 32],           // Blake2b commitment hash
    pub zk_proof_verified: bool,             // ZK proof validation status
    pub routing_proof_hash: [u8; 32],        // Routing optimality proof
    pub settlement_proof_hash: [u8; 32],     // Settlement audit proof
    pub execution_timestamp: i64,            // Atomic execution timestamp
    pub optimal_venue_id: u8,                // Selected DEX venue
    pub privacy_level: PrivacyLevel,         // Privacy configuration
}

#[derive(AnchorSerialize, AnchorDeserialize,Clone, Copy, PartialEq, Eq)]
pub enum PrivacyLevel {
    Public,
    Confidential,
    ZeroKnowledge,
}


// Manual Space implementation for PrivacyLevel
impl anchor_lang::Space for PrivacyLevel {
    const INIT_SPACE: usize = 1; // 1 byte for enum discriminant
}

#[account]
#[derive(InitSpace)]
pub struct CommitmentStorage {
    pub commitment_hash: [u8; 32],
    pub user: Pubkey,
    pub timestamp: i64,
    pub revealed: bool,
    pub escrow_pda: Pubkey,
}

#[account]
#[derive(InitSpace)]

pub struct ProofBatch {
    pub validity_proof: [u8; 256],     // Groth16 proof data
    pub routing_proof: [u8; 256],      // Routing optimality proof
    pub settlement_proof: [u8; 256],   // Settlement verification proof
    #[max_len(10)]
    pub public_inputs: Vec<u64>,       // Public proof inputs
    pub verified: bool,                // Batch verification status
}

impl ProofBatch {
    pub const LEN: usize = 8 + // discriminator
        256 + // validity_proof
        256 + // routing_proof  
        256 + // settlement_proof
        4 + (8 * 10) + // public_inputs vec
        1; // verified bool
}