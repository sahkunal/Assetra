use anchor_lang::prelude::*;

use crate::constants::{MAX_ISRC_LEN, MAX_TITLE_LEN, MAX_URI_LEN, SEED_TRACK};

#[account]
pub struct Track {
    pub creator: Pubkey,
  
    pub track_id: u64,

    pub title: String,
    pub isrc: String,
    pub cover_art_uri: String,
    pub audio_preview_uri: String,

    pub declared_annual_revenue: u64,
    pub tokenized_bps: u16,

    pub valuation_multiple_bps: u32,
    pub token_supply: u64,
    pub mint: Option<Pubkey>,
    pub vault: Option<Pubkey>,
    pub status: TrackStatus,

    pub created_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackStatus {
    Configured,   
    Minted,       
}

impl Track {
    pub const SEED_PREFIX: &'static [u8] = SEED_TRACK;

    pub const SPACE: usize = 8 // discriminator
        + 32 // creator
        + 8  // track_id
        + 4 + MAX_TITLE_LEN
        + 4 + MAX_ISRC_LEN
        + 4 + MAX_URI_LEN // cover_art_uri
        + 4 + MAX_URI_LEN // audio_preview_uri
        + 8  // declared_annual_revenue
        + 2  // tokenized_bps
        + 4  // valuation_multiple_bps
        + 8  // token_supply
        + 1 + 32 // Option<Pubkey> mint
        + 1 + 32 // Option<Pubkey> vault
        + 1  // status enum
        + 8  // created_at
        + 1; // bump
}
