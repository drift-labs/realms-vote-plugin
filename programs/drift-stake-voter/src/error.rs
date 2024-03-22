use anchor_lang::prelude::*;

use drift::error::ErrorCode as DriftErrorCode;

#[error_code]
pub enum DriftVoterError {
    #[msg("Invalid Realm Authority")]
    InvalidRealmAuthority,

    #[msg("Invalid Realm for Registrar")]
    InvalidRealmForRegistrar,

    #[msg("Invalid VoterWeightRecord Realm")]
    InvalidVoterWeightRecordRealm,

    #[msg("Invalid VoterWeightRecord Mint")]
    InvalidVoterWeightRecordMint,

    #[msg("TokenOwnerRecord from own realm is not allowed")]
    TokenOwnerRecordFromOwnRealmNotAllowed,

    #[msg("Governance program not configured")]
    GovernanceProgramNotConfigured,

    #[msg("Governing TokenOwner must match")]
    GoverningTokenOwnerMustMatch,

    #[msg("DriftError")]
    DriftError,
}

impl From<DriftErrorCode> for DriftVoterError {
    fn from(_: DriftErrorCode) -> Self {
        DriftVoterError::DriftError
    }
}
