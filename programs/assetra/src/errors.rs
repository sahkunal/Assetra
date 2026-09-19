use anchor_lang::prelude::*;

#[error_code]
pub enum AssetraError {
    #[msg("Display name exceeds the maximum allowed length.")]
    DisplayNameTooLong,
    #[msg("Rights attestation exceeds the maximum allowed length.")]
    AttestationTooLong,
    #[msg("Title exceeds the maximum allowed length.")]
    TitleTooLong,
    #[msg("ISRC code exceeds the maximum allowed length.")]
    IsrcTooLong,
    #[msg("URI exceeds the maximum allowed length.")]
    UriTooLong,
    #[msg("Declared annual revenue must be greater than zero.")]
    InvalidDeclaredRevenue,
    #[msg("This track is not in the expected status for this instruction.")]
    InvalidTrackStatus,
    #[msg("Only the track's creator may perform this action.")]
    Unauthorized,
    #[msg("Tokenized basis points must be between 1 and 10000 (0.01%-100%).")]
    InvalidTokenizedBps,
    #[msg("Valuation multiple must be greater than zero.")]
    InvalidValuationMultiple,
    #[msg("Token supply must be greater than zero.")]
    InvalidTokenSupply,
    #[msg("Deposit amount must be greater than zero.")]
    InvalidDepositAmount,
    #[msg("Nothing to claim — no rewards have accrued since the last claim.")]
    NothingToClaim,
    #[msg("Revenue pool balance is insufficient to cover this claim while staying rent-exempt.")]
    InsufficientRevenuePoolBalance,
}
