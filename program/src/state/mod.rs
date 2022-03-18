
//! State types
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

mod pass;
mod store;
mod pass_book;

pub use pass::*;
pub use store::*;
pub use pass_book::*;

/// Global prefix for program addresses
pub const PREFIX: &str = "passbook";

pub const COLLECTION_MINT: &str = "mint";

pub const STORE: &str = "store";

pub const PASS: &str = "pass";

/// Max len of pass URI
pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_NAME_LENGTH: usize = 32;

/// Max len of pack description string
pub const MAX_DESCRIPTION_LEN: usize = 500;

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AccountType {
    /// If the account has not been initialized, the enum will be 0
    Uninitialized,
    /// Pass account
    Pass,
    /// Pass store account
    PassStore,
    /// Pass collection account
    PassBook
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Uninitialized
    }
}
