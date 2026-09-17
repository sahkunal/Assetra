use anchor_lang::prelude::*;
use crate:: constants::SEED_HOLDER_POSITION;

#[account]
pub struct HolderPosition{
    pub track: Pubkey,
    pub holder: Pubkey,
    pub last_accumulated_rewaard_per_token: u128,
    pub total_claimed: u64,
    pub bump: u8,
}

impl HolderPosition{
    pub const SEED_PREFIX: &'static [u8] =SEED_HOLDER_POSITION;

    pub const SPACE: usize= 8 
    +32
    +32
    +16
    +8
    +1;
}