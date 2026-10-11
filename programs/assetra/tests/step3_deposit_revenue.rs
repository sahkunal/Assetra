
mod common;

use anchor_lang::system_program;
use anchor_litesvm::AnchorContext;
use litesvm_utils::{Keypair, Pubkey, Signer};

use common::{
    creator_profile_pda, mint_pda, revenue_pool_pda, track_pda, vault_pda,
};

const INITIAL_FUNDS: u64 = 10_000_000_000;
const TOKEN_SUPPLY: u64 = 100_000;

fn setup_pool() -> (AnchorContext, Keypair, Pubkey, Pubkey) {
    let mut ctx = common::new_ctx();
    let creator = common::funded_account(&mut ctx, INITIAL_FUNDS);

    let creator_profile = creator_profile_pda(&ctx.program_id, &creator.pubkey());

    let register_ix = ctx
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

    ctx.execute_instruction(register_ix, &[&creator])
        .unwrap()
        .assert_success();

    let track = track_pda(&ctx.program_id, &creator.pubkey(), 0);

    let create_ix = ctx
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
            token_supply: TOKEN_SUPPLY,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(create_ix, &[&creator])
        .unwrap()
        .assert_success();

    let mint = mint_pda(&ctx.program_id, &track);
    let vault = vault_pda(&ctx.program_id, &track);

    let mint_ix = ctx
        .program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: creator.pubkey(),
            track,
            mint,
            vault,
            token_program: common::token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(mint_ix, &[&creator])
        .unwrap()
        .assert_success();

    let revenue_pool = revenue_pool_pda(&ctx.program_id, &track);

    let pool_ix = ctx
        .program()
        .accounts(assetra::accounts::InitializeRevenuePool {
            authority: creator.pubkey(),
            track,
            revenue_pool,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::InitializeRevenuePool {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(pool_ix, &[&creator])
        .unwrap()
        .assert_success();

    (ctx, creator, track, revenue_pool)
}

fn deposit_ix(
    ctx: &AnchorContext,
    authority: &Keypair,
    track: Pubkey,
    revenue_pool: Pubkey,
    amount: u64,
) -> anchor_lang::solana_program::instruction::Instruction {
    ctx.program()
        .accounts(assetra::accounts::DepositRevenue {
            authority: authority.pubkey(),
            track,
            revenue_pool,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::DepositRevenue { amount })
        .instruction()
        .unwrap()
}

#[test]
fn deposit_revenue_updates_pool_accounting() {
    let (mut ctx, creator, track, revenue_pool) = setup_pool();
    let amount = 1_000_000;

    let ix = deposit_ix(&ctx, &creator, track, revenue_pool, amount);

    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();

    let pool: ::assetra::state::RevenuePool =
        ctx.get_account(&revenue_pool).unwrap();

    assert_eq!(pool.track, track);
    assert_eq!(pool.total_deposited, amount);

    let expected_increment =
        (amount as u128 * ::assetra::constants::REVENUE_PRECISION)
            / TOKEN_SUPPLY as u128;

    assert_eq!(
        pool.accumulated_rewards_per_token,
        expected_increment
    );
}

#[test]
fn deposit_revenue_rejects_zero_amount() {
    let (mut ctx, creator, track, revenue_pool) = setup_pool();

    let ix = deposit_ix(&ctx, &creator, track, revenue_pool, 0);

    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_anchor_error("InvalidDepositAmount");
}

#[test]
fn deposit_revenue_rejects_non_creator() {
    let (mut ctx, _creator, track, revenue_pool) = setup_pool();
    let unauthorized = common::funded_account(&mut ctx, INITIAL_FUNDS);

    let ix = deposit_ix(&ctx, &unauthorized, track, revenue_pool, 1_000_000);

    ctx.execute_instruction(ix, &[&unauthorized])
        .unwrap()
        .assert_anchor_error("Unauthorized");
}
