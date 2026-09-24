// anchor_lang::declare_program!(assetra);

use anchor_litesvm::{AnchorContext, AnchorLiteSVM};
use litesvm_utils::{Keypair, Pubkey, Signer, TestHelpers};

pub const PROGRAM_SO_PATH: &str = "../../target/deploy/assetra.so";
pub const PROGRAM_KEYPAIR_PATH: &str = "../../target/deploy/assetra-keypair.json";

pub fn new_ctx() -> AnchorContext {
    let keypair_json = std::fs::read_to_string(PROGRAM_KEYPAIR_PATH)
        .expect("failed to read assetra-keypair.json — did you run `anchor build`?");
    let keypair_bytes: Vec<u8> = serde_json::from_str(&keypair_json)
        .expect("assetra-keypair.json was not valid keypair JSON");
    let program_keypair = Keypair::try_from(keypair_bytes.as_slice())
        .expect("invalid keypair bytes in assetra-keypair.json");

    let program_bytes = std::fs::read(PROGRAM_SO_PATH)
        .expect("failed to read assetra.so — did you run `anchor build`?");

    AnchorLiteSVM::build_with_program(program_keypair.pubkey(), &program_bytes)
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

/*pub fn mint_pda(program_id: &Pubkey, track: &Pubkey) -> Pubkey {
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
}*/
