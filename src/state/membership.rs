//! Membership definitions

use super::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const MAX_MEMBERSHIP_LEN: usize = 1 //account type 
+ 32 // store
+ 32 // owner
+ 33 // pass
+ 33 // passbook
+ 9 // expires
+ 1; // memebership state

/// Pack state
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum MembershipState {
    /// Not activated
    NotActivated,
    /// Activated
    Activated,
    /// Ended
    Expired,
}

impl Default for MembershipState {
    fn default() -> Self {
        Self::NotActivated
    }
}

/// Membership
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct Membership {
    /// Account type - Membership
    pub account_type: AccountType,
    /// store
    pub store: Pubkey,
    /// owner
    pub owner: Pubkey,
    /// Membership book
    pub passbook: Option<Pubkey>,
    /// Pass
    pub pass: Option<Pubkey>,
    /// pass expiration in unix timestamp
    pub expires_at: Option<u64>,
    /// Membership state
    pub state: MembershipState,
    pub activated_at: Option<u64>,
}

impl Membership {
    pub const PREFIX: &'static str = "membership";
    /// Initialize a PackSet
    pub fn init(&mut self, store: Pubkey, owner: Pubkey) {
        self.account_type = AccountType::Membership;
        self.passbook = None;
        self.pass = None;
        self.store = store;
        self.owner = owner;
        self.expires_at = None;
        self.activated_at = None;
        self.state = MembershipState::NotActivated;
    }
}

impl IsInitialized for Membership {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::Membership
    }
}

impl Sealed for Membership {}

impl Pack for Membership {
    const LEN: usize = MAX_PASS_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::Membership as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
