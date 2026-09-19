use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::constants::REVENUE_PRECISION;
use crate::errors::AssetraError;
use crate::state::{RevenuePool, Track, TrackStatus};

#[derive(Accounts)]
pub struct DepositRevenue<'info> {
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
        mut,
        seeds = [RevenuePool::SEED_PREFIX, track.key().as_ref()],
        bump = revenue_pool.bump,
        constraint = revenue_pool.track == track.key(),
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositRevenue>, amount: u64) -> Result<()> {
    require!(amount > 0, AssetraError::InvalidDepositAmount);
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.revenue_pool.to_account_info(),
            },
        ),
        amount,
    )?;

    let track = &ctx.accounts.track;
    let pool = &mut ctx.accounts.revenue_pool;
    let increment = (amount as u128)
        .checked_mul(REVENUE_PRECISION)
        .expect("deposit precision-scaling overflow")
        .checked_div(track.token_supply as u128)
        .expect("token_supply is guaranteed > 0 by create_track");

    pool.accumulated_rewards_per_token = pool
        .accumulated_rewards_per_token
        .checked_add(increment)
        .expect("accumulated_rewards_per_token overflow");
    pool.total_deposited = pool
        .total_deposited
        .checked_add(amount)
        .expect("total_deposited overflow");

    Ok(())
}
