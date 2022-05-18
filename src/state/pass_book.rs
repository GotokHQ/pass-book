//! PassBook definitions

use super::*;
use crate::{
    error::NFTPassError,
    state::{MAX_DESCRIPTION_LEN, MAX_NAME_LENGTH, MAX_URI_LENGTH, MAX_LENGTH_IMAGE_HASH},
    math::SafeMath
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
+ 1 // pass_state
+ 9 // access
+ 9 // duration
+ 9 // max_supply
+ 1 + 4 + MAX_LENGTH_IMAGE_HASH //image blur hash
+ 8 // created_at
+ 8 // price
+ 32 // price_mint
+ 33 // gatekeeper
+ 128; // Padding


/// Pass state
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
    /// The no of days this pass can be used to access the service
    pub access: Option<u64>,
    /// The no of minutes consumed for each use of this pass
    pub duration: Option<u64>,
    /// Maximum number of passes that can be minted from this pass
    pub max_supply: Option<u64>,
    /// blur has of image
    pub blur_hash: Option<String>,
    /// creation date
    pub created_at: u64,
    /// price
    pub price: u64,
    /// treasury mint
    pub price_mint: Pubkey,
    /// gate keeper mint
    pub gate_keeper: Option<Pubkey>, 
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
    /// Link to pass image
    pub uri: String,
    /// If true authority can make changes at deactivated phase
    pub mutable: bool,
    /// PassBook state
    pub pass_state: PassBookState,
    /// The no of days this pass can be used to access the service
    pub access: Option<u64>,
    /// The no of minutes consumed for each use of this pass
    pub duration: Option<u64>,
    /// Total number of passes created
    pub supply: u64,
    /// Maximum number of passes that can be minted from this pass
    pub max_supply: Option<u64>,
    /// blur hash of image
    pub blur_hash: Option<String>,
    /// creation date
    pub created_at: u64,
    /// price
    pub price: u64,
    /// price mint
    pub price_mint: Pubkey,
    /// gate_keeper that must sign the transaction to buy or mint
    pub gate_keeper: Option<Pubkey>,
    
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
        self.pass_state = PassBookState::NotActivated;
        self.access = params.access;
        self.duration = params.duration;
        self.blur_hash = params.blur_hash;
        self.max_supply = params.max_supply;
        self.created_at = params.created_at;
        self.price = params.price;
        self.price_mint = params.price_mint;
        self.gate_keeper = params.gate_keeper;
    }

    /// Increment total passes
    pub fn increment_supply(&mut self) -> Result<(), ProgramError> {
        self.supply = self.supply.error_increment()?;
        Ok(())
    }

    /// Check if pas is in activated state
    pub fn assert_activated(&self) -> Result<(), ProgramError> {
        if self.pass_state != PassBookState::Activated {
            return Err(NFTPassError::PassNotActivated.into());
        }

        Ok(())
    }

    /// Check if pass is mutable and in a right state to edit data
    pub fn assert_able_to_edit(&self) -> Result<(), ProgramError> {
        if !self.mutable {
            return Err(NFTPassError::ImmutablePassBook.into());
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
