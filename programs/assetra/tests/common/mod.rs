// anchor_lang::declare_program!(assetra);

use anchor_litesvm::{AnchorContext, AnchorLiteSVM};
use litesvm_utils::{Keypair, Pubkey, Signer, TestHelpers};


pub const PROGRAM_SO_PATH: &str = "../../target/deploy/assetra.so";

/// Loads the built program at the ID it declares (`declare_id!`), so tests do
/// not depend on whatever keypair happens to sit in target/deploy.
pub fn new_ctx() -> AnchorContext {
    let program_bytes = std::fs::read(PROGRAM_SO_PATH)
        .expect("failed to read assetra.so — did you run `anchor build`?");

    AnchorLiteSVM::build_with_program(assetra::ID, &program_bytes)
}

/// Creates a funded keypair to act as a creator, investor, etc. in a test.
pub fn funded_account(ctx: &mut AnchorContext, lamports: u64) -> Keypair {
    ctx.svm
        .create_funded_account(lamports)
        .expect("failed to create funded test account")
}

pub fn token_2022_program_id() -> Pubkey {
    anchor_spl::token_2022::ID
}

pub fn creator_profile_pda(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"creator_profile", authority.as_ref()], program_id).0
}

pub fn track_pda(program_id: &Pubkey, authority: &Pubkey, track_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"track", authority.as_ref(), &track_id.to_le_bytes()],
        program_id,
    )
    .0
}

pub fn mint_pda(program_id: &Pubkey, track: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"mint", track.as_ref()], program_id).0
}

pub fn vault_pda(program_id: &Pubkey, track: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", track.as_ref()], program_id).0
}

pub fn revenue_pool_pda(program_id: &Pubkey, track: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"revenue_pool", track.as_ref()], program_id).0
}

pub fn holder_position_pda(program_id: &Pubkey, track: &Pubkey, holder: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"holder_position", track.as_ref(), holder.as_ref()],
        program_id,
    )
    .0
}

pub fn config_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"config"], program_id).0
}

pub fn listing_pda(
    program_id: &Pubkey,
    track: &Pubkey,
    seller: &Pubkey,
    nonce: u64,
) -> Pubkey {
    Pubkey::find_program_address(
        &[b"listing", track.as_ref(), seller.as_ref(), &nonce.to_le_bytes()],
        program_id,
    )
    .0
}

pub fn init_config(ctx: &mut AnchorContext, admin: &Keypair, verifier: Pubkey) {
    let ix = ctx
        .program()
        .accounts(assetra::accounts::InitializeConfig {
            admin: admin.pubkey(),
            config: config_pda(&ctx.program_id),
            system_program: anchor_lang::system_program::ID,
        })
        .args(assetra::instruction::InitializeConfig { verifier })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[admin])
        .unwrap()
        .assert_success();
}

pub fn verify_track_ix(
    ctx: &AnchorContext,
    verifier: &Keypair,
    track: Pubkey,
    verified_annual_revenue: u64,
) -> anchor_lang::solana_program::instruction::Instruction {
    ctx.program()
        .accounts(assetra::accounts::VerifyTrack {
            verifier: verifier.pubkey(),
            config: config_pda(&ctx.program_id),
            track,
        })
        .args(assetra::instruction::VerifyTrack {
            verified_annual_revenue,
        })
        .instruction()
        .unwrap()
}

/// Test shortcut: stand up a config whose admin and verifier are one fresh
/// key, then verify `track` with the given revenue.
pub fn verify_track(ctx: &mut AnchorContext, track: Pubkey, verified_annual_revenue: u64) {
    let verifier = funded_account(ctx, 1_000_000_000);
    init_config(ctx, &verifier, verifier.pubkey());
    let ix = verify_track_ix(ctx, &verifier, track, verified_annual_revenue);
    ctx.execute_instruction(ix, &[&verifier])
        .unwrap()
        .assert_success();
}

// ---------------------------------------------------------------------------
// Full-flow helpers shared by the claim / resale suites
// ---------------------------------------------------------------------------

use anchor_lang::system_program;
use anchor_lang::solana_program::instruction::Instruction;

