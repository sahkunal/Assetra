use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Revoke, TokenAccount, TokenInterface};

use crate::errors::AssetraError;
use crate::state::{Listing, Track};

#[derive(Accounts)]
pub struct CancelResale<'info> {
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
        token::mint = listing.mint,
        token::authority = seller,
        token::token_program = token_program,
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [Listing::SEED_PREFIX, track.key().as_ref(), seller.key().as_ref(), listing.nonce.to_le_bytes().as_ref()],
        bump = listing.bump,
        constraint = listing.seller == seller.key() @ AssetraError::Unauthorized,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,

    pub token_program: Interface<'info, TokenInterface>,
}

/// Revokes the SPL delegate approval `list_resale` granted, then closes the
/// Listing account, returning its rent to the seller. Tokens never left
/// the seller's own account, so there's nothing to transfer back.
pub fn handler(ctx: Context<CancelResale>) -> Result<()> {
    token_interface::revoke(CpiContext::new(
        ctx.accounts.token_program.key(),
        Revoke {
            source: ctx.accounts.seller_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        },
    ))?;

    Ok(())
}
