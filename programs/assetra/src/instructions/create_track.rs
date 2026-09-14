use anchor_lang::prelude::*;

use crate::constants::{MAX_ISRC_LEN, MAX_TITLE_LEN, MAX_URI_LEN};
use crate::errors::AssetraError;
use crate::state::{CreatorProfile, Track, TrackStatus};

#[derive(Accounts)]
pub struct CreateTrack<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [CreatorProfile::SEED_PREFIX, authority.key().as_ref()],
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
    title: String,
    isrc: String,
    cover_art_uri: String,
    audio_preview_uri: String,
    declared_annual_revenue: u64,
    tokenized_bps: u16,
    valuation_multiple_bps: u32,
    token_supply: u64,
) -> Result<()> {
    require!(title.len() <= MAX_TITLE_LEN, AssetraError::TitleTooLong);
    require!(isrc.len() <= MAX_ISRC_LEN, AssetraError::IsrcTooLong);
    require!(cover_art_uri.len() <= MAX_URI_LEN, AssetraError::UriTooLong);
    require!(audio_preview_uri.len() <= MAX_URI_LEN, AssetraError::UriTooLong);
    require!(declared_annual_revenue > 0, AssetraError::InvalidDeclaredRevenue);
    require!(
        tokenized_bps > 0 && tokenized_bps <= 10_000,
        AssetraError::InvalidTokenizedBps
    );
    require!(valuation_multiple_bps > 0, AssetraError::InvalidValuationMultiple);
    require!(token_supply > 0, AssetraError::InvalidTokenSupply);

    let creator_profile = &mut ctx.accounts.creator_profile;
    let track_id = creator_profile.track_count;

    let track = &mut ctx.accounts.track;
    track.creator = ctx.accounts.authority.key();
    track.track_id = track_id;
    track.title = title;
    track.isrc = isrc;
    track.cover_art_uri = cover_art_uri;
    track.audio_preview_uri = audio_preview_uri;
    track.declared_annual_revenue = declared_annual_revenue;
    track.tokenized_bps = tokenized_bps;
    track.valuation_multiple_bps = valuation_multiple_bps;
    track.token_supply = token_supply;

    track.mint = None;
    track.vault = None;
    track.status = TrackStatus::Configured;

    track.created_at = Clock::get()?.unix_timestamp;
    track.bump = ctx.bumps.track;

    creator_profile.track_count = creator_profile
        .track_count
        .checked_add(1)
        .expect("track_count overflow");

    Ok(())
}
