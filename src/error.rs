use solana_program::{program_error::ProgramError, decode_error::DecodeError};
/// Error types in Meson.
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MesonError {
    // 0
    #[error("Invalid instruction!")]
    InvalidInstruction,

    // 1
    #[error("Invalid ETH address!")]
    InvalidEthAddress,

    // 2
    #[error("Invalid signature!")]
    InvalidSignature,

    // 3
    #[error("Invalid encoded length!")]
    InvalidEncodedLength,

    // 4
    #[error("Invalid encoded version!")]
    InvalidEncodedVersion,

    // 5
    #[error("Target chain (swap-in chain) mismatch!")]
    InChainMismatch,

    // 6
    #[error("Source chain (swap-out chain) mismatch!")]
    OutChainMismatch,

    // 7
    #[error("Swap amount too large!")]
    SwapAmountOverMax,

    // 8
    #[error("The expected PDA account mismatches the input one!")]
    PdaAccountMismatch,

    // 9
    #[error("This PDA account is not writable!")]
    PdaAccountNotWritable,

    // 10
    #[error("This PDA account has already been created!")]
    PdaAccountAlreadyCreated,

    // 11
    #[error("Admin should sign this transaction!")]
    AdminNotSigner,

    // 12
    #[error("Coin type mismatch!")]
    CoinTypeMismatch,

    // 13
    #[error("Only premium manager can call this function!")]
    OnlyPremiumManager,

    // 14
    #[error("Only pool owner can call this function!")]
    PoolNotPoolOwner,

    // 15
    #[error("Pool index cannot be zero!")]
    PoolIndexCannotBeZero,

    // 16
    #[error("Swap not exists!")]
    SwapNotExists,

    // 17
    #[error("Swap bonded to others!")]
    SwapBondedToOthers,

    // 18
    #[error("Swap expires too early!")]
    SwapExpireTooEarly,

    // 19
    #[error("Swap expires too late!")]
    SwapExpireTooLate,

    // 20
    #[error("Pool index mismatch!")]
    PoolIndexMismatch,

    // 21
    #[error("Pool address authorized to another!")]
    PoolAddrAuthorizedToAnother,

    // 22
    #[error("Swap cannot cancel before expire!")]
    SwapCannotCancelBeforeExpire,

    // 23
    #[error("Token account information mismatch!")]
    TokenAccountMismatch,
}

impl From<MesonError> for ProgramError {
    fn from(e: MesonError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MesonError {
    fn type_of() -> &'static str {
        "MesonError"
    }
}
