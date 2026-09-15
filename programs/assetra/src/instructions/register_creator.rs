use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::AssetraError;
use crate::state::CreatorProfile;

#[derive(Accounts)]
pub struct RegisterCreator<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = CreatorProfile::SPACE,
        seeds = [CreatorProfile::SEED_PREFIX, authority.key().as_ref()],
        bump,
    )]
    pub creator_profile: Account<'info, CreatorProfile>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterCreator>,
    display_name: String,
    rights_attestation: String,
) -> Result<()> {
    require!(
        display_name.len() <= MAX_DISPLAY_URI_LEN,
        AssetraError::DisplayNameTooLong
    );
    require!(
        rights_attestation.len() <= MAX_ATTESTATION_LEN,
        AssetraError::AttestationTooLong
    );

    let profile = &mut ctx.accounts.creator_profile;
    profile.authority = ctx.accounts.authority.key();
    profile.display_name = display_name;
    profile.rights_attestatiioin = rights_attestation;
    profile.self_attested = true.to_string();
    profile.admin_verification = false; // set later by a separate admin-gated ix
    profile.track_count = 0;
    profile.created_at = Clock::get()?.unix_timestamp as u64;
    profile.bump = ctx.bumps.creator_profile;

    Ok(())
}
