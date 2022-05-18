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
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: EditPassBookArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_account = next_account_info(account_info_iter)?;
    let authority_account = next_account_info(account_info_iter)?;

    assert_signer(&authority_account)?;

    let price_mint_account = next_account_info(account_info_iter).ok();

    let mut pass_book = PassBook::unpack(&pass_book_account.data.borrow_mut())?;

    assert_account_key(authority_account, &pass_book.authority, 
        Some(NFTPassError::InvalidAuthorityKey),)?;

    pass_book.assert_able_to_edit()?;

    apply_changes(&mut pass_book, args, price_mint_account)?;

    pass_book.puff_out_data_fields();

    PassBook::pack(pass_book, *pass_book_account.data.borrow_mut())?;

    Ok(())
}

fn apply_changes(pass_book: &mut PassBook, changes: EditPassBookArgs, price_mint_account: Option<&AccountInfo>) -> Result<(), ProgramError> {
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

    if let Some(new_blur_hash) = changes.blur_hash {
        if let Some(old_hash) = &pass_book.blur_hash {
            if new_blur_hash == *old_hash {
                return Err(NFTPassError::CantSetTheSameValue.into());
            }
        };
        pass_book.blur_hash = Some(new_blur_hash);
    }

    if let Some(new_price_mint_account) = price_mint_account {
        if *new_price_mint_account.key == pass_book.price_mint {
            return Err(NFTPassError::CantSetTheSameValue.into());
        }
        let is_native = *new_price_mint_account.key == solana_program::system_program::ID;

        if !is_native {
            assert_owned_by(new_price_mint_account, &spl_token::id())?;
        }

        pass_book.price_mint = *new_price_mint_account.key;
    }

    Ok(())
}
