use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("HaJijGCZjztBgf8jGyfW6sCEskNMfgr73pqtEAExSqpn");

#[program]
pub mod assetra {
    use super::*;

    pub fn register_creator(
        ctx: Context<RegisterCreator>,
        display_name: String,
        rights_attestation: String,
    ) -> Result<()> {
        instructions::register_creator::handler(ctx, display_name, rights_attestation)
    }
    pub fn create_track(
        ctx: Context<CreateTrack>,
        title: String,
        isrc: String,
        cover_art_uri: String,
        audio_preview_uri: String,
        declared_annual_revenue: u64,
    ) -> Result<()> {
        instructions::create_track::handler(
            ctx,
            title,
            isrc,
            cover_art_uri,
            audio_preview_uri,
            declared_annual_revenue,
        )
    }
}

