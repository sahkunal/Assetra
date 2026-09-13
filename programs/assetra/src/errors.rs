use anchor_lang::prelude::*;

#[error_code]
pub enum AssetraError {
    #[msg("Creator profile already registered for this wallet.")]
    CreatorAlreadyRegistered,

    #[msg("Investor profile already registered for this wallet.")]
    InvestorAlreadyRegistered,
}