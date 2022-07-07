use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use super::{AccountType, USE_AUTHORITY_LENGTH};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, BorshSchema)]
pub struct Uses {
    // 16 bytes + Option byte
    pub remaining: u64, //8
    pub total: u64,     //8
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone, BorshSchema)]
pub struct UseAuthority {
    pub account_type: AccountType, //1
    pub membership: Pubkey,        // 32
    pub allowed_uses: u64,         //8
    pub bump: u8,                  //1
}

impl UseAuthority {
    pub const PREFIX: &'static str = "user";
    /// Initialize a use authority
    pub fn init(&mut self, membership: Pubkey, bump: u8) {
        self.account_type = AccountType::UseAuthority;
        self.membership = membership;
        self.bump = bump;
    }
}

impl IsInitialized for UseAuthority {
    fn is_initialized(&self) -> bool {
        self.account_type != AccountType::Uninitialized
            && self.account_type == AccountType::UseAuthority
    }
}

impl Sealed for UseAuthority {}

impl Pack for UseAuthority {
    const LEN: usize = USE_AUTHORITY_LENGTH;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if (src[0] != AccountType::UseAuthority as u8
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
