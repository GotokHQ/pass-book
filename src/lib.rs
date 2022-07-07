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
use state::{
    Membership, Payout, Store, StoreAuthority, TradeHistory, UseAuthority, COLLECTION_MINT, PREFIX,
};

solana_program::declare_id!("passjvPvHQWN4SvBCmHk1gdrtBvoHRERtQK9MKemreQ");

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

/// Generates payout account address
pub fn find_payout_program_address(
    program_id: &Pubkey,
    authority: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &authority.to_bytes(),
            &mint.to_bytes(),
            Payout::PREFIX.as_bytes(),
        ],
        program_id,
    )
}

/// Generates trade history account address
pub fn find_trade_history_program_address(
    program_id: &Pubkey,
    passbook: &Pubkey,
    wallet: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &passbook.to_bytes(),
            &wallet.to_bytes(),
            TradeHistory::PREFIX.as_bytes(),
        ],
        program_id,
    )
}

/// Generates master pass address
pub fn find_pass_book_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PREFIX.as_bytes(), program_id.as_ref(), &mint.to_bytes()],
        program_id,
    )
}

/// Generate membership pda
pub fn find_membership_program_address(
    program_id: &Pubkey,
    store: &Pubkey,
    wallet: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &store.to_bytes(),
            &wallet.to_bytes(),
            Membership::PREFIX.as_bytes(),
        ],
        program_id,
    )
}

/// Generate use authority pda
pub fn find_use_authority_program_address(
    program_id: &Pubkey,
    membership: &Pubkey,
    user: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &membership.to_bytes(),
            &user.to_bytes(),
            UseAuthority::PREFIX.as_bytes(),
        ],
        program_id,
    )
}

/// Generate store authority pda
pub fn find_store_authority_program_address(
    program_id: &Pubkey,
    store: &Pubkey,
    user: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &store.to_bytes(),
            &user.to_bytes(),
            StoreAuthority::PREFIX.as_bytes(),
        ],
        program_id,
    )
}
