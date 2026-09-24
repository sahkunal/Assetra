// anchor_lang::declare_program!(assetra);
mod common;

use anchor_lang::system_program;
use litesvm_utils::Signer;

use common::{creator_profile_pda, track_pda};

#[test]
fn register_creator_succeeds_and_sets_expected_fields() {
    let mut ctx = common::new_ctx();
    let authority = common::funded_account(&mut ctx, 10_000_000_000);
    let creator_profile = creator_profile_pda(&ctx.program_id, &authority.pubkey());

    let ix = ctx
        .program()
        .accounts(assetra::accounts::RegisterCreator {
            authority: authority.pubkey(),
            creator_profile,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::RegisterCreator {
            display_name: "Test Artist".to_string(),
            rights_attestation: "I own the master and publishing for my catalog.".to_string(),
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&authority])
        .unwrap()
        .assert_success();

    let profile: ::assetra::state::CreatorProfile = ctx.get_account(&creator_profile).unwrap();
    assert_eq!(profile.authority, authority.pubkey());
    assert_eq!(profile.display_name, "Test Artist");
    // assert!(profile.self_attested);
    assert!(!profile.admin_verified);
    assert_eq!(profile.track_count, 0);
}

#[test]
fn register_creator_rejects_oversized_display_name() {
    let mut ctx = common::new_ctx();
    let authority = common::funded_account(&mut ctx, 10_000_000_000);
    let creator_profile = creator_profile_pda(&ctx.program_id, &authority.pubkey());

    let too_long = "x".repeat(::assetra::constants::MAX_DISPLAY_NAME_LEN + 1);

    let ix = ctx
        .program()
        .accounts(assetra::accounts::RegisterCreator {
            authority: authority.pubkey(),
            creator_profile,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::RegisterCreator {
            display_name: too_long,
            rights_attestation: "ok".to_string(),
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&authority]);
    assert!(result.is_err(), "expected oversized display_name to be rejected");
}

#[test]
fn create_track_sets_metadata_and_tokenization_in_one_call() {
    let mut ctx = common::new_ctx();
    let authority = common::funded_account(&mut ctx, 10_000_000_000);
    let creator_profile = creator_profile_pda(&ctx.program_id, &authority.pubkey());

    ctx.execute_instruction(
        ctx.program()
            .accounts(assetra::accounts::RegisterCreator {
                authority: authority.pubkey(),
                creator_profile,
                system_program: system_program::ID,
            })
            .args(assetra::instruction::RegisterCreator {
                display_name: "Test Artist".to_string(),
                rights_attestation: "self-attested".to_string(),
            })
            .instruction()
            .unwrap(),
        &[&authority],
    )
    .unwrap()
    .assert_success();

    let track = track_pda(&ctx.program_id, &authority.pubkey(), 0);

    ctx.execute_instruction(
        ctx.program()
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
                tokenized_bps: 2000,
                valuation_multiple_bps: 500,
                token_supply: 100_000,
            })
            .instruction()
            .unwrap(),
        &[&authority],
    )
    .unwrap()
    .assert_success();

    let track_state: ::assetra::state::Track = ctx.get_account(&track).unwrap();
    assert_eq!(track_state.track_id, 0);
    assert_eq!(track_state.title, "Midnight Drive");
    assert_eq!(track_state.tokenized_bps, 2000);
    assert_eq!(track_state.valuation_multiple_bps, 500);
    assert_eq!(track_state.token_supply, 100_000);
    assert_eq!(track_state.mint, None);
    assert_eq!(track_state.status, ::assetra::state::TrackStatus::Configured);

    let profile: ::assetra::state::CreatorProfile = ctx.get_account(&creator_profile).unwrap();
    assert_eq!(profile.track_count, 1);
}

#[test]
fn create_track_rejects_zero_tokenized_bps() {
    let mut ctx = common::new_ctx();
    let authority = common::funded_account(&mut ctx, 10_000_000_000);
    let creator_profile = creator_profile_pda(&ctx.program_id, &authority.pubkey());

    ctx.execute_instruction(
        ctx.program()
            .accounts(assetra::accounts::RegisterCreator {
                authority: authority.pubkey(),
                creator_profile,
                system_program: system_program::ID,
            })
            .args(assetra::instruction::RegisterCreator {
                display_name: "Test Artist".to_string(),
                rights_attestation: "self-attested".to_string(),
            })
            .instruction()
            .unwrap(),
        &[&authority],
    )
    .unwrap()
    .assert_success();

    let track = track_pda(&ctx.program_id, &authority.pubkey(), 0);

    let ix = ctx
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
            cover_art_uri: "uri".to_string(),
            audio_preview_uri: "uri".to_string(),
            declared_annual_revenue: 500_000,
            tokenized_bps: 0, // invalid
            valuation_multiple_bps: 500,
            token_supply: 100_000,
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&authority]);
    assert!(result.is_err(), "expected zero tokenized_bps to be rejected");
}
