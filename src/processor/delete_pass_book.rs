//! DeletePack instruction processing

use crate::{
    error::NFTPassError,
    find_pass_book_program_address,
    state::{PassBook, PREFIX},
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
    let new_master_edition_owner_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;

    assert_owned_by(pass_book_account, program_id)?;

    assert_signer(&authority_account)?;

    let pass_book = PassBook::unpack(&pass_book_account.data.borrow_mut())?;

    assert_account_key(
        authority_account,
        &pass_book.authority,
        Some(NFTPassError::InvalidAuthorityKey),
    )?;

    assert_account_key(
        token_account,
        &pass_book.token,
        Some(NFTPassError::InvalidTokenAccountKey),
    )?;

    let (pass_book_key, pass_book_bump_seed) =
        find_pass_book_program_address(program_id, &pass_book.mint);

    let pass_book_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &pass_book.mint.to_bytes(),
        &[pass_book_bump_seed],
    ];

    if pass_book_account.key != &pass_book_key {
        return Err(NFTPassError::InvalidPassBookKey.into());
    }

    // Obtain PassBook token account instance
    let pass_book_token_account = spl_token::state::Account::unpack(&token_account.data.borrow())?;

    // Transfer PackCard tokens
    spl_token_transfer(
        token_account.clone(),
        new_master_edition_owner_account.clone(),
        pass_book_account.clone(),
        pass_book_token_account.amount,
        &[pass_book_signer_seeds],
    )?;

    empty_account_balance(pass_book_account, refunder_account)?;
    
    Ok(())
}
