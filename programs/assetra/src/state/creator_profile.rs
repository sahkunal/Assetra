use anchor_lang::prelude::*;

use crate::constants::{MAX_ATTESTATION_LEN, MAX_DISPLAY_NAME_LEN, SEED_CREATOR_PROFILE};
#[account]
pub struct CreatorProfile {
    pub authority: Pubkey,
    pub display_name: String,
    pub rights_attestation: String,
    pub self_attested: bool,
    pub admin_verified: bool,
    pub track_count: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl CreatorProfile {
    pub const SEED_PREFIX: &'static [u8] = SEED_CREATOR_PROFILE;

    pub const SPACE: usize = 8 // discriminator
        + 32 // authority
        + 4 + MAX_DISPLAY_NAME_LEN // display_name (String prefix + bytes)
        + 4 + MAX_ATTESTATION_LEN  // rights_attestation
        + 1  // self_attested
        + 1  // admin_verified
        + 8  // track_count
        + 8  // created_at
        + 1; // bump
}
