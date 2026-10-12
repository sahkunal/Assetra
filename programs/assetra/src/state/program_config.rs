use anchor_lang::prelude::*;

use crate::constants::SEED_CONFIG;

/// Singleton configuration. `verifier` is the party trusted to attest that a
/// track's declared royalty data is real (a mock oracle for now; swap the key
/// for a real attestation service later without touching the instructions).
#[account]
pub struct ProgramConfig {
    pub admin: Pubkey,
    pub verifier: Pubkey,
    pub bump: u8,
}

impl ProgramConfig {
    pub const SEED_PREFIX: &'static [u8] = SEED_CONFIG;

    pub const SPACE: usize = 8 // discriminator
        + 32 // admin
        + 32 // verifier
        + 1; // bump
}
