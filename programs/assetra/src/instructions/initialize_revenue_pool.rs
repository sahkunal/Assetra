use anchor_lang::prelude::*;

use crate::errors::AssetraError;
use crate::state::{RevenuePool, Track, TrackStatus};

#[derive(Accounts)]
pub struct InitializeRevenuePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
        constraint = track.creator == authority.key() @ AssetraError::Unauthorized,
        constraint = track.status == TrackStatus::Minted @ AssetraError::InvalidTrackStatus,
    )]
    pub track: Account<'info, Track>,

    #[account(
        init,
        payer = authority,
        space = RevenuePool::SPACE,
        seeds = [RevenuePool::SEED_PREFIX, track.key().as_ref()],
        bump,
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeRevenuePool>) -> Result<()> {
    let pool = &mut ctx.accounts.revenue_pool;
    pool.track = ctx.accounts.track.key();
    pool.total_deposited = 0;
    pool.accumulated_rewards_per_token = 0;
    pool.bump = ctx.bumps.revenue_pool;
    Ok(())
}
