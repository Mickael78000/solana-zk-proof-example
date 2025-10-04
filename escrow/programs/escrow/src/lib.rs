#![allow(unexpected_cfgs)]
// Stops Rust Analyzer complaining about missing configs
// See https://solana.stackexchange.com/questions/17777

use anchor_lang::prelude::*;
use handlers::*;

pub mod constants;
pub mod error;
pub mod handlers;
pub mod state;

declare_id!("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y");

#[program]
pub mod escrow {
    use super::*;

    // Original escrow functions
    pub fn make_offer(
        context: Context<MakeOffer>,
        id: u64,
        token_a_offered_amount: u64,
        token_b_wanted_amount: u64,
    ) -> Result<()> {
        handlers::make_offer::make_offer(context, id, token_a_offered_amount, token_b_wanted_amount)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        handlers::take_offer::take_offer(context)
    }

    pub fn refund_offer(context: Context<RefundOffer>) -> Result<()> {
        handlers::refund_offer::refund_offer(context)
    }

    // Enhanced privacy-preserving functions
    pub fn initialize_enhanced(
        ctx: Context<InitializeEnhanced>,
        commitment_hash: [u8; 32],
        taker_amount: u64,
        privacy_level: state::PrivacyLevel,
    ) -> Result<()> {
        handlers::initialize_enhanced::initialize_enhanced(ctx, commitment_hash, taker_amount, privacy_level)
    }

    pub fn verify_zk_proofs(
        ctx: Context<VerifyZKProofs>,
        validity_proof: [u8; 256],
        routing_proof: [u8; 256],
        public_inputs: Vec<u64>,
    ) -> Result<()> {
        handlers::verify_zk_proofs::verify_zk_proofs(ctx, validity_proof, routing_proof, public_inputs)
    }

    pub fn reveal_and_verify(
        ctx: Context<RevealAndVerify>,
        order_details: state::OrderDetails,
        nonce: [u8; 32],
    ) -> Result<()> {
        handlers::reveal_and_verify::reveal_and_verify(ctx, order_details, nonce)
    }

    pub fn execute_atomic_swap(ctx: Context<ExecuteAtomicSwap>) -> Result<()> {
        handlers::execute_atomic_swap::execute_atomic_swap(ctx)
    }
}

#[cfg(test)]
mod escrow_test_helpers;
#[cfg(test)]
mod tests;