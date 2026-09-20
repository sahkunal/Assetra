
use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub const TOKEN_2022_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");


pub const PROGRAM_SO_PATH: &str = "../../target/deploy/assetra.so";

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub program_id: Pubkey,
}

impl TestContext {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        let program_id = assetra::ID;

        svm.add_program_from_file(program_id, PROGRAM_SO_PATH)
            .expect("failed to load assetra.so — did you run `anchor build` first?");

        litesvm_token::add_token_program(&mut svm);

        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10_000_000_000) // 10 SOL
            .expect("airdrop failed");

        Self {
            svm,
            payer,
            program_id,
        }
    }


    pub fn send_ix(&mut self, ix: Instruction, extra_signers: &[&Keypair]) {
        let mut signers: Vec<&Keypair> = vec![&self.payer];
        signers.extend_from_slice(extra_signers);

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.payer.pubkey()),
            &signers,
            self.svm.latest_blockhash(),
        );

        let result = self.svm.send_transaction(tx);
        assert!(result.is_ok(), "transaction failed: {:?}", result.err());
    }

    pub fn send_ix_expect_err(&mut self, ix: Instruction, extra_signers: &[&Keypair]) -> String {
        let mut signers: Vec<&Keypair> = vec![&self.payer];
        signers.extend_from_slice(extra_signers);

        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.payer.pubkey()),
            &signers,
            self.svm.latest_blockhash(),
        );

        let result = self.svm.send_transaction(tx);
        format!("{:?}", result.err().expect("expected transaction to fail, but it succeeded"))
    }
}

pub fn register_creator_ix(
    program_id: Pubkey,
    authority: Pubkey,
    creator_profile: Pubkey,
    display_name: String,
    rights_attestation: String,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::RegisterCreator {
            authority,
            creator_profile,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::RegisterCreator {
            display_name,
            rights_attestation,
        }
        .data(),
    }
}


pub fn create_track_ix(
    program_id: Pubkey,
    authority: Pubkey,
    creator_profile: Pubkey,
    track: Pubkey,
    title: String,
    isrc: String,
    cover_art_uri: String,
    audio_preview_uri: String,
    declared_annual_revenue: u64,
    tokenized_bps: u16,
    valuation_multiple_bps: u32,
    token_supply: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::CreateTrack {
            authority,
            creator_profile,
            track,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::CreateTrack {
            title,
            isrc,
            cover_art_uri,
            audio_preview_uri,
            declared_annual_revenue,
            tokenized_bps,
            valuation_multiple_bps,
            token_supply,
        }
        .data(),
    }
}


pub fn mint_track_tokens_ix(
    program_id: Pubkey,
    authority: Pubkey,
    track: Pubkey,
    mint: Pubkey,
    vault: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::MintTrackTokens {
            authority,
            track,
            mint,
            vault,
            token_program: TOKEN_2022_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::MintTrackTokens {}.data(),
    }
}

/// Builds an `initialize_revenue_pool` instruction.
pub fn initialize_revenue_pool_ix(
    program_id: Pubkey,
    authority: Pubkey,
    track: Pubkey,
    revenue_pool: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::InitializeRevenuePool {
            authority,
            track,
            revenue_pool,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::InitializeRevenuePool {}.data(),
    }
}

/// Builds a `deposit_revenue` instruction.
pub fn deposit_revenue_ix(
    program_id: Pubkey,
    authority: Pubkey,
    track: Pubkey,
    revenue_pool: Pubkey,
    amount: u64,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::DepositRevenue {
            authority,
            track,
            revenue_pool,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::DepositRevenue { amount }.data(),
    }
}

/// Builds an `open_holder_position` instruction.
pub fn open_holder_position_ix(
    program_id: Pubkey,
    holder: Pubkey,
    track: Pubkey,
    holder_token_account: Pubkey,
    revenue_pool: Pubkey,
    holder_position: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::OpenHolderPosition {
            holder,
            track,
            holder_token_account,
            revenue_pool,
            holder_position,
            mint,
            token_program: TOKEN_2022_PROGRAM_ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: assetra::instruction::OpenHolderPosition {}.data(),
    }
}

/// Builds a `claim_revenue` instruction.
pub fn claim_revenue_ix(
    program_id: Pubkey,
    holder: Pubkey,
    track: Pubkey,
    holder_token_account: Pubkey,
    revenue_pool: Pubkey,
    holder_position: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: assetra::accounts::ClaimRevenue {
            holder,
            track,
            holder_token_account,
            revenue_pool,
            holder_position,
        }
        .to_account_metas(None),
        data: assetra::instruction::ClaimRevenue {}.data(),
    }
}


pub fn seed_token_account(
    svm: &mut LiteSVM,
    pubkey: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) {
    let mut data = [0u8; 165]; // base SPL/Token-2022 account layout, no extensions
    data[0..32].copy_from_slice(mint.as_ref());
    data[32..64].copy_from_slice(owner.as_ref());
    data[64..72].copy_from_slice(&amount.to_le_bytes());
    data[108] = 1; 
   
    let account = litesvm::solana_account::Account {
        lamports: 2_000_000, 
        data: data.to_vec(),
        owner: TOKEN_2022_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(pubkey, account)
        .expect("failed to seed test token account");
}
