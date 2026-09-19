mod common;

use anchor_lang::AccountDeserialize;
use assetra::state::{HolderPosition, RevenuePool};
use solana_sdk::{pubkey::Pubkey, signature::Signer};

use common::{
    claim_revenue_ix, create_track_ix, deposit_revenue_ix, initialize_revenue_pool_ix,
    mint_track_tokens_ix, open_holder_position_ix, register_creator_ix, seed_token_account,
    TestContext,
};

fn creator_profile_pda(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[assetra::constants::SEED_CREATOR_PROFILE, authority.as_ref()],
        program_id,
    )
}
fn track_pda(program_id: &Pubkey, authority: &Pubkey, track_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[assetra::constants::SEED_TRACK, authority.as_ref(), &track_id.to_le_bytes()],
        program_id,
    )
}
fn mint_pda(program_id: &Pubkey, track: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[assetra::constants::SEED_MINT, track.as_ref()], program_id)
}
fn vault_pda(program_id: &Pubkey, track: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[assetra::constants::SEED_VAULT, track.as_ref()], program_id)
}
fn revenue_pool_pda(program_id: &Pubkey, track: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[assetra::constants::SEED_REVENUE_POOL, track.as_ref()],
        program_id,
    )
}
fn holder_position_pda(program_id: &Pubkey, track: &Pubkey, holder: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[assetra::constants::SEED_HOLDER_POSITION, track.as_ref(), holder.as_ref()],
        program_id,
    )
}

fn setup_minted_track_with_pool(ctx: &mut TestContext) -> (Pubkey, Pubkey) {
    let authority = ctx.payer.pubkey();
    let (creator_profile, _) = creator_profile_pda(&ctx.program_id, &authority);

    ctx.send_ix(
        register_creator_ix(ctx.program_id, authority, creator_profile, "Artist".to_string(), "self-attested".to_string()),
        &[],
    );

    let (track, _) = track_pda(&ctx.program_id, &authority, 0);
    ctx.send_ix(
        create_track_ix(
            ctx.program_id, authority, creator_profile, track,
            "Track".to_string(), "USRC17600001".to_string(), "uri".to_string(), "uri".to_string(),
            500_000, 2000, 500, 100_000,
        ),
        &[],
    );

    let (mint, _) = mint_pda(&ctx.program_id, &track);
    let (vault, _) = vault_pda(&ctx.program_id, &track);
    ctx.send_ix(mint_track_tokens_ix(ctx.program_id, authority, track, mint, vault), &[]);

    let (revenue_pool, _) = revenue_pool_pda(&ctx.program_id, &track);
    ctx.send_ix(initialize_revenue_pool_ix(ctx.program_id, authority, track, revenue_pool), &[]);

    (track, mint)
}

#[test]
fn deposit_revenue_updates_accumulator_correctly() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (track, _mint) = setup_minted_track_with_pool(&mut ctx);
    let (revenue_pool, _) = revenue_pool_pda(&ctx.program_id, &track);

    // 100,000 token supply. Deposit 1,000,000 lamports.
    // increment = 1_000_000 * 10^12 / 100_000 = 10^13
    ctx.send_ix(deposit_revenue_ix(ctx.program_id, authority, track, revenue_pool, 1_000_000), &[]);

    let account = ctx.svm.get_account(&revenue_pool).unwrap();
    let pool = RevenuePool::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(pool.total_deposited, 1_000_000);
    assert_eq!(pool.accumulated_rewards_per_token, 10_000_000_000_000u128);
}

