use  anchor_lang::prelude::*;
use crate:: constants::*;
use crate::errors::*;
use crate::state::*;

#[derive(Accounts)]


pub struct CreateTrack<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    
     #[account(
        mut,
        seeds = [b"creator_profile", authority.key().as_ref()],
        bump = creator_profile.bump,
        has_one = authority,
    )]
    pub creator_profile: Account<'info, CreatorProfile>,

    #[account(
        init,
        payer = authority,
        space = Track::SPACE,
        seeds = [
            Track::SEED_PREFIX,
            authority.key().as_ref(),
            creator_profile.track_count.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub track: Account<'info, Track>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateTrack>,
    title :String,
    isrc: String,
    cover_art_uri: String,
    audio_preview_uri: String,
    declared_annual_revenue: u64,
)-> Result<()>{
    require!(title.len()<= MAX_TITLE_LEN,AssetraError::TitleTooLong);
    require!(isrc.len()<=MAX_ISRC_LEN, AssetraError::IsrcTooLong);
    require!(cover_art_uri.len()<=MAX_URI_LEN, AssetraError::UriTooLong);
    require!(audio_preview_uri.len()<=MAX_DISPLAY_URI_LEN, AssetraError::UriTooLong);
    require!(declared_annual_revenue>0, AssetraError:: InvalidDeclaredRevenue);

    let create_profile= &mut ctx.accounts.creator_profile;
    let track_id= create_profile.track_count;

    let track = &mut ctx.accounts.track;
    track.creator= ctx.accounts.authority.key();
    track.track_id= track_id;
    track.title= title;
    track.isrc= isrc;
    track.cover_art_uri= cover_art_uri;
    track.audio_preview_uri= audio_preview_uri;
    track.declared_annual_revenue= declared_annual_revenue;

    track.tokenized_bps= 0;
    track.valuation_multiple_bps= 0;
    track.token_supply= 0;
    track.mint= None;
    track.vault= None;
    track.status=TrackStatus::Registered;

    track.created_at=Clock::get()?.unix_timestamp;
    track.bump= ctx.bumps.track;

    create_profile.track_count= create_profile
    .track_count
    .checked_add(1)
    .expect("track_count_overflow");

    Ok(())

}
