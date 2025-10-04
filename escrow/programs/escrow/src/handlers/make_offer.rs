use super::shared::transfer_tokens;
use crate::{error::ErrorCode, state::Offer};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

// See https://www.anchor-lang.com/docs/account-constraints#instruction-attribute
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    // Used to manage associated token accounts
    // ie where a wallet holds a specific type of token
    pub associated_token_program: Program<'info, AssociatedToken>,

    // Work with either the classic token program or
    // the newer token extensions program
    pub token_program: Interface<'info, TokenInterface>,

    // Used to create accounts
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
        seeds = [b"offer", id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
}

// Handle the make offer instruction by:
// 1. Moving the tokens from the maker's ATA to the vault
// 2. Saving the details of the offer to the offer account
pub fn make_offer(
    context: Context<MakeOffer>,
    id: u64,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
) -> Result<()> {
    // Validate amounts
    require!(token_a_offered_amount > 0, ErrorCode::InvalidAmount);
    require!(token_b_wanted_amount > 0, ErrorCode::InvalidAmount);

    // Validate token mints are different
    require!(
        context.accounts.token_mint_a.key() != context.accounts.token_mint_b.key(),
        ErrorCode::InvalidTokenMint
    );

    // Move the tokens from the maker's ATA to the vault
    transfer_tokens(
        &context.accounts.maker_token_account_a,
        &context.accounts.vault,
        &token_a_offered_amount,
        &context.accounts.token_mint_a,
        &context.accounts.maker.to_account_info(),
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| ErrorCode::InsufficientMakerBalance)?;

    // Save the details of the offer to the offer account
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_b_wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}
