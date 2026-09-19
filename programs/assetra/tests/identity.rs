mod common;

use anchor_lang::AccountDeserialize;
use assetra::state::{CreatorProfile, Track, TrackStatus};
use solana_sdk::{pubkey::Pubkey, signature::Signer};

use common::{create_track_ix, register_creator_ix, TestContext};

fn creator_profile_pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[assetra::constants::SEED_CREATOR_PROFILE, authority.as_ref()],
        program_id,
    )
}

fn track_pda(program_id: &Pubkey, authority: &Pubkey, track_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            assetra::constants::SEED_TRACK,
            authority.as_ref(),
            &track_id.to_le_bytes(),
        ],
        program_id,
    )
}

#[test]
fn register_creator_succeeds_and_sets_expected_fields() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (creator_profile, _bump) = creator_profile_pda(&ctx.program_id, &authority);

    let ix = register_creator_ix(
        ctx.program_id,
        authority,
        creator_profile,
        "Test Artist".to_string(),
        "I own the master and publishing for my catalog.".to_string(),
    );
    ctx.send_ix(ix, &[]);

    let account = ctx.svm.get_account(&creator_profile).unwrap();
    let profile = CreatorProfile::try_deserialize(&mut account.data.as_slice()).unwrap();

    assert_eq!(profile.authority, authority);
    assert_eq!(profile.display_name, "Test Artist");
    assert_eq!(profile.self_attested, "I own the master and publishing for my catalog.");
    assert!(profile.admin_verified.is_empty()); // never auto-set — matches the design note
    assert_eq!(profile.track_count, 0);
}

#[test]
fn register_creator_rejects_oversized_display_name() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (creator_profile, _bump) = creator_profile_pda(&ctx.program_id, &authority);

    let too_long = "x".repeat(assetra::constants::MAX_DISPLAY_NAME_LEN + 1);
    let ix = register_creator_ix(ctx.program_id, authority, creator_profile, too_long, "ok".to_string());

    let err = ctx.send_ix_expect_err(ix, &[]);
    assert!(
        err.contains("DisplayNameTooLong") || err.contains("6000") || err.contains("custom program error"),
        "unexpected error: {err}"
    );
}

#[test]
fn create_track_sets_metadata_and_tokenization_in_one_call() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (creator_profile, _) = creator_profile_pda(&ctx.program_id, &authority);

    ctx.send_ix(
        register_creator_ix(
            ctx.program_id,
            authority,
            creator_profile,
            "Test Artist".to_string(),
            "self-attested".to_string(),
        ),
        &[],
    );

    let (track, _) = track_pda(&ctx.program_id, &authority, 0);
    ctx.send_ix(
        create_track_ix(
            ctx.program_id,
            authority,
            creator_profile,
            track,
            "Midnight Drive".to_string(),
            "USRC17600001".to_string(),
            "https://example.com/cover.png".to_string(),
            "https://example.com/preview.mp3".to_string(),
            500_000,
            2000,  // 20% tokenized
            500,   // 5.00x valuation multiple
            100_000, // token supply
        ),
        &[],
    );

    let track_account = ctx.svm.get_account(&track).unwrap();
    let track_state = Track::try_deserialize(&mut track_account.data.as_slice()).unwrap();

    assert_eq!(track_state.track_id, 0);
    assert_eq!(track_state.title, "Midnight Drive");
    assert_eq!(track_state.tokenized_bps, 2000);
    assert_eq!(track_state.valuation_multiple_bps, 500);
    assert_eq!(track_state.token_supply, 100_000);
    assert_eq!(track_state.mint, None); // Phase 4 still separate
    assert_eq!(track_state.status, TrackStatus::Configured);

    let profile_account = ctx.svm.get_account(&creator_profile).unwrap();
    let profile = CreatorProfile::try_deserialize(&mut profile_account.data.as_slice()).unwrap();
    assert_eq!(profile.track_count, 1);
}

#[test]
fn create_track_rejects_zero_tokenized_bps() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (creator_profile, _) = creator_profile_pda(&ctx.program_id, &authority);

    ctx.send_ix(
        register_creator_ix(
            ctx.program_id,
            authority,
            creator_profile,
            "Test Artist".to_string(),
            "self-attested".to_string(),
        ),
        &[],
    );

    let (track, _) = track_pda(&ctx.program_id, &authority, 0);
    let err = ctx.send_ix_expect_err(
        create_track_ix(
            ctx.program_id,
            authority,
            creator_profile,
            track,
            "Midnight Drive".to_string(),
            "USRC17600001".to_string(),
            "uri".to_string(),
            "uri".to_string(),
            500_000,
            0, // invalid — must be > 0
            500,
            100_000,
        ),
        &[],
    );
    assert!(err.contains("InvalidTokenizedBps") || err.contains("custom program error"));
}

#[test]
fn second_track_from_same_creator_gets_distinct_pda() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (creator_profile, _) = creator_profile_pda(&ctx.program_id, &authority);

    ctx.send_ix(
        register_creator_ix(
            ctx.program_id,
            authority,
            creator_profile,
            "Test Artist".to_string(),
            "self-attested".to_string(),
        ),
        &[],
    );

    let (track_0, _) = track_pda(&ctx.program_id, &authority, 0);
    ctx.send_ix(
        create_track_ix(
            ctx.program_id,
            authority,
            creator_profile,
            track_0,
            "Track One".to_string(),
            "USRC17600001".to_string(),
            "uri".to_string(),
            "uri".to_string(),
            100_000,
            2000,
            500,
            50_000,
        ),
        &[],
    );

    let (track_1, _) = track_pda(&ctx.program_id, &authority, 1);
    assert_ne!(track_0, track_1, "sequential track_ids must derive distinct PDAs");

    ctx.send_ix(
        create_track_ix(
            ctx.program_id,
            authority,
            creator_profile,
            track_1,
            "Track Two".to_string(),
            "USRC17600002".to_string(),
            "uri".to_string(),
            "uri".to_string(),
            200_000,
            1500,
            600,
            75_000,
        ),
        &[],
    );

    let profile_account = ctx.svm.get_account(&creator_profile).unwrap();
    let profile = CreatorProfile::try_deserialize(&mut profile_account.data.as_slice()).unwrap();
    assert_eq!(profile.track_count, 2);
}
