mod common;

use anchor_lang::system_program;
use anchor_litesvm::AnchorContext;
use litesvm_utils::{Keypair, Pubkey, Signer};

use common::{config_pda, creator_profile_pda, mint_pda, track_pda, vault_pda};

const INITIAL_FUNDS: u64 = 10_000_000_000;

/// Registers a creator and creates (but does not verify) one track.
fn setup_configured_track() -> (AnchorContext, Keypair, Pubkey) {
    let mut ctx = common::new_ctx();
    let creator = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let creator_profile = creator_profile_pda(&ctx.program_id, &creator.pubkey());

    let ix = ctx
        .program()
        .accounts(assetra::accounts::RegisterCreator {
            authority: creator.pubkey(),
            creator_profile,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::RegisterCreator {
            display_name: "Test Artist".to_string(),
            rights_attestation: "I own the rights.".to_string(),
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();

    let track = track_pda(&ctx.program_id, &creator.pubkey(), 0);
    let ix = ctx
        .program()
        .accounts(assetra::accounts::CreateTrack {
            authority: creator.pubkey(),
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
            token_supply: 100_000,
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();

    (ctx, creator, track)
}

fn mint_ix(
    ctx: &AnchorContext,
    creator: &Keypair,
    track: Pubkey,
) -> anchor_lang::solana_program::instruction::Instruction {
    ctx.program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: creator.pubkey(),
            track,
            mint: mint_pda(&ctx.program_id, &track),
            vault: vault_pda(&ctx.program_id, &track),
            token_program: common::token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction()
        .unwrap()
}

#[test]
fn initialize_config_stores_admin_and_verifier() {
    let mut ctx = common::new_ctx();
    let admin = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);

    common::init_config(&mut ctx, &admin, verifier.pubkey());

    let config: ::assetra::state::ProgramConfig =
        ctx.get_account(&config_pda(&ctx.program_id)).unwrap();
    assert_eq!(config.admin, admin.pubkey());
    assert_eq!(config.verifier, verifier.pubkey());
}

#[test]
fn verify_track_marks_verified_and_records_revenue() {
    let (mut ctx, _creator, track) = setup_configured_track();
    let verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &verifier, verifier.pubkey());

    // Verifier vouches for less than the creator declared.
    let ix = common::verify_track_ix(&ctx, &verifier, track, 400_000);
    ctx.execute_instruction(ix, &[&verifier])
        .unwrap()
        .assert_success();

    let state: ::assetra::state::Track = ctx.get_account(&track).unwrap();
    assert_eq!(state.status, ::assetra::state::TrackStatus::Verified);
    assert_eq!(state.declared_annual_revenue, 500_000);
    assert_eq!(state.verified_annual_revenue, 400_000);
}

#[test]
fn verify_track_rejects_non_verifier() {
    let (mut ctx, creator, track) = setup_configured_track();
    let verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &verifier, verifier.pubkey());

    // The creator cannot self-verify.
    let ix = common::verify_track_ix(&ctx, &creator, track, 500_000);
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_anchor_error("Unauthorized");
}

#[test]
fn verify_track_rejects_zero_revenue() {
    let (mut ctx, _creator, track) = setup_configured_track();
    let verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &verifier, verifier.pubkey());

    let ix = common::verify_track_ix(&ctx, &verifier, track, 0);
    ctx.execute_instruction(ix, &[&verifier])
        .unwrap()
        .assert_anchor_error("InvalidVerifiedRevenue");
}

#[test]
fn verify_track_rejects_second_verification() {
    let (mut ctx, _creator, track) = setup_configured_track();
    let verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &verifier, verifier.pubkey());

    let ix = common::verify_track_ix(&ctx, &verifier, track, 500_000);
    ctx.execute_instruction(ix, &[&verifier])
        .unwrap()
        .assert_success();

    let ix = common::verify_track_ix(&ctx, &verifier, track, 900_000);
    ctx.execute_instruction(ix, &[&verifier])
        .unwrap()
        .assert_anchor_error("InvalidTrackStatus");
}

#[test]
fn mint_track_tokens_rejects_unverified_track() {
    let (mut ctx, creator, track) = setup_configured_track();

    let ix = mint_ix(&ctx, &creator, track);
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_anchor_error("InvalidTrackStatus");
}

#[test]
fn set_verifier_rotates_key_and_locks_out_old_one() {
    let (mut ctx, _creator, track) = setup_configured_track();
    let admin = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let old_verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let new_verifier = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &admin, old_verifier.pubkey());

    let ix = ctx
        .program()
        .accounts(assetra::accounts::SetVerifier {
            admin: admin.pubkey(),
            config: config_pda(&ctx.program_id),
        })
        .args(assetra::instruction::SetVerifier {
            new_verifier: new_verifier.pubkey(),
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    let ix = common::verify_track_ix(&ctx, &old_verifier, track, 500_000);
    ctx.execute_instruction(ix, &[&old_verifier])
        .unwrap()
        .assert_anchor_error("Unauthorized");

    let ix = common::verify_track_ix(&ctx, &new_verifier, track, 500_000);
    ctx.execute_instruction(ix, &[&new_verifier])
        .unwrap()
        .assert_success();
}

#[test]
fn set_verifier_rejects_non_admin() {
    let mut ctx = common::new_ctx();
    let admin = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let attacker = common::funded_account(&mut ctx, INITIAL_FUNDS);
    common::init_config(&mut ctx, &admin, admin.pubkey());

    let ix = ctx
        .program()
        .accounts(assetra::accounts::SetVerifier {
            admin: attacker.pubkey(),
            config: config_pda(&ctx.program_id),
        })
        .args(assetra::instruction::SetVerifier {
            new_verifier: attacker.pubkey(),
        })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&attacker])
        .unwrap()
        .assert_anchor_error("Unauthorized");
}
