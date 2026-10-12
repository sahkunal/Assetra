use anchor_lang::prelude::*;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use crate::instructions::*;

declare_id!("FmS7DSf1d5KGNQtVy6usXFaHi7fNnTtiJofHZSQzGKYi");

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

    pub fn initialize_config(ctx: Context<InitializeConfig>, verifier: Pubkey) -> Result<()> {
        instructions::initialize_config::handler(ctx, verifier)
    }

    pub fn set_verifier(ctx: Context<SetVerifier>, new_verifier: Pubkey) -> Result<()> {
        instructions::set_verifier::handler(ctx, new_verifier)
    }

    pub fn verify_track(ctx: Context<VerifyTrack>, verified_annual_revenue: u64) -> Result<()> {
        instructions::verify_track::handler(ctx, verified_annual_revenue)
    }

    pub fn mint_track_tokens(ctx: Context<MintTrackTokens>) -> Result<()> {
        instructions::mint_track_tokens::handler(ctx)
    }

    pub fn deposit_revenue(ctx: Context<DepositRevenue>, amount: u64) -> Result<()> {
        instructions::deposit_revenue::handler(ctx, amount)
    }

    pub fn claim_revenue(ctx: Context<ClaimRevenue>) -> Result<()> {
        instructions::claim_revenue::handler(ctx)
    }

    pub fn initialize_revenue_pool(ctx: Context<InitializeRevenuePool>) -> Result<()> {
        instructions::initialize_revenue_pool::handler(ctx)
    }

    pub fn buy_resale(ctx: Context<BuyResale>) -> Result<()> {
        instructions::buy_resale::handler(ctx)
    }

    pub fn list_resale(
        ctx: Context<ListResale>,
        amount: u64,
        price_per_token: u64,
        nonce: u64,
    ) -> Result<()> {
        instructions::list_resale::handler(ctx, amount, price_per_token, nonce)
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        instructions::buy_tokens::handler(ctx, amount)
    }

    pub fn cancel_resale(ctx: Context<CancelResale>) -> Result<()> {
        instructions::cancel_resale::handler(ctx)
    }
}
