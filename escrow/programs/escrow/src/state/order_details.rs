use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OrderDetails {
    pub token_amount: u64,
    pub min_receive_amount: u64,
    pub preferred_venue: u8,
    pub max_slippage: u16,
}