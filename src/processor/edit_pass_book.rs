//! EditPack instruction processing

use crate::{
    error::NFTPassError,
    instruction::EditPassBookArgs,
    state::{PassBook, MAX_DESCRIPTION_LEN, MAX_URI_LENGTH},
    utils::*,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};

/// Process EditPassBook instruction
pub fn edit_pass_book(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: EditPassBookArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;
    
    assert_owned_by(pass_book_account, program_id)?;
    assert_signer(&authority_account)?;

    let mint_account = next_account_info(account_info_iter).ok();

    let mut pass_book = PassBook::unpack(&pass_book_account.data.borrow_mut())?;

    assert_account_key(authority_account, &pass_book.authority, 
        Some(NFTPassError::InvalidCreatorKey),)?;

    pass_book.assert_able_to_edit()?;

    apply_changes(&mut pass_book, args, mint_account)?;

    pass_book.puff_out_data_fields();

    PassBook::pack(pass_book, *pass_book_account.data.borrow_mut())?;

    Ok(())
}

fn apply_changes(pass_book: &mut PassBook, changes: EditPassBookArgs, mint_account: Option<&AccountInfo>) -> Result<(), ProgramError> {
    if let Some(new_name) = changes.name {
        if new_name == pass_book.name {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        pass_book.name = new_name;
    }

    if let Some(description) = changes.description {
        if description == pass_book.description {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        if description.len() > MAX_DESCRIPTION_LEN {
            return Err(NFTPassError::DescriptionTooLong.into());
        }
        pass_book.description = description;
    }

    if let Some(uri) = changes.uri {
        if uri == pass_book.uri {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        if uri.len() > MAX_URI_LENGTH {
            return Err(NFTPassError::UriTooLong.into());
        }
        pass_book.uri = uri;
    }

    if let Some(new_mutable_value) = changes.mutable {
        if new_mutable_value == pass_book.mutable {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        pass_book.mutable = new_mutable_value;
    }

    if let Some(new_price) = changes.price {
        if new_price == pass_book.price {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        pass_book.price = new_price;
    }

    if let Some(new_mint_account) = mint_account {
        if *new_mint_account.key == pass_book.mint {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        let is_native = cmp_pubkeys(new_mint_account.key, &spl_token::native_mint::id());

        if !is_native {
            assert_owned_by(new_mint_account, &spl_token::id())?;
        }

        pass_book.mint = *new_mint_account.key;
    }

    Ok(())
}
