//! Pass definitions

use super::*;
use crate::{
    error::NFTPassError,
    state::{MAX_DESCRIPTION_LEN, MAX_URI_LENGTH, MAX_NAME_LENGTH},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const MAX_PASS_LEN: usize = 
1 //account type 
+4
+ MAX_NAME_LENGTH // name
+4
+ MAX_DESCRIPTION_LEN //description
+4
+ MAX_URI_LENGTH //uri
+ 32 // mint
+ 32 // collection pub key
+ 65 // expires
+ 1  // pass state
+ 192; // Padding


/// Pack state
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum PassState {
    /// Not activated
    NotActivated,
    /// Activated
    Activated,
    /// Ended
    Ended,
}

impl Default for PassState {
    fn default() -> Self {
        Self::NotActivated
    }
}

/// Initialize a PackSet params
pub struct InitPassParams {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// URI
    pub uri: String,
    /// Pass book this bass belongs to
    pub pass_book: Pubkey, 
    /// pass expiration in unix timestamp
    pub expires_at: Option<u64>
}


/// Pack set
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema, Default)]
pub struct Pass {
    /// Account type - Pass
    pub account_type: AccountType,
    /// mint
    pub mint: Pubkey,
    /// Pass book
    pub pass_book: Pubkey,
    /// Description
    pub description: String,
    /// Link to pack set image
    pub uri: String,
    /// Name
    pub name: String, 
    /// pass expiration in unix timestamp
    pub expires_at: Option<u64>,
    /// Pass state
    pub pass_state: PassState 
}


impl Pass {
    pub const PREFIX: &'static str = "pass";
    /// Initialize a PackSet
    pub fn init(&mut self, params: InitPassParams) {
        self.account_type = AccountType::Pass;
        self.pass_book = params.pass_book;
        self.owner = params.owner;
        self.description = params.description;
        self.uri = params.uri;
        self.name = params.name;
        self.expires_at = params.expires_at;
        self.pass_state = PassState::NotActivated;
    }

    // /// Decrement supply value
    // pub fn decrement_supply(&mut self) -> Result<(), ProgramError> {
    //     if let Some(max_supply) = self.max_supply {
    //         self.max_supply = Some(max_supply.error_decrement()?);
    //     }
    //     Ok(())
    // }

    /// Check if pack is in activated state
    pub fn assert_activated(&self) -> Result<(), ProgramError> {
        if self.pass_state != PassState::Activated {
            return Err(NFTPassError::PassNotActivated.into());
        }

        Ok(())
    }


    /// fill unused bytes with zeroes
    pub fn puff_out_data_fields(&mut self) {
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_URI_LENGTH - self.uri.len() {
            array_of_zeroes.push(0u8);
        }
        self.uri = self.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

        let mut array_of_zeroes = vec![];

        while array_of_zeroes.len() < MAX_DESCRIPTION_LEN - self.description.len() {
            array_of_zeroes.push(0u8);
        }
        self.description =
            self.description.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
    }
}

impl IsInitialized for Pass {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized && self.account_type == AccountType::Pass
    }
}

impl Sealed for Pass {}

impl Pack for Pass {
    const LEN: usize = MAX_PASS_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::Pass as u8 && src[0] != AccountType::Uninitialized as u8)
            || src.len() != Self::LEN
        {
            msg!("Failed to deserialize");
            return Err(ProgramError::InvalidAccountData);
        }

        let result: Self = try_from_slice_unchecked(src)?;

        Ok(result)
    }
}