pub const INITIAL_FUNDS: u64 = 10_000_000_000;
pub const TOKEN_SUPPLY: u64 = 100_000;

pub fn associated_token_program_id() -> Pubkey {
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        .parse()
        .unwrap()
}

pub fn ata(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &associated_token_program_id(),
    )
    .0
}

/// Balance of a Token-2022 account (0 if it does not exist).
pub fn token_balance(ctx: &AnchorContext, address: &Pubkey) -> u64 {
    use anchor_spl::token_2022::spl_token_2022::extension::StateWithExtensions;
    use anchor_spl::token_2022::spl_token_2022::state::Account;

    match ctx.svm.get_account(address) {
        Some(acc) if !acc.data.is_empty() => {
            StateWithExtensions::<Account>::unpack(&acc.data)
                .expect("should decode Token-2022 account")
                .base
                .amount
        }
        _ => 0,
    }
}

/// Delegated amount of a Token-2022 account.
pub fn delegated_amount(ctx: &AnchorContext, address: &Pubkey) -> u64 {
    use anchor_spl::token_2022::spl_token_2022::extension::StateWithExtensions;
    use anchor_spl::token_2022::spl_token_2022::state::Account;

    let acc = ctx.svm.get_account(address).expect("token account exists");
    StateWithExtensions::<Account>::unpack(&acc.data)
        .unwrap()
        .base
        .delegated_amount
}

pub fn lamports(ctx: &AnchorContext, address: &Pubkey) -> u64 {
    ctx.svm.get_account(address).map(|a| a.lamports).unwrap_or(0)
}

