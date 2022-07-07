//! Trade History definitions

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

pub const MAX_TRADE_HISTORY_LEN: usize = 1+
32 // passbook
+ 8 // already_bought
+ 32; // wallet

/// Trade History
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct TradeHistory {
    /// Account type - Pass
    pub account_type: AccountType,
    /// Passbook
    pub passbook: Pubkey,
    /// The amount of passes purchased by wallet
    pub already_bought: u64,
    /// The user wallet with this history
    pub wallet: Pubkey,
}

impl TradeHistory {
    pub const PREFIX: &'static str = "history";
    /// Initialize a PackSet
    pub fn init(&mut self, passbook: Pubkey, wallet: Pubkey) {
        self.account_type = AccountType::TradeHistory;
        self.passbook = passbook;
        self.already_bought = 0;
        self.wallet = wallet;
    }

    /// Increment the already bought
    pub fn increment_already_bought(&mut self) -> Result<(), ProgramError> {
        self.already_bought = self.already_bought.error_increment()?;
        Ok(())
    }
}

impl IsInitialized for TradeHistory {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::TradeHistory
    }
}

impl Sealed for TradeHistory {}

impl Pack for TradeHistory {
    const LEN: usize = MAX_TRADE_HISTORY_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::TradeHistory as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
