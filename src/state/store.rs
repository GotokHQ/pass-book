//! Pass definitions

use super::*;
use crate::{math::SafeMath};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const MAX_PASS_STORE_LEN: usize = 1+
32 // authority mint
+ 8 // total redeemed
+ 8 // total edition
+ 8 // total master edition
+ 118; // Padding

/// Pass Store
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct PassStore {
    /// Account type - Pass
    pub account_type: AccountType,
    /// Authority of this store
    pub authority: Pubkey,
    /// count of editions redeemed
    pub redemptions_count: u64,
    /// count of editions printed
    pub pass_count: u64,
    /// count of master edition passes belonging to this store
    pub pass_book_count: u64,
}

impl PassStore {
    pub const PREFIX: &'static str = "store";
    /// Initialize a PackSet
    pub fn init(&mut self, authority: Pubkey) {
        self.account_type = AccountType::PassStore;
        self.authority = authority;
        self.redemptions_count = 0;
        self.pass_count = 0;
        self.pass_book_count = 0;
    }

    /// Increment the total editions redeemed
    pub fn increment_redemptions_count(&mut self) -> Result<(), ProgramError> {
        self.redemptions_count = self.redemptions_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of editions printed
    pub fn increment_pass_count(&mut self) -> Result<(), ProgramError> {
        self.pass_count = self.pass_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of master edition passes
    pub fn increment_pass_book_count(&mut self) -> Result<(), ProgramError> {
        self.pass_book_count = self.pass_book_count.error_increment()?;
        Ok(())
    }
}

impl IsInitialized for PassStore {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::PassStore
    }
}

impl Sealed for PassStore {}

impl Pack for PassStore {
    const LEN: usize = MAX_PASS_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::PassStore as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