#[test]
fn holder_position_checkpoints_to_current_accumulator_not_zero() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (track, mint) = setup_minted_track_with_pool(&mut ctx);
    let (revenue_pool, _) = revenue_pool_pda(&ctx.program_id, &track);

    // A deposit happens BEFORE this holder ever acquires tokens.
    ctx.send_ix(deposit_revenue_ix(ctx.program_id, authority, track, revenue_pool, 1_000_000), &[]);

    let holder = solana_sdk::signature::Keypair::new();
    ctx.svm.airdrop(&holder.pubkey(), 1_000_000_000).unwrap();

    let holder_token_account = Pubkey::new_unique();
    common::seed_token_account(&mut ctx.svm, holder_token_account, mint, holder.pubkey(), 10_000);

    let (holder_position, _) = holder_position_pda(&ctx.program_id, &track, &holder.pubkey());
    ctx.send_ix(
        open_holder_position_ix(
            ctx.program_id, holder.pubkey(), track, holder_token_account,
            revenue_pool, holder_position, mint,
        ),
        &[&holder],
    );

    let account = ctx.svm.get_account(&holder_position).unwrap();
    let position = HolderPosition::try_deserialize(&mut account.data.as_slice()).unwrap();

    // Checkpointed to the CURRENT accumulator (post the pre-existing
    // deposit), not 0 — this holder must NOT be able to claim the deposit
    // that happened before they held anything.
    let pool_account = ctx.svm.get_account(&revenue_pool).unwrap();
    let pool = RevenuePool::try_deserialize(&mut pool_account.data.as_slice()).unwrap();
    assert_eq!(position.last_accumulated_rewards_per_token, pool.accumulated_rewards_per_token);
    assert_ne!(position.last_accumulated_rewards_per_token, 0);
}

#[test]
fn claim_revenue_pays_correct_pro_rata_share() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (track, mint) = setup_minted_track_with_pool(&mut ctx);
    let (revenue_pool, _) = revenue_pool_pda(&ctx.program_id, &track);

    let holder = solana_sdk::signature::Keypair::new();
    ctx.svm.airdrop(&holder.pubkey(), 1_000_000_000).unwrap();
    let holder_token_account = Pubkey::new_unique();
    // Holder owns 10,000 of the 100,000 supply (10%).
    common::seed_token_account(&mut ctx.svm, holder_token_account, mint, holder.pubkey(), 10_000);

    let (holder_position, _) = holder_position_pda(&ctx.program_id, &track, &holder.pubkey());
    ctx.send_ix(
        open_holder_position_ix(
            ctx.program_id, holder.pubkey(), track, holder_token_account,
            revenue_pool, holder_position, mint,
        ),
        &[&holder],
    );

    ctx.send_ix(deposit_revenue_ix(ctx.program_id, authority, track, revenue_pool, 1_000_000), &[]);

    let holder_balance_before = ctx.svm.get_balance(&holder.pubkey()).unwrap();

    ctx.send_ix(
        claim_revenue_ix(ctx.program_id, holder.pubkey(), track, holder_token_account, revenue_pool, holder_position),
        &[&holder],
    );

    let holder_balance_after = ctx.svm.get_balance(&holder.pubkey()).unwrap();
    assert_eq!(holder_balance_after - holder_balance_before, 100_000);

    let position_account = ctx.svm.get_account(&holder_position).unwrap();
    let position = HolderPosition::try_deserialize(&mut position_account.data.as_slice()).unwrap();
    assert_eq!(position.total_claimed, 100_000);
}

#[test]
fn claim_revenue_rejects_second_claim_with_nothing_new_accrued() {
    let mut ctx = TestContext::new();
    let authority = ctx.payer.pubkey();
    let (track, mint) = setup_minted_track_with_pool(&mut ctx);
    let (revenue_pool, _) = revenue_pool_pda(&ctx.program_id, &track);

    let holder = solana_sdk::signature::Keypair::new();
    ctx.svm.airdrop(&holder.pubkey(), 1_000_000_000).unwrap();
    let holder_token_account = Pubkey::new_unique();
    common::seed_token_account(&mut ctx.svm, holder_token_account, mint, holder.pubkey(), 10_000);

    let (holder_position, _) = holder_position_pda(&ctx.program_id, &track, &holder.pubkey());
    ctx.send_ix(
        open_holder_position_ix(ctx.program_id, holder.pubkey(), track, holder_token_account, revenue_pool, holder_position, mint),
        &[&holder],
    );
    ctx.send_ix(deposit_revenue_ix(ctx.program_id, authority, track, revenue_pool, 1_000_000), &[]);
    ctx.send_ix(
        claim_revenue_ix(ctx.program_id, holder.pubkey(), track, holder_token_account, revenue_pool, holder_position),
        &[&holder],
    );

    let err = ctx.send_ix_expect_err(
        claim_revenue_ix(ctx.program_id, holder.pubkey(), track, holder_token_account, revenue_pool, holder_position),
        &[&holder],
    );
    assert!(err.contains("NothingToClaim") || err.contains("custom program error"));
}
