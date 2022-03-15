pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;

mod math;

/// Current program version
pub const PROGRAM_VERSION: u8 = 1;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;


// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
use solana_program::pubkey::Pubkey;
use state::{Store, COLLECTION_MINT, PREFIX};


solana_program::declare_id!("passK9sjcBkUzWu35gf2x4EmpcrkZB9NXgHWtgAzxhB");

/// Generates pass collection mint
pub fn find_pass_collection_mint(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            COLLECTION_MINT.as_bytes(),
        ],
        program_id,
    )
}

/// Generates program authority
pub fn find_program_authority(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PREFIX.as_bytes(), program_id.as_ref()], program_id)
}

/// Generates pass store address
pub fn find_pass_store_program_address(program_id: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &authority.to_bytes(),
            Store::PREFIX.as_bytes(),
        ],
        program_id,
    )
}

/// Generates master pass address
pub fn find_master_pass_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &mint.to_bytes()
        ],
        program_id,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
