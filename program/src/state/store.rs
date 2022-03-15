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

pub const MAX_PASS_STORE_LEN: usize = 
32 //store mint
+ 64 // total redeemed
+ 64 // total edition
+ 64 // total master edition
+ 118; // Padding

/// Pass Store
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct Store {
    /// Account type - Pass
    pub account_type: AccountType,
    /// Authority of this store
    pub authority: Pubkey,
    /// count of editions redeemed
    pub redemptions_count: u64,
    /// count of editions printed
    pub editions_count: u64,
    /// count of master edition passes belonging to this store
    pub pass_collection_count: u64,
}

impl Store {
    pub const PREFIX: &'static str = "store";
    /// Initialize a PackSet
    pub fn init(&mut self, authority: Pubkey) {
        self.account_type = AccountType::Store;
        self.authority = authority;
        self.redemptions_count = 0;
        self.editions_count = 0;
        self.pass_collection_count = 0;
    }

    /// Increment the total editions redeemed
    pub fn increment_redemptions_count(&mut self) -> Result<(), ProgramError> {
        self.redemptions_count = self.redemptions_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of editions printed
    pub fn increment_editions_count(&mut self) -> Result<(), ProgramError> {
        self.editions_count = self.editions_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of master edition passes
    pub fn increment_pass_collection_count(&mut self) -> Result<(), ProgramError> {
        self.pass_collection_count = self.pass_collection_count.error_increment()?;
        Ok(())
    }
}

impl IsInitialized for Store {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::Store
    }
}

impl Sealed for Store {}

impl Pack for Store {
    const LEN: usize = MAX_PASS_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::Store as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
