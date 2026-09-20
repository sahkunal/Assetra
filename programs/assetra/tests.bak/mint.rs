mod common;

use anchor_lang::AccountDeserialize;
use assetra::state::{Track, TrackStatus};
use anchor_lang::prelude::Pubkey;

use common::{create_track_ix, mint_track_tokens_ix, register_creator_ix, TestContext, TOKEN_2022_PROGRAM_ID};

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

fn mint_pda(program_id: &Pubkey, track: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[assetra::constants::SEED_MINT, track.as_ref()], program_id)
}

fn vault_pda(program_id: &Pubkey, track: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[assetra::constants::SEED_VAULT, track.as_ref()], program_id)
}

/// Registers a creator and one fully-configured track (metadata +
/// tokenization economics set together, per the merged create_track),
/// returning the track PDA — shared setup for the minting tests below.
fn setup_configured_track(ctx: &mut TestContext) -> Pubkey {
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
            "uri".to_string(),
            "uri".to_string(),
            500_000,
            2000,
            500,
            100_000,
        ),
        &[],
    );

    track
}

#[test]
fn mint_track_tokens_mints_full_supply_into_vault() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let track = setup_configured_track(&mut ctx);

    let (mint, _) = mint_pda(&ctx.program_id, &track);
    let (vault, _) = vault_pda(&ctx.program_id, &track);

    ctx.send_ix(
        mint_track_tokens_ix(ctx.program_id, authority, track, mint, vault),
        &[],
    );

    let track_account = ctx.svm.get_account(&track).unwrap();
    let track_state = Track::try_deserialize(&mut track_account.data.as_slice()).unwrap();
    assert_eq!(track_state.mint, Some(mint));
    assert_eq!(track_state.vault, Some(vault));
    assert_eq!(track_state.status, TrackStatus::Minted);

    // Confirm the vault actually received the full configured supply, not
    // just that the instruction succeeded.
    let vault_account = ctx.svm.get_account(&vault).unwrap();
    assert_eq!(vault_account.owner, TOKEN_2022_PROGRAM_ID);
    let amount = u64::from_le_bytes(vault_account.data[64..72].try_into().unwrap());
    assert_eq!(amount, 100_000);
}

#[test]
fn mint_track_tokens_rejects_running_twice() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let track = setup_configured_track(&mut ctx);

    let (mint, _) = mint_pda(&ctx.program_id, &track);
    let (vault, _) = vault_pda(&ctx.program_id, &track);

    ctx.send_ix(
        mint_track_tokens_ix(ctx.program_id, authority, track, mint, vault),
        &[],
    );

    // mint/vault PDAs already exist now — `init` on the second attempt
    // fails before the status check even runs, which is fine: either
    // failure mode correctly prevents minting twice.
    let err = ctx.send_ix_expect_err(
        mint_track_tokens_ix(ctx.program_id, authority, track, mint, vault),
        &[],
    );
    assert!(!err.is_empty());
}
