
mod common;

use anchor_lang::system_program;
use anchor_litesvm::AnchorContext;
use litesvm_utils::Signer;

use common::{
    creator_profile_pda, mint_pda, revenue_pool_pda, track_pda, vault_pda,
};

const INITIAL_FUNDS: u64 = 10_000_000_000;
const TOKEN_SUPPLY: u64 = 100_000;

fn setup_configured_track() -> (
    AnchorContext,
    litesvm_utils::Keypair,
    litesvm_utils::Pubkey,
) {
    let mut ctx = common::new_ctx();
    let authority = common::funded_account(&mut ctx, INITIAL_FUNDS);

    let creator_profile = creator_profile_pda(&ctx.program_id, &authority.pubkey());

    let register_ix = ctx
        .program()
        .accounts(assetra::accounts::RegisterCreator {
            authority: authority.pubkey(),
            creator_profile,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::RegisterCreator {
            display_name: "Test Artist".to_string(),
            rights_attestation: "I own the rights.".to_string(),
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(register_ix, &[&authority])
        .unwrap()
        .assert_success();

    let track = track_pda(&ctx.program_id, &authority.pubkey(), 0);

    let create_ix = ctx
        .program()
        .accounts(assetra::accounts::CreateTrack {
            authority: authority.pubkey(),
            creator_profile,
            track,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::CreateTrack {
            title: "Midnight Drive".to_string(),
            isrc: "USRC17600001".to_string(),
            cover_art_uri: "https://example.com/cover.png".to_string(),
            audio_preview_uri: "https://example.com/preview.mp3".to_string(),
            declared_annual_revenue: 500_000,
            tokenized_bps: 2_000,
            valuation_multiple_bps: 500,
            token_supply: TOKEN_SUPPLY,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(create_ix, &[&authority])
        .unwrap()
        .assert_success();

    (ctx, authority, track)
}

fn mint_track(
    ctx: &mut AnchorContext,
    authority: &litesvm_utils::Keypair,
    track: litesvm_utils::Pubkey,
) {
    common::verify_track(ctx, track, 500_000);

    let mint = mint_pda(&ctx.program_id, &track);
    let vault = vault_pda(&ctx.program_id, &track);

    let ix = ctx
        .program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: authority.pubkey(),
            track,
            mint,
            vault,
            token_program: common::token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[authority])
        .unwrap()
        .assert_success();
}

#[test]
fn mint_track_tokens_sets_minted_status_and_accounts() {
    let (mut ctx, authority, track) = setup_configured_track();

    mint_track(&mut ctx, &authority, track);

    let state: ::assetra::state::Track = ctx.get_account(&track).unwrap();

    assert_eq!(state.status, ::assetra::state::TrackStatus::Minted);
    assert_eq!(state.mint, Some(mint_pda(&ctx.program_id, &track)));
    assert_eq!(state.vault, Some(vault_pda(&ctx.program_id, &track)));
}

#[test]
fn mint_track_tokens_rejects_second_mint() {
    let (mut ctx, authority, track) = setup_configured_track();

    mint_track(&mut ctx, &authority, track);

    let mint = mint_pda(&ctx.program_id, &track);
    let vault = vault_pda(&ctx.program_id, &track);

    let ix = ctx
        .program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: authority.pubkey(),
            track,
            mint,
            vault,
            token_program: common::token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction()
        .unwrap();

    // The transaction can be returned successfully by the helper even when
    // the on-chain instruction fails. Assert the transaction result itself.
    ctx.execute_instruction(ix, &[&authority])
        .unwrap()
        .assert_failure();
}

#[test]
fn initialize_revenue_pool_starts_with_zero_rewards() {
    let (mut ctx, authority, track) = setup_configured_track();

    mint_track(&mut ctx, &authority, track);

    let revenue_pool = revenue_pool_pda(&ctx.program_id, &track);

    let ix = ctx
        .program()
        .accounts(assetra::accounts::InitializeRevenuePool {
            authority: authority.pubkey(),
            track,
            revenue_pool,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::InitializeRevenuePool {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&authority])
        .unwrap()
        .assert_success();

    let pool: ::assetra::state::RevenuePool =
        ctx.get_account(&revenue_pool).unwrap();

    assert_eq!(pool.track, track);
    assert_eq!(pool.total_deposited, 0);
    assert_eq!(pool.accumulated_rewards_per_token, 0);
}
