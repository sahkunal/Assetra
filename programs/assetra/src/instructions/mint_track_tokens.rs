use anchor_lang::prelude::*;

use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};
use crate::constants::*;
use crate::errors::*;
use crate ::state::*;

#[derive(Accounts)]
pub struct MintTrackTokens<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds=[Track::SEED_PREFIX,
        track.creator.as_ref(),
        track.track_id.to_le_bytes().as_ref()],
        bump= track.bump,
        constraint= track.creator == authority.key() @AssetraError::Unauthorized,
        constraint= track.status == TrackStatus::Configured @AssetraError::InvalidTrackStatus,
    )]
    pub track:Account<'info, Track>,

    #[account(
        init,
        payer= authority,
        seeds= [SEED_MINT, track.key().as_ref()],
        bump,
        mint::decimals=0,
        mint::authority= track,
        mint::token_program= token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer= authority,
        seeds= [SEED_VAULT, track.key().as_ref()],
        bump,
        token::mint= mint,
        token::authority= track,
        token::token_program= token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MintTrackTokens>)->Result<()>{
    let track= &ctx.accounts.track;
    let supply= track.token_supply;
    let creator= track.creator;
    let track_id_bytes= track.track_id.to_le_bytes();
    let bump= track.bump;
    let signer_seeds: &[&[u8]] = &[
        Track::SEED_PREFIX,
        creator.as_ref(),
        track_id_bytes.as_ref(),
        &[bump],
    ];

    token_interface::mint_to(
        CpiContext::new_with_signer(
         ctx.accounts.token_program.key(),
         MintTo{
            mint: ctx.accounts.mint.to_account_info(),
            to : ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.track.to_account_info(),
         },
         &[signer_seeds],  
        ),
        supply,
    )?;
    let track = &mut ctx.accounts.track;
    track.mint = Some(ctx.accounts.mint.key());
    track.vault = Some(ctx.accounts.vault.key());
    track.status = TrackStatus::Minted;

    Ok(())
}