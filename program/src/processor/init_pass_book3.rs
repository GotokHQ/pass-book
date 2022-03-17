//! Init pass instruction processing

use crate::{
    error::NFTPassError,
    find_pass_book_program_address, find_pass_store_program_address, id,
    instruction::InitPassBookArgs,
    state::{
        InitPassBook, PassBook, PassStore, MAX_DESCRIPTION_LEN, MAX_MASTER_PASS_LEN,
        MAX_NAME_LENGTH, MAX_URI_LENGTH, PREFIX,
    },
    utils::*,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use mpl_token_metadata::{
    error::MetadataError,
    state::{Key, MasterEdition, MasterEditionV2, Metadata, EDITION},
    utils::{assert_derivation, assert_initialized},
};

use spl_token::state::Account;

/// Process InitPass instruction
pub fn init_pass_book(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: InitPassBookArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let pass_book_account = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    if account_info_iter.len() == 10 {
        let token_metadata_program_id = mpl_token_metadata::id();
        let collection_mint_info = next_account_info(account_info_iter)?;
        let collection_master_metadata_info = next_account_info(account_info_iter)?;
        let collection_master_edition_info = next_account_info(account_info_iter)?;
        assert_owned_by(collection_mint_info, &spl_token::id())?;
        assert_owned_by(collection_master_edition_info, &mpl_token_metadata::id())?;
        assert_owned_by(collection_master_metadata_info, &mpl_token_metadata::id())?;

        // Check for v2
        if collection_master_edition_info.data_is_empty() {
            return Err(MetadataError::Uninitialized.into());
        }
        let master_edition = MasterEditionV2::from_account_info(collection_master_edition_info)?;
        if master_edition.key() != Key::MasterEditionV2 {
            return Err(MetadataError::InvalidEditionKey.into());
        }
        let master_metadata = Metadata::from_account_info(collection_master_metadata_info)?;
        assert_account_key(
            collection_mint_info,
            &master_metadata.mint,
            Some(NFTPassError::InvalidMintKey),
        )?;
        assert_derivation(
            &token_metadata_program_id,
            collection_master_edition_info,
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                token_metadata_program_id.as_ref(),
                master_metadata.mint.as_ref(),
                EDITION.as_bytes(),
            ],
        )?;
    }

    assert_signer(authority_info)?;

    let (store_key, store_bump_seed) =
        find_pass_store_program_address(program_id, authority_info.key);
    assert_account_key(store_info, &store_key, Some(NFTPassError::InvalidStoreKey))?;

    let store_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &authority_info.key.to_bytes(),
        PassStore::PREFIX.as_bytes(),
        &[store_bump_seed],
    ];

    let mut store: PassStore = get_pass_store_data(
        program_id,
        store_info,
        authority_info,
        payer_account_info,
        rent_info,
        system_account_info,
        store_signer_seeds,
    )?;

    assert_account_key(
        authority_info,
        &store.authority,
        Some(NFTPassError::InvalidAuthorityKey),
    )?;

    let (master_pass_key, master_pass_bump_seed) =
        find_pass_book_program_address(program_id, collection_mint_info.key);

    let master_pass_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &collection_mint_info.key.to_bytes(),
        &[master_pass_bump_seed],
    ];
    if pass_book_account.key != &master_pass_key {
        return Err(NFTPassError::InvalidPassBookKey.into());
    }
    // create and allocated pass pda account
    create_or_allocate_account_raw(
        id(),
        pass_book_account,
        rent_info,
        system_account_info,
        payer_account_info,
        MAX_MASTER_PASS_LEN,
        master_pass_signer_seeds,
    )?;

    let mut master_pass = PassBook::unpack_unchecked(&pass_book_account.data.borrow_mut())?;

    if master_pass.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if args.name.len() > MAX_NAME_LENGTH {
        return Err(NFTPassError::NameTooLong.into());
    }

    if args.uri.len() > MAX_URI_LENGTH {
        return Err(NFTPassError::UriTooLong.into());
    }

    if args.description.len() > MAX_DESCRIPTION_LEN {
        return Err(NFTPassError::DescriptionTooLong.into());
    }

    master_pass.init(InitPassBook {
        collection_mint: *collection_mint_info.key,
        name: args.name,
        description: args.description,
        uri: args.uri,
        authority: *authority_info.key,
        mutable: args.mutable,
        period: args.period,
    });

    master_pass.puff_out_data_fields();

    PassBook::pack(master_pass, *pass_book_account.data.borrow_mut())?;
    store.increment_pass_collection_count()?;
    PassStore::pack(store, *store_info.data.borrow_mut())?;
    Ok(())
}

pub fn get_pass_store_data<'a>(
    program_id: &Pubkey,
    store_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    signers_seeds: &[&[u8]],
) -> Result<PassStore, ProgramError> {
    // set up pass store account

    let unpack = PassStore::unpack(&store_info.data.borrow_mut());

    let proving_process = match unpack {
        Ok(data) => Ok(data),
        Err(_) => {
            // create pass store account
            create_or_allocate_account_raw(
                *program_id,
                store_info,
                rent_sysvar_info,
                system_program_info,
                payer_info,
                PassStore::LEN,
                signers_seeds,
            )?;

            msg!("New pass store account was created");

            let mut data = PassStore::unpack_unchecked(&store_info.data.borrow_mut())?;

            data.init(*authority_info.key);
            Ok(data)
        }
    };

    proving_process
}
