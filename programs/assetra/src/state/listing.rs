use anchor_lang::prelude::*;

use crate:: constants::*;

#[account]
pub struct Listing{
    pub track: Pubkey,
    pub seller: Pubkey,
    pub mint: Pubkey,
    pub price_per_token: u64,
    pub amount: u64,
    pub nonce: u64,
    pub bump: u8,
}
impl Listing{
    pub const SEED_PREFIX: &'static [u8] = SEED_LISTING;

    pub const SPACE: usize = 8 // discriminator
        + 32 // track
        + 32 // seller
        + 32 // mint
        + 8  // price_per_token
        + 8  // amount
        + 8  // nonce
        + 1; // bump
}