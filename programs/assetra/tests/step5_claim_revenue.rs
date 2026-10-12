mod common;

use litesvm_utils::Signer;

use common::{claim_ix, deposit, holder_position_pda, invest, revenue_pool_pda, setup_pool};

const HOLDING: u64 = 1_000; // 1% of the 100_000 supply
const DEPOSIT: u64 = 10_000_000;
// 1% of the deposit.
const EXPECTED_REWARD: u64 = 100_000;

#[test]
fn claim_revenue_pays_pro_rata_share() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = invest(&mut ctx, &creator, track, HOLDING);
    deposit(&mut ctx, &creator, track, DEPOSIT);

    let pool = revenue_pool_pda(&ctx.program_id, &track);
    let pool_before = common::lamports(&ctx, &pool);

    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();

    assert_eq!(pool_before - common::lamports(&ctx, &pool), EXPECTED_REWARD);

    let position: ::assetra::state::HolderPosition = ctx
        .get_account(&holder_position_pda(&ctx.program_id, &track, &investor.pubkey()))
        .unwrap();
    assert_eq!(position.total_claimed, EXPECTED_REWARD);
    assert_eq!(position.unclaimed_rewards, 0);
}

#[test]
fn claim_revenue_rejects_second_claim_without_new_deposit() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = invest(&mut ctx, &creator, track, HOLDING);
    deposit(&mut ctx, &creator, track, DEPOSIT);

    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();
    ctx.svm.expire_blockhash();

    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_anchor_error("NothingToClaim");
}

#[test]
fn claim_revenue_rejects_when_nothing_deposited() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = invest(&mut ctx, &creator, track, HOLDING);

    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_anchor_error("NothingToClaim");
}

#[test]
fn claim_revenue_picks_up_a_second_deposit() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = invest(&mut ctx, &creator, track, HOLDING);

    deposit(&mut ctx, &creator, track, DEPOSIT);
    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();
    ctx.svm.expire_blockhash();

    deposit(&mut ctx, &creator, track, DEPOSIT);
    let ix = claim_ix(&ctx, &investor, track);
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();

    let position: ::assetra::state::HolderPosition = ctx
        .get_account(&holder_position_pda(&ctx.program_id, &track, &investor.pubkey()))
        .unwrap();
    assert_eq!(position.total_claimed, 2 * EXPECTED_REWARD);
}

#[test]
fn claim_revenue_ignores_rewards_deposited_before_purchase() {
    let (mut ctx, creator, track) = setup_pool();
    deposit(&mut ctx, &creator, track, DEPOSIT);

    // Bought after the deposit: no claim on revenue that predates the purchase.
    let late_investor = invest(&mut ctx, &creator, track, HOLDING);

    let ix = claim_ix(&ctx, &late_investor, track);
    ctx.execute_instruction(ix, &[&late_investor])
        .unwrap()
        .assert_anchor_error("NothingToClaim");
}

#[test]
fn claim_revenue_rejects_wallet_that_never_held_tokens() {
    let (mut ctx, creator, track) = setup_pool();
    let _investor = invest(&mut ctx, &creator, track, HOLDING);
    deposit(&mut ctx, &creator, track, DEPOSIT);

    let outsider = common::funded_account(&mut ctx, common::INITIAL_FUNDS);
    let ix = claim_ix(&ctx, &outsider, track);
    ctx.execute_instruction(ix, &[&outsider])
        .unwrap()
        .assert_failure();
}
