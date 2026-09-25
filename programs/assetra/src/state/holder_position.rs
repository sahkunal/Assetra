use anchor_lang::prelude::*;
use crate:: constants::SEED_HOLDER_POSITION;

#[account]
pub struct HolderPosition {
    pub track: Pubkey,
    pub holder: Pubkey,
    pub reward_debt: u128,
    pub unclaimed_rewards: u64,
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

     pub fn settle(&mut self, balance: u64, acc_per_token_now: u128) {
        let accrued_scaled = (balance as u128)
            .checked_mul(acc_per_token_now)
            .expect("settle: balance * accumulator overflow");
        let newly_accrued_scaled = accrued_scaled.saturating_sub(self.reward_debt);
        let newly_accrued: u64 = (newly_accrued_scaled / REVENUE_PRECISION)
            .try_into()
            .expect("settle: newly accrued amount exceeds u64");

        self.unclaimed_rewards = self
            .unclaimed_rewards
            .checked_add(newly_accrued)
            .expect("unclaimed_rewards overflow");
        self.reward_debt = accrued_scaled;
    }
}