use anchor_lang::prelude::*;
use crate:: constants::SEED_REVENUE_POOL;

#[account]
pub struct RevenuePool{
    pub track: Pubkey,
    pub total_deposited: Pubkey,
    pub accumulated_rewards_per_token: u128,
    pub bump : u8,
}

impl RevenuePool{
    pub const SEED_PREFIX: &'static [u8] = SEED_REVENUE_POOL;

    pub const SPACE : usize = 8
    +32
    +8
    +16
    +1;
}


