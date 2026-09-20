use anchor_lang::prelude::*;

use crate::constants::*;

#[account]
pub struct CreatorProfile{
    pub authority: Pubkey,
    pub display_name: String,
    pub rights_attestatioin : String,
    pub self_attested: String,
    pub admin_verified: bool,
    pub track_count: u64,
    pub created_at: u64,
    pub bump: u8,
}

impl CreatorProfile{
    pub const SEED_PREFIX: &'static [u8] = SEED_CREATOR_PROFILE;
    pub const SPACE: usize= 8
    +32 +4 + MAX_DISPLAY_NAME_LEN + 4 + MAX_ATTESTATION_LEN +1+1+8+8+1;
}