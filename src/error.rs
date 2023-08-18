use solana_program::{program_error::ProgramError, decode_error::DecodeError};
/// Error types in Meson.
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MesonError {
    #[error("Invalid instruction!")]
    InvalidInstruction,

    #[error("Invalid ETH address!")]
    InvalidEthAddress,

    // #[error("Invalid public key!")]
    // InvalidPublicKey,

    #[error("Invalid signature!")]
    InvalidSignature,

    #[error("Invalid encoded length!")]
    InvalidEncodedLength,

    #[error("Invalid encoded version!")]
    InvalidEncodedVersion,

    #[error("Target chain (swap-in chain) mismatch!")]
    InChainMismatch,

    #[error("Source chain (swap-out chain) mismatch!")]
    OutChainMismatch,

    #[error("Swap amount too large!")]
    SwapAmountOverMax,

}

impl From<MesonError> for ProgramError {
    fn from(e: MesonError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for MesonError {
    fn type_of() -> &'static str {
        "TokenError"
    }
}
