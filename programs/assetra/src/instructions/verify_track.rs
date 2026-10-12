use anchor_lang::prelude::*;

use crate::errors::AssetraError;
use crate::state::{ProgramConfig, Track, TrackStatus};

#[derive(Accounts)]
pub struct VerifyTrack<'info> {
    pub verifier: Signer<'info>,

    #[account(
        seeds = [ProgramConfig::SEED_PREFIX],
        bump = config.bump,
        has_one = verifier @ AssetraError::Unauthorized,
    )]
    pub config: Account<'info, ProgramConfig>,

    #[account(
        mut,
        seeds = [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
        constraint = track.status == TrackStatus::Configured @ AssetraError::InvalidTrackStatus,
    )]
    pub track: Account<'info, Track>,
}

/// Attests the track's royalty data. `verified_annual_revenue` is the figure
/// the verifier vouches for; primary-sale pricing uses it, not the creator's
/// declared number.
pub fn handler(ctx: Context<VerifyTrack>, verified_annual_revenue: u64) -> Result<()> {
    require!(
        verified_annual_revenue > 0,
        AssetraError::InvalidVerifiedRevenue
    );

    let track = &mut ctx.accounts.track;
    track.verified_annual_revenue = verified_annual_revenue;
    track.verified_at = Clock::get()?.unix_timestamp;
    track.status = TrackStatus::Verified;
    Ok(())
}
