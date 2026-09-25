use anchor_lang::prelude::*;
use anchor_lang:: system_program::{self, Transfer as SystemTransfer};
use anchor_spl:: associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::constants::*;
use crate::state::*;
use crate::errors::AssetraError;

#[derive(Accounts)]
pub struct BuyResale<'info>{
    #[account(mut)]
    pub buyer: Signer<'info>,

    ///CHECK: 
    #[account(mut,
    address = listing.seller)]
    pub seller: UncheckedAccount<'info>,

    #[account(
        seeds=[
            Track::SEED_PREFIX,
            track.creator.as_ref(),
            track.track_id.to_le_bytes().as_ref(),
        ],
        bump= track.bump,
    )]
    pub track:Account<'info, Track>,

    #[account(
        mut,
        token::mint= mint,
        token::authority= seller,
        token::token_program= token_program,
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer= buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program,
        constraint = buyer_token_account.key() != seller_token_account.key() @ AssetraError::Unauthorized,
    )]
    pub buyer_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds= [RevenuePool::SEED_PREFIX, track.key().as_ref()],
        bump= revenue_pool.bump,
        constraint= revenue_pool.track== track.key(),
    )]
    pub revenue_pool: Account<'info, RevenuePool>,

    #[account(
        mut,
        seeds = [SEED_HOLDER_POSITION, track.key().as_ref(), seller.key().as_ref()],
        bump = seller_holder_position.bump,
        constraint = seller_holder_position.holder == seller.key(),
        constraint = seller_holder_position.track == track.key(),
    )]
    pub seller_holder_position: Account<'info, HolderPosition>,

    #[account(
        init_if_needed,
        payer = buyer,
        space = HolderPosition::SPACE,
        seeds = [SEED_HOLDER_POSITION, track.key().as_ref(), buyer.key().as_ref()],
        bump,
    )]
    pub buyer_holder_position: Account<'info, HolderPosition>,

    #[account(
        mut,
        seeds = [Listing::SEED_PREFIX, track.key().as_ref(), seller.key().as_ref(), listing.nonce.to_le_bytes().as_ref()],
        bump = listing.bump,
        constraint = listing.track == track.key(),
        constraint = listing.mint == mint.key(),
        close = seller,
    )]
    pub listing: Account<'info, Listing>,
    

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


pub fn handler(ctx: Context<BuyResale>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let amount = listing.amount;

    let total_price: u64 = (listing.price_per_token as u128)
        .checked_mul(amount as u128)
        .expect("resale total price overflow")
        .try_into()
        .expect("resale total price exceeds u64 — unreachable at realistic prices");

    // ---- capture pre-trade balances BEFORE either transfer runs ----
    let seller_balance_before = ctx.accounts.seller_token_account.amount;
    let buyer_balance_before = ctx.accounts.buyer_token_account.amount;
    let buyer_position_is_new = ctx.accounts.buyer_holder_position.track == Pubkey::default();
    let acc_now = ctx.accounts.revenue_pool.accumulated_rewards_per_token;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.key(),
            SystemTransfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        total_price,
    )?;
    let track_key = ctx.accounts.track.key();
    let seller_key = ctx.accounts.seller.key();
    let nonce_bytes = listing.nonce.to_le_bytes();
    let listing_bump = listing.bump;
    let signer_seeds: &[&[u8]] = &[
        Listing::SEED_PREFIX,
        track_key.as_ref(),
        seller_key.as_ref(),
        nonce_bytes.as_ref(),
        &[listing_bump],
    ];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.seller_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
        0, // decimals — matches mint_track_tokens.rs
    )?;

    let seller_position = &mut ctx.accounts.seller_holder_position;
    seller_position.settle(seller_balance_before, acc_now);
    let seller_new_balance = seller_balance_before
        .checked_sub(amount)
        .expect("seller sold more than they held — unreachable, enforced by delegate approval");
    seller_position.reward_debt = (seller_new_balance as u128)
        .checked_mul(acc_now)
        .expect("seller reward_debt overflow");

    let buyer_position = &mut ctx.accounts.buyer_holder_position;
    if buyer_position_is_new {
        buyer_position.track = track_key;
        buyer_position.holder = ctx.accounts.buyer.key();
        buyer_position.unclaimed_rewards = 0;
        buyer_position.total_claimed = 0;
        buyer_position.bump = ctx.bumps.buyer_holder_position;
    } else {
        buyer_position.settle(buyer_balance_before, acc_now);
    }
    let buyer_new_balance = buyer_balance_before
        .checked_add(amount)
        .expect("buyer post-trade balance overflow");
    buyer_position.reward_debt = (buyer_new_balance as u128)
        .checked_mul(acc_now)
        .expect("buyer reward_debt overflow");

    Ok(())
}