/// Registers a creator, creates + verifies + mints a track, and initializes
/// its revenue pool. Returns (ctx, creator, track).
pub fn setup_pool() -> (AnchorContext, Keypair, Pubkey) {
    let mut ctx = new_ctx();
    let creator = funded_account(&mut ctx, INITIAL_FUNDS);
    let profile = creator_profile_pda(&ctx.program_id, &creator.pubkey());
    let track = track_pda(&ctx.program_id, &creator.pubkey(), 0);

    let ixs: Vec<Instruction> = vec![
        ctx.program()
            .accounts(assetra::accounts::RegisterCreator {
                authority: creator.pubkey(),
                creator_profile: profile,
                system_program: system_program::ID,
            })
            .args(assetra::instruction::RegisterCreator {
                display_name: "Test Artist".to_string(),
                rights_attestation: "I own the rights.".to_string(),
            })
            .instruction()
            .unwrap(),
        ctx.program()
            .accounts(assetra::accounts::CreateTrack {
                authority: creator.pubkey(),
                creator_profile: profile,
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
            .unwrap(),
    ];
    for ix in ixs {
        ctx.execute_instruction(ix, &[&creator])
            .unwrap()
            .assert_success();
    }

    verify_track(&mut ctx, track, 500_000);

    let ix = ctx
        .program()
        .accounts(assetra::accounts::MintTrackTokens {
            authority: creator.pubkey(),
            track,
            mint: mint_pda(&ctx.program_id, &track),
            vault: vault_pda(&ctx.program_id, &track),
            token_program: token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::MintTrackTokens {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();

    let ix = ctx
        .program()
        .accounts(assetra::accounts::InitializeRevenuePool {
            authority: creator.pubkey(),
            track,
            revenue_pool: revenue_pool_pda(&ctx.program_id, &track),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::InitializeRevenuePool {})
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&creator])
        .unwrap()
        .assert_success();

    (ctx, creator, track)
}

/// Funds a new investor and buys `amount` tokens from the primary vault.
pub fn invest(
    ctx: &mut AnchorContext,
    creator: &Keypair,
    track: Pubkey,
    amount: u64,
) -> Keypair {
    let investor = funded_account(ctx, INITIAL_FUNDS);
    let mint = mint_pda(&ctx.program_id, &track);
    let tp = token_2022_program_id();

    let ix = ctx
        .program()
        .accounts(assetra::accounts::BuyTokens {
            investor: investor.pubkey(),
            creator: creator.pubkey(),
            track,
            vault: vault_pda(&ctx.program_id, &track),
            investor_token_account: ata(&investor.pubkey(), &mint, &tp),
            mint,
            revenue_pool: revenue_pool_pda(&ctx.program_id, &track),
            holder_position: holder_position_pda(&ctx.program_id, &track, &investor.pubkey()),
            token_program: tp,
            associated_token_program: associated_token_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::BuyTokens { amount })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[&investor])
        .unwrap()
        .assert_success();
    investor
}

pub fn deposit(ctx: &mut AnchorContext, creator: &Keypair, track: Pubkey, amount: u64) {
    let ix = ctx
        .program()
        .accounts(assetra::accounts::DepositRevenue {
            authority: creator.pubkey(),
            track,
            revenue_pool: revenue_pool_pda(&ctx.program_id, &track),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::DepositRevenue { amount })
        .instruction()
        .unwrap();
    ctx.execute_instruction(ix, &[creator])
        .unwrap()
        .assert_success();
    // Identical consecutive transactions would otherwise be deduplicated.
    ctx.svm.expire_blockhash();
}

pub fn claim_ix(ctx: &AnchorContext, holder: &Keypair, track: Pubkey) -> Instruction {
    let mint = mint_pda(&ctx.program_id, &track);
    ctx.program()
        .accounts(assetra::accounts::ClaimRevenue {
            holder: holder.pubkey(),
            track,
            holder_token_account: ata(&holder.pubkey(), &mint, &token_2022_program_id()),
            revenue_pool: revenue_pool_pda(&ctx.program_id, &track),
            holder_position: holder_position_pda(&ctx.program_id, &track, &holder.pubkey()),
        })
        .args(assetra::instruction::ClaimRevenue {})
        .instruction()
        .unwrap()
}

pub fn list_ix(
    ctx: &AnchorContext,
    seller: &Keypair,
    track: Pubkey,
    amount: u64,
    price_per_token: u64,
    nonce: u64,
) -> Instruction {
    let mint = mint_pda(&ctx.program_id, &track);
    ctx.program()
        .accounts(assetra::accounts::ListResale {
            seller: seller.pubkey(),
            track,
            seller_token_account: ata(&seller.pubkey(), &mint, &token_2022_program_id()),
            mint,
            listing: listing_pda(&ctx.program_id, &track, &seller.pubkey(), nonce),
            token_program: token_2022_program_id(),
            system_program: system_program::ID,
        })
        .args(assetra::instruction::ListResale {
            amount,
            price_per_token,
            nonce,
        })
        .instruction()
        .unwrap()
}

pub fn buy_resale_ix(
    ctx: &AnchorContext,
    buyer: &Keypair,
    seller: &Pubkey,
    track: Pubkey,
    nonce: u64,
) -> Instruction {
    let mint = mint_pda(&ctx.program_id, &track);
    let tp = token_2022_program_id();
    ctx.program()
        .accounts(assetra::accounts::BuyResale {
            buyer: buyer.pubkey(),
            seller: *seller,
            track,
            seller_token_account: ata(seller, &mint, &tp),
            buyer_token_account: ata(&buyer.pubkey(), &mint, &tp),
            mint,
            revenue_pool: revenue_pool_pda(&ctx.program_id, &track),
            seller_holder_position: holder_position_pda(&ctx.program_id, &track, seller),
            buyer_holder_position: holder_position_pda(&ctx.program_id, &track, &buyer.pubkey()),
            listing: listing_pda(&ctx.program_id, &track, seller, nonce),
            system_program: system_program::ID,
            token_program: tp,
            associated_token_program: associated_token_program_id(),
        })
        .args(assetra::instruction::BuyResale {})
        .instruction()
        .unwrap()
}

pub fn cancel_ix(ctx: &AnchorContext, seller: &Keypair, track: Pubkey, nonce: u64) -> Instruction {
    let mint = mint_pda(&ctx.program_id, &track);
    ctx.program()
        .accounts(assetra::accounts::CancelResale {
            seller: seller.pubkey(),
            track,
            seller_token_account: ata(&seller.pubkey(), &mint, &token_2022_program_id()),
            listing: listing_pda(&ctx.program_id, &track, &seller.pubkey(), nonce),
            token_program: token_2022_program_id(),
        })
        .args(assetra::instruction::CancelResale {})
        .instruction()
        .unwrap()
}
