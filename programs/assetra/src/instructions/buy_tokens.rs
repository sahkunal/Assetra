use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer as SystemTransfer};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::constants::SEED_HOLDER_POSITION;
use crate::errors::AssetraError;
use crate::state::{HolderPosition, RevenuePool, Track, TrackStatus};

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    /// CHECK: only receives lamports; identity enforced by the `address`
    #[account(mut, address = track.creator)]
    pub creator: UncheckedAccount<'info>,

    #[account(
        seeds = [
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump = track.bump,
        constraint = track.status == TrackStatus::Minted @ AssetraError::InvalidTrackStatus,
    )]
    pub track: Account<'info, Track>,

    #[account(
        mut,
        seeds = [crate::constants::SEED_VAULT, track.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = track,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = mint,
        associated_token::authority = investor,
        associated_token::token_program = token_program,
    )]
    pub investor_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [RevenuePool::SEED_PREFIX, track.key().as_ref()],
        bump = revenue_pool.bump,
        constraint = revenue_pool.track == track.key(),
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    #[account(
        init_if_needed,
        payer = investor,
        space = HolderPosition::SPACE,
        seeds = [SEED_HOLDER_POSITION, track.key().as_ref(), investor.key().as_ref()],
        bump,
    )]
    pub holder_position: Account<'info, HolderPosition>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
    require!(amount > 0, AssetraError::InvalidPurchaseAmount);
    require!(
        ctx.accounts.vault.amount >= amount,
        AssetraError::InsufficientVaultSupply
    );

    let track = &ctx.accounts.track;
    let numerator = (track.declared_annual_revenue as u128)
        .checked_mul(track.valuation_multiple_bps as u128)
        .expect("price numerator overflow: revenue * multiple")
        .checked_mul(track.tokenized_bps as u128)
        .expect("price numerator overflow: * tokenized_bps")
        .checked_mul(amount as u128)
        .expect("price numerator overflow: * amount");
    let denominator: u128 = 100 * 10_000 * (track.token_supply as u128);
    let total_price: u64 = (numerator / denominator)
        .try_into()
        .expect("purchase price exceeds u64 — unreachable at realistic valuations");
    require!(total_price > 0, AssetraError::InvalidPurchaseAmount);

    let balance_before = ctx.accounts.investor_token_account.amount;
    let is_new_position = ctx.accounts.holder_position.track == Pubkey::default();
    let acc_now = ctx.accounts.revenue_pool.accumulated_rewards_per_token;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            SystemTransfer {
                from: ctx.accounts.investor.to_account_info(),
                to: ctx.accounts.creator.to_account_info(),
            },
        ),
        total_price,
    )?;
    let creator_key = track.creator;
    let track_id_bytes = track.track_id.to_le_bytes();
    let bump = track.bump;
    let signer_seeds: &[&[u8]] = &[
        Track::SEED_PREFIX,
        creator_key.as_ref(),
        track_id_bytes.as_ref(),
        &[bump],
    ];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.investor_token_account.to_account_info(),
                authority: ctx.accounts.track.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
        0, // decimals — matches mint_track_tokens.rs
    )?;

    let position = &mut ctx.accounts.holder_position;
    if is_new_position {
        position.track = track.key();
        position.holder = ctx.accounts.investor.key();
        position.unclaimed_rewards = 0;
        position.total_claimed = 0;
        position.bump = ctx.bumps.holder_position;
    } else {
        position.settle(balance_before, acc_now);
    }

    let new_balance = balance_before
        .checked_add(amount)
        .expect("post-purchase balance overflow");
    position.reward_debt = (new_balance as u128)
        .checked_mul(acc_now)
        .expect("reward_debt overflow");

    Ok(())
}
