use anchor_lang::prelude::*;

use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};
use crate:: constants::*;
use crate::errors::*;
use crate ::state::*;

#[derive(Accounts)]
pub struct MintTrackTokensM<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds=[SEED_PREFIX, 
        track.creator.as_ref(),
        track.track_id.to_le_bytes().as_ref()],
        bump= track.bump,
        constraint= token.creator== authority.key() @AssetraError::Unauthorized,
        constraint= token.status == TrackStatus::Configured @AssetraError::InvalidTrackStatus,
    )]
    pub track:Account<'info, Track>

    

}
