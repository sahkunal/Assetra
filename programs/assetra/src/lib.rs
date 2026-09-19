use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use crate ::instructions::*;

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
        tokenized_bps: u16,
        valuation_multiple_bps: u32,
        token_supply: u64,
    ) -> Result<()> {
        instructions::create_track::handler(
            ctx,
            title,
            isrc,
            cover_art_uri,
            audio_preview_uri,
            declared_annual_revenue,
            tokenized_bps,
            valuation_multiple_bps,
            token_supply,
        )
    }

    pub fn mint_track_tokens(ctx: Context<MintTrackTokens>) -> Result<()> {
        instructions::mint_track_tokens::handler(ctx)
    }

     pub fn deposit_revenue(ctx: Context<DepositRevenue>, amount: u64) -> Result<()> {
        instructions::deposit_revenue::handler(ctx, amount)
    }

    pub fn open_holder_position(ctx: Context<OpenHolderPosition>) -> Result<()> {
        instructions::open_holder_position::handler(ctx)
    }

    pub fn claim_revenue(ctx: Context<ClaimRevenue>) -> Result<()> {
        instructions::claim_revenue::handler(ctx)
    }

    pub fn initialize_revenue_pool(ctx: Context<InitializeRevenuePool>) -> Result<()> {
    instructions::initialize_revenue_pool::handler(ctx)
}
}

