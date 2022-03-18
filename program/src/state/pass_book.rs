//! PassBook definitions

use super::*;
use crate::{
    error::NFTPassError,
    state::{MAX_DESCRIPTION_LEN, MAX_NAME_LENGTH, MAX_URI_LENGTH},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

pub const MAX_PASS_BOOK_LEN: usize = 1 //account type 
+4
+ MAX_NAME_LENGTH // name
+4
+ MAX_DESCRIPTION_LEN //description
+4
+ MAX_URI_LENGTH //uri
+ 32 // authority pub key
+ 32 // mint pub key
+ 1 // mutable
+ 1 // period
+ 1 // pass_state
+ 128; // Padding

/// Distribution type
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum PassType {
    /// A membership pass requires setting validity period in days
    Membership,
    /// A collection pass requires holding NFT belonging to a collection mint 
    Collection,
    /// A time pass requires a duration of time this pass is valid for and signed by a timestamp authority
    Time,
}

/// Pack state
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum PassBookState {
    /// Not activated
    NotActivated,
    /// Activated
    Activated,
    /// Deactivated
    Deactivated,
    /// Ended
    Ended,
}

impl Default for PassBookState {
    fn default() -> Self {
        Self::NotActivated
    }
}

/// Initialize a PackSet params
pub struct InitPassBook {
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// URI
    pub uri: String,
    /// PassBook authority
    pub authority: Pubkey,
    /// PassBook mint
    pub mint: Pubkey,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,
    /// Validity period in days
    pub validity_period: Option<u32>,
    /// Collection mint
    pub collection_mint: Option<Pubkey>,
    /// Time validation authority
    pub time_validation_authority: Option<Pubkey>,
    /// Pass type
    pub pass_type: PassType,
}

/// Pack set
#[repr(C)]
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct PassBook {
    /// Account type - PassBook
    pub account_type: AccountType,
    /// PassBook authority
    pub authority: Pubkey,
    /// PassBook mint
    pub mint: Pubkey,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Link to pack set image
    pub uri: String,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,
    /// Pass type
    pub pass_type: PassType,
    /// Validity period in days
    pub validity_period: Option<u32>,
    /// Collection mint
    pub collection_mint: Option<Pubkey>,
    /// Time validation authority
    pub time_validation_authority: Option<Pubkey>,
    /// PassBook state
    pub pass_state: PassBookState,
}

impl PassBook {
    /// Initialize a PackSet
    pub fn init(&mut self, params: InitPassBook) {
        self.account_type = AccountType::PassBook;
        self.authority = params.authority;
        self.description = params.description;
        self.uri = params.uri;
        self.name = params.name;
        self.mint = params.mint;
        self.mutable = params.mutable;
        self.pass_type = params.pass_type;
        self.validity_period = params.validity_period;
        self.collection_mint = params.collection_mint;
        self.time_validation_authority = params.time_validation_authority;
        self.pass_state = PassBookState::NotActivated;
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
        if self.pass_state != PassBookState::Activated {
            return Err(NFTPassError::PassNotActivated.into());
        }

        Ok(())
    }

    /// Check if pack is mutable and in a right state to edit data
    pub fn assert_able_to_edit(&self) -> Result<(), ProgramError> {
        if !self.mutable {
            return Err(NFTPassError::ImmutablePass.into());
        }

        if self.pass_state == PassBookState::Activated {
            return Err(NFTPassError::WrongPassState.into());
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

impl IsInitialized for PassBook {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::PassBook
    }
}

impl Sealed for PassBook {}

impl Pack for PassBook {
    const LEN: usize = MAX_PASS_BOOK_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::PassBook as u8
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
