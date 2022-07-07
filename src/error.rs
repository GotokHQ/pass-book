//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the NFT pass program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum NFTPassError {
    /// Pass should be activated
    #[error("Pass should be activated")]
    PassNotActivated,

    /// Can't set the same value
    #[error("Can't set the same value")]
    CantSetTheSameValue,

    /// Invalid Authority
    #[error("InvalidCreatorKey")]
    InvalidCreatorKey,

    /// Invalid Mint Key
    #[error("InvalidMintKey")]
    InvalidMintKey,

    /// Invalid Update Authority Key
    #[error("InvalidUpdateAuthorityKey")]
    InvalidUpdateAuthorityKey,

    /// Invalid PassStore Key
    #[error("InvalidStoreKey")]
    InvalidStoreKey,

    /// Invalid Payout Key
    #[error("InvalidPayoutKey")]
    InvalidPayoutKey,

    /// Invalid Program Authority Key
    #[error("InvalidProgramAuthority")]
    InvalidProgramAuthority,

    /// Wrong pass state to change data
    #[error("Wrong pass state to change data")]
    WrongPassState,

    /// Pass is immutable
    #[error("Pass is immutable")]
    ImmutablePassBook,

    /// Overflow
    #[error("Overflow")]
    Overflow,

    /// Underflow
    #[error("Underflow")]
    Underflow,

    /// Invalid Master Pass Key
    #[error("InvalidPassBookKey")]
    InvalidPassBookKey,

    /// Invalid Pass Key
    #[error("InvalidPassKey")]
    InvalidPassKey,

    /// Invalid Pass Collection Mint Key
    #[error("InvalidPassCollectionMintKey")]
    InvalidPassCollectionMintKey,

    /// Invalid Duration
    #[error("InvalidDuration")]
    InvalidDuration,

    /// Name too long
    #[error("Name too long")]
    NameTooLong,

    /// URI too long
    #[error("URI too long")]
    UriTooLong,

    /// Description too long
    #[error("Description too long")]
    DescriptionTooLong,

    /// Invalid seller basis point
    #[error("Invalid basis point")]
    InvalidBasisPoints,

    /// Wrong max supply
    #[error("Wrong max supply")]
    WrongMaxSupply,

    /// Wrong validity period
    #[error("Wrong validity period")]
    WrongValidityPeriod,

    /// Wrong duration
    #[error("Wrong duration")]
    WrongDuration,

    /// Master edition should have unlimited supply
    #[error("Master edition should have unlimited supply")]
    WrongMasterSupply,

    /// Invalid Token Account Key
    #[error("InvalidTokenAccountKey")]
    InvalidTokenAccountKey,

    /// Pass book is already activated
    #[error("PassBookIsAlreadyActivated")]
    PassBookIsAlreadyActivated,

    /// Pass book is already deactivated
    #[error("PassBookIsAlreadyDeactivated")]
    PassBookIsAlreadyDeactivated,

    /// Price token does not match what was provided
    #[error("PriceTokenMismatch")]
    PriceTokenMismatch,

    /// User wallet must match user token account
    #[error("UserWalletMustMatchUserTokenAccount")]
    UserWalletMustMatchUserTokenAccount,

    /// Invalid Vault Key
    #[error("InvalidVaultToken")]
    InvalidVaultToken,

    /// Invalid Market Authoriy Key
    #[error("InvalidMarketAuthority")]
    InvalidMarketAuthority,

    /// Market seller basis require market authority
    #[error("MarketSellerBasisPointRequiresMarketAuthority")]
    MarketSellerBasisPointRequiresMarketAuthority,

    /// Wrong market seller basis point
    #[error("WrongMarketSellerBasisPoint")]
    WrongMarketSellerBasisPoint,

    /// Math overflow
    #[error("MathOverflow")]
    MathOverflow,

    /// Wrong Referral Share
    #[error("WrongReferralShare")]
    WrongReferralShare,

    /// Wrong Referral Kickback
    #[error("WrongReferralKickBack")]
    WrongReferralKickBack,

    /// Invalid TradeHistory Key
    #[error("InvalidTradeHistoryKey")]
    InvalidTradeHistoryKey,

    /// SupplyIsGtThanMaxSupply
    #[error("SupplyIsGtThanMaxSupply")]
    SupplyIsGtThanMaxSupply,

    /// UserReachBuyLimit
    #[error("UserReachBuyLimit")]
    UserReachBuyLimit,

    /// InvalidPayerTokenAccount
    #[error("InvalidPayerTokenAccount")]
    InvalidPayerTokenAccount,

    /// InvalidTokenAccount
    #[error("InvalidTokenAccount")]
    InvalidTokenAccount,

    /// Uninitialized
    #[error("Uninitialized")]
    Uninitialized,

    /// Invalid Membership Key
    #[error("InvalidMembershipKey")]
    InvalidMembershipKey,

    /// User has active membership
    #[error("UserHasActiveMembership")]
    UserHasActiveMembership,
}

impl From<NFTPassError> for ProgramError {
    fn from(e: NFTPassError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for NFTPassError {
    fn type_of() -> &'static str {
        "NFTPassError"
    }
}

impl PrintProgramError for NFTPassError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        msg!(&self.to_string())
    }
}
