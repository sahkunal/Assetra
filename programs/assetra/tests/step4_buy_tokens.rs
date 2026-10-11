
mod common;
use common::token_2022_program_id;
use anchor_lang::system_program;
use anchor_litesvm::AnchorContext;
use litesvm_utils::{Keypair, Pubkey, Signer};
use std::str::FromStr;


use common::{
    creator_profile_pda, holder_position_pda, mint_pda,
    revenue_pool_pda, track_pda, vault_pda,
};

const INITIAL_FUNDS: u64 = 10_000_000_000;
const TOKEN_SUPPLY: u64 = 100_000;
const PURCHASE_AMOUNT: u64 = 1_000;




fn associated_token_program_id() -> Pubkey {
    Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")
        .expect("invalid associated token program ID")
}

fn associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            owner.as_ref(),
            token_program.as_ref(),
            mint.as_ref(),
        ],
        &associated_token_program_id(),
    )
    .0
}



fn setup_pool() -> (AnchorContext, Keypair, Pubkey) {
    let mut ctx = common::new_ctx();
    let creator = common::funded_account(&mut ctx, INITIAL_FUNDS);
    let creator_profile = creator_profile_pda(&ctx.program_id, &creator.pubkey());

    let ix = ctx.program()
        .accounts(assetra::accounts::RegisterCreator {
            authority: creator.pubkey(),
            creator_profile,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::RegisterCreator {
            display_name: "Test Artist".to_string(),
            rights_attestation: "I own the rights.".to_string(),
        })
        .instruction().unwrap();

    ctx.execute_instruction(ix, &[&creator]).unwrap().assert_success();

    let track = track_pda(&ctx.program_id, &creator.pubkey(), 0);

    let ix = ctx.program()
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
        .instruction().unwrap();

    ctx.execute_instruction(ix, &[&creator]).unwrap().assert_success();

    let mint = mint_pda(&ctx.program_id, &track);
    let vault = vault_pda(&ctx.program_id, &track);

    let ix = ctx.program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: creator.pubkey(),
            track,
            mint,
            vault,
            token_program: common::token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction().unwrap();

    ctx.execute_instruction(ix, &[&creator]).unwrap().assert_success();

    let revenue_pool = revenue_pool_pda(&ctx.program_id, &track);

    let ix = ctx.program()
        .accounts(assetra::accounts::InitializeRevenuePool {
            authority: creator.pubkey(),
            track,
            revenue_pool,
            system_program: system_program::ID,
        })
        .args(assetra::instruction::InitializeRevenuePool {})
        .instruction().unwrap();

    ctx.execute_instruction(ix, &[&creator]).unwrap().assert_success();

    (ctx, creator, track)
}

fn buy_ix(
    ctx: &AnchorContext,
    creator: &Keypair,
    investor: &Keypair,
    track: Pubkey,
    amount: u64,
) -> anchor_lang::solana_program::instruction::Instruction {
    let mint = mint_pda(&ctx.program_id, &track);
    let vault = vault_pda(&ctx.program_id, &track);
    let revenue_pool = revenue_pool_pda(&ctx.program_id, &track);
    let holder_position =
        holder_position_pda(&ctx.program_id, &track, &investor.pubkey());

    let token_program = token_2022_program_id();

    let investor_token_account = associated_token_address(
        &investor.pubkey(),
        &mint,
        &token_program,
    );

    ctx.program()
        .accounts(assetra::accounts::BuyTokens {
            investor: investor.pubkey(),
            creator: creator.pubkey(),
            track,
            vault,
            investor_token_account,
            mint,
            revenue_pool,
            holder_position,
            token_program,
            associated_token_program: associated_token_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::BuyTokens { amount })
        .instruction()
        .unwrap()
}

fn expected_price(amount: u64) -> u64 {
    ((500_000u128 * 500 * 2_000 * amount as u128)
        / (100u128 * 10_000 * TOKEN_SUPPLY as u128)) as u64
}


fn token_balance(ctx: &AnchorContext, address: &Pubkey) -> u64 {
    use anchor_spl::token_2022::spl_token_2022::extension::StateWithExtensions;
    use anchor_spl::token_2022::spl_token_2022::state::Account;

    let account = ctx
        .svm
        .get_account(address)
        .expect("token account should exist");

    let token_account =
        StateWithExtensions::<Account>::unpack(&account.data)
            .expect("should decode Token-2022 account");

    token_account.base.amount
}




#[test]
fn buy_tokens_transfers_tokens_and_pays_creator() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = common::funded_account(&mut ctx, INITIAL_FUNDS);

   let mint = mint_pda(&ctx.program_id, &track);
let token_program = token_2022_program_id();

let investor_ata = associated_token_address(
    &investor.pubkey(),
    &mint,
    &token_program,
);

    let creator_before = ctx.svm.get_account(&creator.pubkey()).unwrap().lamports;
    let vault_before = token_balance(&ctx, &vault_pda(&ctx.program_id, &track));

    let ix = buy_ix(&ctx, &creator, &investor, track, PURCHASE_AMOUNT);

    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();

    let creator_after = ctx.svm.get_account(&creator.pubkey()).unwrap().lamports;
    let vault_after = token_balance(&ctx, &vault_pda(&ctx.program_id, &track));

    assert_eq!(
        creator_after - creator_before,
        expected_price(PURCHASE_AMOUNT)
    );
    assert_eq!(token_balance(&ctx, &investor_ata), PURCHASE_AMOUNT);
    assert_eq!(vault_before - vault_after, PURCHASE_AMOUNT);

    let position: ::assetra::state::HolderPosition =
        ctx.get_account(&holder_position_pda(
            &ctx.program_id,
            &track,
            &investor.pubkey(),
        )).unwrap();

    assert_eq!(position.track, track);
    assert_eq!(position.holder, investor.pubkey());
    assert_eq!(position.unclaimed_rewards, 0);
    assert_eq!(position.total_claimed, 0);
}

#[test]
fn buy_tokens_rejects_amount_above_vault_supply() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = common::funded_account(&mut ctx, INITIAL_FUNDS);

    let ix = buy_ix(&ctx, &creator, &investor, track, TOKEN_SUPPLY + 1);

    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_anchor_error("InsufficientVaultSupply");
}

#[test]
fn buy_tokens_rejects_zero_amount() {
    let (mut ctx, creator, track) = setup_pool();
    let investor = common::funded_account(&mut ctx, INITIAL_FUNDS);

    let ix = buy_ix(&ctx, &creator, &investor, track, 0);

    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_anchor_error("InvalidPurchaseAmount");
}
