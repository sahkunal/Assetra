use  anchor_lang::prelude::*;

use crate::errors::AssetraErrors;
use crate::state::{CreatorProfile};

#[derive(Accounts)]


pub struct CreateTrack<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds=[creator_profile::SEED_PREFIX, authority.key().as_ref()],
        bump= creator_profile.bump,
        has_one= authority,
    )]
    pub creator_profile: Account<'info, CreatorProfile>,

    #[account(
        init,
        payer= authority,
        space =Track::SPACE,
        seeds =[
            Track:: SEED_PREFIX,
            authority.key().as_Ref(),
            creator_profile.track_count.to_le_bytes().as_ref(),  
        ],
        bump,
    )]
    pub track : Account<'info, Track>,
    pub system_program: Program<'info, System>,
}
