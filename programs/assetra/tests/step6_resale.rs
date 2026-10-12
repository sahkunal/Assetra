mod common;

use litesvm_utils::Signer;

use common::{
    ata, buy_resale_ix, cancel_ix, claim_ix, deposit, holder_position_pda, invest, list_ix,
    listing_pda, mint_pda, setup_pool, token_2022_program_id, token_balance,
};

const HOLDING: u64 = 1_000;
const LIST_AMOUNT: u64 = 400;
const PRICE: u64 = 50;
const DEPOSIT: u64 = 10_000_000;

#[test]
fn list_resale_records_listing_and_delegates_tokens() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    let listing_key = listing_pda(&ctx.program_id, &track, &seller.pubkey(), 0);
    let listing: ::assetra::state::Listing = ctx.get_account(&listing_key).unwrap();
    assert_eq!(listing.seller, seller.pubkey());
    assert_eq!(listing.track, track);
    assert_eq!(listing.amount, LIST_AMOUNT);
    assert_eq!(listing.price_per_token, PRICE);

    let mint = mint_pda(&ctx.program_id, &track);
    let seller_ata = ata(&seller.pubkey(), &mint, &token_2022_program_id());
    assert_eq!(common::delegated_amount(&ctx, &seller_ata), LIST_AMOUNT);
    // Tokens stay in the seller's account until sold.
    assert_eq!(token_balance(&ctx, &seller_ata), HOLDING);
}

#[test]
fn list_resale_rejects_zero_amount() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, 0, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_anchor_error("InvalidListingAmount");
}

#[test]
fn list_resale_rejects_zero_price() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, 0, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_anchor_error("InvalidListingPrice");
}

#[test]
fn list_resale_rejects_amount_above_balance() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, HOLDING + 1, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_anchor_error("InsufficientListedBalance");
}

#[test]
fn buy_resale_moves_tokens_and_pays_seller() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);
    let buyer = common::funded_account(&mut ctx, common::INITIAL_FUNDS);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    let seller_before = common::lamports(&ctx, &seller.pubkey());

    let ix = buy_resale_ix(&ctx, &buyer, &seller.pubkey(), track, 0);
    ctx.execute_instruction(ix, &[&buyer])
        .unwrap()
        .assert_success();

    let mint = mint_pda(&ctx.program_id, &track);
    let tp = token_2022_program_id();
    assert_eq!(
        token_balance(&ctx, &ata(&buyer.pubkey(), &mint, &tp)),
        LIST_AMOUNT
    );
    assert_eq!(
        token_balance(&ctx, &ata(&seller.pubkey(), &mint, &tp)),
        HOLDING - LIST_AMOUNT
    );

    // Seller receives the sale price plus the closed listing's rent.
    let listing_key = listing_pda(&ctx.program_id, &track, &seller.pubkey(), 0);
    assert!(ctx.svm.get_account(&listing_key).map_or(true, |a| a.lamports == 0));
    assert!(common::lamports(&ctx, &seller.pubkey()) >= seller_before + LIST_AMOUNT * PRICE);

    let position: ::assetra::state::HolderPosition = ctx
        .get_account(&holder_position_pda(&ctx.program_id, &track, &buyer.pubkey()))
        .unwrap();
    assert_eq!(position.holder, buyer.pubkey());
}

