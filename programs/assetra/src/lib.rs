use anchor_lang::prelude::*;

declare_id!("HaJijGCZjztBgf8jGyfW6sCEskNMfgr73pqtEAExSqpn");

#[program]
pub mod assetra {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
