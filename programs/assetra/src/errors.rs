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
}