//! Init pass instruction processing

use crate::{
    error::NFTPassError,
    id,
    instruction::InitPassBookArgs,
    state::{
        InitPassBook, PassBook, PassStore, MAX_DESCRIPTION_LEN, MAX_MASTER_PASS_LEN,
        MAX_NAME_LENGTH, MAX_URI_LENGTH, PREFIX, PassType
    },
    find_pass_store_program_address,
    find_pass_book_program_address,
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
    let pass_book_info = next_account_info(account_info_iter)?;
    let source_info = next_account_info(account_info_iter)?;
    let token_account_info = next_account_info(account_info_iter)?;
    let store_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let master_metadata_info = next_account_info(account_info_iter)?;
    let master_edition_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;

    assert_signer(authority_info)?;
    assert_owned_by(mint_info, &spl_token::id())?;
    assert_owned_by(master_edition_info, &mpl_token_metadata::id())?;
    assert_owned_by(master_metadata_info, &mpl_token_metadata::id())?;

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
        store_signer_seeds
    )?;

    assert_account_key(authority_info, &store.authority, Some(NFTPassError::InvalidAuthorityKey))?;

    let (master_pass_key, master_pass_bump_seed) =
    find_pass_book_program_address(program_id, mint_info.key);

    let master_pass_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.to_bytes(),
        &[master_pass_bump_seed],
    ];
    if pass_book_info.key != &master_pass_key {
        return Err(NFTPassError::InvalidPassBookKey.into());
    }
    // create and allocated pass pda account
    create_or_allocate_account_raw(
        id(),
        pass_book_info,
        rent_info,
        system_account_info,
        payer_account_info,
        MAX_MASTER_PASS_LEN,
        master_pass_signer_seeds,
    )?;

    let mut master_pass = PassBook::unpack_unchecked(&pass_book_info.data.borrow_mut())?;

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

    match args.pass_type {
        PassType::Membership => {
            if args.validity_period.is_none() || args.collection_mint.is_some() || args.time_validation_authority.is_some() {
                return Err(ProgramError::InvalidArgument);
            }
        },
        PassType::Collection => {
            if args.collection_mint.is_none() || args.validity_period.is_some() || args.time_validation_authority.is_some() {
                return Err(ProgramError::InvalidArgument);
            }  
        },
        PassType::Time => {
            if args.time_validation_authority.is_none() || args.collection_mint.is_some() || args.validity_period.is_some() {
                return Err(ProgramError::InvalidArgument);
            }          
        }
    }
    master_pass.init(InitPassBook {
        mint: *mint_info.key,
        name: args.name,
        description: args.description,
        uri: args.uri,
        authority: *authority_info.key,
        mutable: args.mutable,
        validity_period: args.validity_period,
        time_validation_authority: args.time_validation_authority,
        collection_mint: args.collection_mint,
        pass_type: args.pass_type
    });

    master_pass.puff_out_data_fields();

    let token_metadata_program_id = mpl_token_metadata::id();


    // Check for v2
    if master_edition_info.data_is_empty() {
        return Err(MetadataError::Uninitialized.into());
    }
    let master_edition = MasterEditionV2::from_account_info(master_edition_info)?;
    if master_edition.key() != Key::MasterEditionV2 {
        return Err(MetadataError::InvalidEditionKey.into());
    }
    let master_metadata = Metadata::from_account_info(master_metadata_info)?;
    assert_account_key(mint_info, &master_metadata.mint,Some(NFTPassError::InvalidMintKey))?;
    assert_derivation(
        &token_metadata_program_id,
        master_edition_info,
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            token_metadata_program_id.as_ref(),
            master_metadata.mint.as_ref(),
            EDITION.as_bytes(),
        ],
    )?;

    let source: Account = assert_initialized(source_info)?;
    if source.mint != master_metadata.mint {
        return Err(MetadataError::MintMismatch.into());
    }

    // initialize token account to hold mint tokens
    spl_initialize_account(
        token_account_info,
        mint_info,
        pass_book_info,
        rent_info,
    )?;

    // Transfer from source to token account
    spl_token_transfer(
        source_info.clone(),
        token_account_info.clone(),
        authority_info.clone(),
        1, // transfer master edition
        &[],
    )?;

    PassBook::pack(master_pass, *pass_book_info.data.borrow_mut())?;
    store.increment_pass_book_count()?;
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
