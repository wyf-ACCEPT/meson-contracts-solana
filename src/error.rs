/// Error types in Meson.
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MesonError {
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