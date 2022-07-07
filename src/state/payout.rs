//! Pass definitions

use super::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

#[derive(Debug)]
pub struct PayoutInfoArgs {
    pub authority: Pubkey,
    pub payout_account: Pubkey,
    pub token_account: Pubkey,
}

// maximum number of balances to hold
pub const MAX_NUM_BALANCES: usize = 10;

pub const MAX_BALANCE_LEN: usize = 1 + 32 + 8 + 8;

pub const MAX_PAYOUT_LEN: usize = 1+
32 // authority 
+ 32 // mint
+ 32 // treasury_holder
+ 4
+ 16;

/// Payout Account
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct Payout {
    /// Account type - Pass
    pub account_type: AccountType,
    /// Authority of this account
    pub authority: Pubkey,
    /// mint for this payout
    pub mint: Pubkey,
    /// token of this account
    pub treasury_holder: Pubkey,
    /// total balance paid in
    pub cash_in: u64,
    /// total balance paid out
    pub cash_out: u64,
}  

impl Payout {
    pub const PREFIX: &'static str = "payout";
    /// Initialize a PackSet
    pub fn init(&mut self, authority: Pubkey, mint: Pubkey, treasury_holder: Pubkey) {
        self.account_type = AccountType::Payout;
        self.authority = authority;
        self.mint = mint;
        self.treasury_holder = treasury_holder;
    }
}

impl IsInitialized for Payout {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::Payout
    }
}

impl Sealed for Payout {}

impl Pack for Payout {
    const LEN: usize = MAX_PAYOUT_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::Payout as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
