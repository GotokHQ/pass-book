//! DeletePack instruction processing

use crate::{
    error::NFTPassError,
    state::{PassBook},
    utils::*,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_pack::Pack,
    pubkey::Pubkey,
};

/// Process DeletePassBook instruction
pub fn delete_pass_book(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    let refunder_account = next_account_info(account_info_iter)?;

    assert_owned_by(pass_book_account, program_id)?;

    assert_signer(&authority_account)?;

    let pass_book = PassBook::unpack(&pass_book_account.data.borrow_mut())?;

    assert_account_key(
        authority_account,
        &pass_book.authority,
        Some(NFTPassError::InvalidCreatorKey),
    )?;

    // Transfer PackCard tokens
    empty_account_balance(pass_book_account, refunder_account)?;
    
    Ok(())
}
