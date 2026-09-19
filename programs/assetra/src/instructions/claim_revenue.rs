use anchor_lang::prelude::*;
use anchor_spl:: token_interface:: TokenAccount;

use crate::constants::*;
use crate:: errors::AssetraError;
use crate::program::Assetra;
use crate:: state::{HolderPosition, RevenuePool, Track};

#[derive(Accounts)]
pub struct ClaimRevenue<'info> {
    #[account(mut)]
    pub holder: Signer<'info>,

    #[account(
        seeds= [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
    )]
    pub track: Account<'info,Track>,

    #[account(
        constraint= Some(holder_token_account.mint)== track.mint @ AssetraError::InvalidTrackStatus,
        constraint = holder_token_account.owner == holder.key(),
    )]
    pub holder_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [RevenuePool:: SEED_PREFIX, track.key().as_ref()],
        bump= revenue_pool.bump,
        constraint = revenue_pool.track== track.key(),  
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    #[account(
        mut,
        seeds= [SEED_HOLDER_POSITION, track.key().as_ref(),holder.key().as_ref()],
        bump = holder_position.bump,
        has_one = holder @AssetraError::Unauthorized,
        constraint = holder_position.track == track.key(),
    )]
    pub holder_position: Account<'info, HolderPosition>,

}
pub fn handler (ctx: Context<ClaimRevenue>)-> Result<()>{
    let pool = &mut ctx.accounts.revenue_pool;
    let position = &mut ctx.accounts.holder_position;
    let holder_balance= ctx.accounts.holder_token_account.amount as u128;

    let owed_scale= pool
    .accumulated_rewards_per_token
    .checked_sub(position.last_accumulated_rewards_per_token)
    .expect("accumulator move forward only- checkpoint cant exceed current value")
    .checked_mul(holder_balance)
    .expect("claim with overflow");

    let pending: u64 =(owed_scale/ REVENUE_PRECISION)
    .try_into()
    .expect("pending claim exceeds u64 — unreachable at realistic deposit sizes");

    require!(pending >0, AssetraError::NothingToClaim);

    let pool_min_rent= Rent::get()?.minimum_balance(RevenuePool::SPACE);
    let pool_lamports= pool.to_account_info().lamports();
    require!(
        pool_lamports>= pool_min_rent.saturating_add(pending),
        AssetraError::InsufficientRevenuePoolBalance
    );
    **pool.to_account_info().try_borrow_mut_lamports()? -= pending;
    **ctx.accounts.holder.to_account_info().try_borrow_mut_lamports()? += pending;

    position.last_accumulated_rewards_per_token = pool.accumulated_rewards_per_token;
    position.total_claimed = position
        .total_claimed
        .checked_add(pending)
        .expect("total_claimed overflow");

    Ok(())
}
