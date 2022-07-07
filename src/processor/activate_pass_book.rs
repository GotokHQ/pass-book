
//! ActivatePassBook instruction processing

use crate::{
    error::NFTPassError,
    state::{PassBook, PassBookState},
    utils::*,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_pack::Pack,
    pubkey::Pubkey,
};

/// Process ActivatePassBook instruction
pub fn activate_pass_book(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;

    assert_owned_by(pass_book_account, program_id)?;

    assert_signer(&authority_account)?;

    let mut pass_book = PassBook::unpack(&pass_book_account.data.borrow_mut())?;

    assert_account_key(
        authority_account,
        &&pass_book.authority,
        Some(NFTPassError::InvalidCreatorKey),
    )?;

    if pass_book.state == PassBookState::Activated {
        return Err(NFTPassError::PassBookIsAlreadyActivated.into());
    }

    pass_book.state = PassBookState::Activated;

    PassBook::pack(pass_book, *pass_book_account.data.borrow_mut())?;
    
    Ok(())
}
