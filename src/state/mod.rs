
//! State types
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

mod pass;
mod store;
mod pass_book;
mod payout;
mod trade_history;
mod membership;
mod uses;

pub use pass::*;
pub use store::*;
pub use pass_book::*;
pub use payout::*;
pub use trade_history::*;
pub use membership::*;
pub use uses::*;


/// Global prefix for program addresses
pub const PREFIX: &str = "passbook";

pub const COLLECTION_MINT: &str = "mint";

pub const STORE: &str = "store";

pub const PASS: &str = "pass";

/// Max len of pass URI
pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_NAME_LENGTH: usize = 32;

/// Max len of pass description string
pub const MAX_DESCRIPTION_LEN: usize = 500;


pub const USES_LENGTH: usize = 24; //8 byte padding

pub const USE_AUTHORITY_LENGTH: usize = 50; //8 byte padding

pub const STORE_AUTHORITY_LENGTH: usize = 50; //8 byte padding

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    Uninitialized,
    /// Pass account
    Pass,
    /// Pass store account
    Store,
    /// Pass collection account
    PassBook,
    /// Payout
    Payout,
    /// TradeHistory
    TradeHistory,
    /// Membership
    Membership,
    /// Use authority record
    UseAuthority,
    /// Store authority record
    StoreAuthority
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Uninitialized
    }
}
