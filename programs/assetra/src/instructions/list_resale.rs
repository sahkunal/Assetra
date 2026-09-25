use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Approve, Mint, TokenAccount, TokenInterface};

use crate::constants::SEED_LISTING;
use crate::errors::AssetraError;
use crate::state::{Listing, Track};

#[derive(Accounts)]
#[instruction(amount: u64, price_per_token: u64, nonce: u64)]
pub struct ListResale<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
    )]
    pub track: Account<'info, Track>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = seller,
        token::token_program = token_program,
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = seller,
        space = Listing::SPACE,
        seeds = [SEED_LISTING, track.key().as_ref(), seller.key().as_ref(), nonce.to_le_bytes().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ListResale>,
    amount: u64,
    price_per_token: u64,
    nonce: u64,
) -> Result<()> {
    require!(amount > 0, AssetraError::InvalidListingAmount);
    require!(price_per_token > 0, AssetraError::InvalidListingPrice);
    require!(
        ctx.accounts.seller_token_account.amount >= amount,
        AssetraError::InsufficientListedBalance
    );

    token_interface::approve(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Approve {
                to: ctx.accounts.seller_token_account.to_account_info(),
                delegate: ctx.accounts.listing.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        amount,
    )?;

    let listing = &mut ctx.accounts.listing;
    listing.track = ctx.accounts.track.key();
    listing.seller = ctx.accounts.seller.key();
    listing.mint = ctx.accounts.mint.key();
    listing.price_per_token = price_per_token;
    listing.amount = amount;
    listing.nonce = nonce;
    listing.bump = ctx.bumps.listing;

    Ok(())
}