#[test]
fn resale_splits_future_revenue_by_new_balances_and_keeps_past_revenue_with_seller() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);
    let buyer = common::funded_account(&mut ctx, common::INITIAL_FUNDS);

    // Revenue earned while the seller still holds everything.
    deposit(&mut ctx, &creator, track, DEPOSIT);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();
    let ix = buy_resale_ix(&ctx, &buyer, &seller.pubkey(), track, 0);
    ctx.execute_instruction(ix, &[&buyer])
        .unwrap()
        .assert_success();

    // Revenue earned after the trade.
    deposit(&mut ctx, &creator, track, DEPOSIT);

    // Seller: 1% of first deposit (100_000) + 0.6% of second (60_000).
    let ix = claim_ix(&ctx, &seller, track);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();
    let seller_pos: ::assetra::state::HolderPosition = ctx
        .get_account(&holder_position_pda(&ctx.program_id, &track, &seller.pubkey()))
        .unwrap();
    assert_eq!(seller_pos.total_claimed, 160_000);

    // Buyer: nothing from the first deposit, 0.4% of the second (40_000).
    let ix = claim_ix(&ctx, &buyer, track);
    ctx.execute_instruction(ix, &[&buyer])
        .unwrap()
        .assert_success();
    let buyer_pos: ::assetra::state::HolderPosition = ctx
        .get_account(&holder_position_pda(&ctx.program_id, &track, &buyer.pubkey()))
        .unwrap();
    assert_eq!(buyer_pos.total_claimed, 40_000);
}

#[test]
fn cancel_resale_closes_listing_and_revokes_delegate() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    let ix = cancel_ix(&ctx, &seller, track, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    let listing_key = listing_pda(&ctx.program_id, &track, &seller.pubkey(), 0);
    assert!(ctx.svm.get_account(&listing_key).map_or(true, |a| a.lamports == 0));

    let mint = mint_pda(&ctx.program_id, &track);
    let seller_ata = ata(&seller.pubkey(), &mint, &token_2022_program_id());
    assert_eq!(common::delegated_amount(&ctx, &seller_ata), 0);
}

#[test]
fn cancel_resale_rejects_non_seller() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);
    let attacker = invest(&mut ctx, &creator, track, HOLDING);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    // Attacker points the instruction at the seller's listing.
    let mut ix = cancel_ix(&ctx, &attacker, track, 0);
    let listing_key = listing_pda(&ctx.program_id, &track, &seller.pubkey(), 0);
    ix.accounts[3].pubkey = listing_key; // `listing` slot
    ctx.execute_instruction(ix, &[&attacker])
        .unwrap()
        .assert_failure();
}

#[test]
fn buy_resale_fails_after_cancel() {
    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);
    let buyer = common::funded_account(&mut ctx, common::INITIAL_FUNDS);

    let ix = list_ix(&ctx, &seller, track, LIST_AMOUNT, PRICE, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();
    let ix = cancel_ix(&ctx, &seller, track, 0);
    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_success();

    let ix = buy_resale_ix(&ctx, &buyer, &seller.pubkey(), track, 0);
    ctx.execute_instruction(ix, &[&buyer])
        .unwrap()
        .assert_failure();
}

/// A seller must not be able to list tokens of an unrelated mint under a
/// real track: sale accounting reads balances of whatever mint is passed in.
#[test]
fn list_resale_rejects_mint_that_is_not_the_tracks_mint() {
    use litesvm_utils::TestHelpers;

    let (mut ctx, creator, track) = setup_pool();
    let seller = invest(&mut ctx, &creator, track, HOLDING);

    let fake_mint = ctx.svm.create_token_mint(&seller, 0).unwrap();
    let fake_ata = ctx
        .svm
        .create_associated_token_account(&fake_mint.pubkey(), &seller)
        .unwrap();
    ctx.svm
        .mint_to(&fake_mint.pubkey(), &fake_ata, &seller, 1_000_000)
        .unwrap();

    let ix = ctx
        .program()
        .accounts(assetra::accounts::ListResale {
            seller: seller.pubkey(),
            track,
            seller_token_account: fake_ata,
            mint: fake_mint.pubkey(),
            listing: listing_pda(&ctx.program_id, &track, &seller.pubkey(), 0),
            token_program: spl_token::ID,
            system_program: anchor_lang::system_program::ID,
        })
        .args(assetra::instruction::ListResale {
            amount: 1,
            price_per_token: PRICE,
            nonce: 0,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&seller])
        .unwrap()
        .assert_anchor_error("InvalidMint");
}
