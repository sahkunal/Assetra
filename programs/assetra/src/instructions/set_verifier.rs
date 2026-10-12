use anchor_lang::prelude::*;

use crate::errors::AssetraError;
use crate::state::ProgramConfig;

#[derive(Accounts)]
pub struct SetVerifier<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [ProgramConfig::SEED_PREFIX],
        bump = config.bump,
        has_one = admin @ AssetraError::Unauthorized,
    )]
    pub config: Account<'info, ProgramConfig>,
}

pub fn handler(ctx: Context<SetVerifier>, new_verifier: Pubkey) -> Result<()> {
    ctx.accounts.config.verifier = new_verifier;
    Ok(())
}
