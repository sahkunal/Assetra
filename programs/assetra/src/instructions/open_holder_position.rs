use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::SEED_HOLDER_POSITION;
use crate::errors::AssetraError;
use crate::state::{HolderPosition, RevenuePool, Track};

#[derive(Accounts)]
pub struct OpenHolderPosition<'info> {
    #[account(mut)]
    pub holder: Signer<'info>,

    #[account(
        seeds = [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
    )]
    pub track: Account<'info, Track>,

    #[account(
        constraint = Some(holder_token_account.mint) == track.mint @ AssetraError::InvalidTrackStatus,
        constraint = holder_token_account.owner == holder.key(),
    )]
    pub holder_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [RevenuePool::SEED_PREFIX, track.key().as_ref()],
        bump = revenue_pool.bump,
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    #[account(
        init,
        payer = holder,
        space = HolderPosition::SPACE,
        seeds = [SEED_HOLDER_POSITION, track.key().as_ref(), holder.key().as_ref()],
        bump,
    )]
    pub holder_position: Account<'info, HolderPosition>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<OpenHolderPosition>) -> Result<()> {
    let position = &mut ctx.accounts.holder_position;
    position.track = ctx.accounts.track.key();
    position.holder = ctx.accounts.holder.key();
    position.last_accumulated_rewards_per_token =
        ctx.accounts.revenue_pool.accumulated_rewards_per_token;
    position.total_claimed = 0;
    position.bump = ctx.bumps.holder_position;
    Ok(())
}
