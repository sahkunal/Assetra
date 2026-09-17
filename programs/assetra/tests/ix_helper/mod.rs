use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::liteSVM;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

pub const TOKEN_2022_PROGRAM_ID: Pubkey= solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const PROGRAM_SO_PATH: &str = "../../target/deploy/assetra.so";

pub struct TestContext {
    pub svm: LiteSVm,
    pub payer :Keypair,
    pub program_id: Pubkey,
}

impl TestContext {
    pub fn new()-> Self{
        let mut svm= LiteSVM::new();
        let program_id = assetra::ID;
        
    }
}