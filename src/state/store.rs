//! Pass definitions

use super::*;
use crate::math::SafeMath;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const MAX_STORE_LEN: usize = 1+
32 // authority mint
+ 8 // total redeemed
+ 8 // membership count
+ 8 // passes count
+ 8 // active membership
+ 8 // total pass books
+ 33 // store referrer
+ 9; // referral end

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
    /// count of memberships created
    pub membership_count: u64,
    /// count of active membership
    pub active_membership_count: u64,
    /// count of passes created
    pub pass_count: u64,
    /// count of master edition passes belonging to this store
    pub pass_book_count: u64,
    /// referrer wallet
    pub referrer: Option<Pubkey>,
    /// Date referral rewards end
    pub referral_end_date: Option<u64>,
}

impl Store {
    pub const PREFIX: &'static str = "store";
    /// Initialize a PackSet
    pub fn init(&mut self, authority: Pubkey) {
        self.account_type = AccountType::Store;
        self.authority = authority;
        self.redemptions_count = 0;
        self.membership_count = 0;
        self.active_membership_count = 0;
        self.pass_count = 0;
        self.pass_book_count = 0;
    }

    /// Increment the total editions redeemed
    pub fn increment_redemptions_count(&mut self) -> Result<(), ProgramError> {
        self.redemptions_count = self.redemptions_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of editions printed
    pub fn increment_membership_count(&mut self) -> Result<(), ProgramError> {
        self.membership_count = self.membership_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of passes issued
    pub fn increment_pass_count(&mut self) -> Result<(), ProgramError> {
        self.pass_count = self.pass_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of active membership issued
    pub fn increment_active_membership_count(&mut self) -> Result<(), ProgramError> {
        self.active_membership_count = self.active_membership_count.error_increment()?;
        Ok(())
    }

    /// Increment the total number of master edition passes
    pub fn increment_pass_book_count(&mut self) -> Result<(), ProgramError> {
        self.pass_book_count = self.pass_book_count.error_increment()?;
        Ok(())
    }
}

impl IsInitialized for Store {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized && self.account_type == AccountType::Store
    }
}

impl Sealed for Store {}

impl Pack for Store {
    const LEN: usize = MAX_STORE_LEN;

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

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, BorshSchema)]
pub struct StoreAuthority {
    pub account_type: AccountType, //1
    pub store: Pubkey,             // 32
    pub allowed_redemptions: u64,  //8
    pub bump: u8,                  //1
}

impl StoreAuthority {
    pub const PREFIX: &'static str = "admin";
    /// Initialize a store authority
    pub fn init(&mut self, store: Pubkey, bump: u8) {
        self.account_type = AccountType::StoreAuthority;
        self.store = store;
        self.bump = bump;
    }
}

impl IsInitialized for StoreAuthority {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::StoreAuthority
    }
}

impl Sealed for StoreAuthority {}

impl Pack for StoreAuthority {
    const LEN: usize = STORE_AUTHORITY_LENGTH;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::StoreAuthority as u8
            && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
