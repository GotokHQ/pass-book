//! Init pass instruction processing

use crate::{
    error::NFTPassError,
    find_pass_book_program_address, find_pass_store_program_address, find_payout_program_address,
    id,
    instruction::InitPassBookArgs,
    math::SafeMath,
    state::{
        InitPassBook, PassBook, PassStore, Payout, MAX_DESCRIPTION_LEN, MAX_NAME_LENGTH,
        MAX_PASS_BOOK_LEN, MAX_URI_LENGTH, PREFIX,
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
    system_program::ID as system_id,
    sysvar::{clock::Clock, Sysvar},
};

use mpl_token_metadata::{
    error::MetadataError,
    state::{Key, MasterEdition, MasterEditionV2, Metadata, EDITION},
    utils::{assert_derivation, assert_initialized},
};

use std::slice::Iter;

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
    let price_mint_info = next_account_info(account_info_iter)?;
    let clock_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let system_account_info = next_account_info(account_info_iter)?;
    let clock = &Clock::from_account_info(clock_info)?;
    let _ = next_account_info(account_info_iter)?; //token program

    assert_signer(authority_info)?;

    assert_owned_by(mint_info, &spl_token::id())?;
    assert_owned_by(source_info, &spl_token::id())?;
    assert_owned_by(token_account_info, &spl_token::id())?;
    assert_owned_by(master_edition_info, &mpl_token_metadata::id())?;
    assert_owned_by(master_metadata_info, &mpl_token_metadata::id())?;

    let is_native = *price_mint_info.key == system_id;

    if !is_native {
        assert_owned_by(price_mint_info, &spl_token::id())?;
    }

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

    let (pass_book_key, pass_book_bump_seed) =
        find_pass_book_program_address(program_id, mint_info.key);

    let pass_book_signer_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &mint_info.key.to_bytes(),
        &[pass_book_bump_seed],
    ];
    if pass_book_info.key != &pass_book_key {
        return Err(NFTPassError::InvalidPassBookKey.into());
    }
    // create and allocated pass pda account
    create_or_allocate_account_raw(
        id(),
        pass_book_info,
        rent_info,
        system_account_info,
        payer_account_info,
        MAX_PASS_BOOK_LEN,
        pass_book_signer_seeds,
    )?;

    let mut pass_book = PassBook::unpack_unchecked(&pass_book_info.data.borrow_mut())?;

    if pass_book.is_initialized() {
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

    if let Some(duration) = args.duration {
        if duration == 0 {
            return Err(NFTPassError::WrongDuration.into());
        }
    }

    if let Some(validity) = args.access {
        if validity == 0 {
            return Err(NFTPassError::WrongValidityPeriod.into());
        }
    }

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
    assert_account_key(
        mint_info,
        &master_metadata.mint,
        Some(NFTPassError::InvalidMintKey),
    )?;
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
    spl_initialize_account(token_account_info, mint_info, pass_book_info, rent_info)?;

    // Transfer from source to token account
    spl_token_transfer(
        source_info.clone(),
        token_account_info.clone(),
        authority_info.clone(),
        1, // transfer master edition
        &[],
    )?;

    match args.max_supply {
        Some(max_supply) => {
            if max_supply == 0 {
                return Err(NFTPassError::WrongMaxSupply.into());
            }
            if let Some(m_e_max_supply) = master_edition.max_supply() {
                if (max_supply as u64) > m_e_max_supply.error_sub(master_edition.supply())?
                    || max_supply == 0
                {
                    return Err(NFTPassError::WrongMaxSupply.into());
                }
            }
        }
        _ => {
            if master_edition.max_supply().is_some() {
                return Err(NFTPassError::WrongMasterSupply.into());
            }
        }
    }

    create_payout_account(
        program_id,
        &master_metadata,
        account_info_iter,
        payer_account_info,
        rent_info,
        system_account_info,
        price_mint_info,
    )?;

    let gate_keeper_account = next_account_info(account_info_iter).ok();

    let gate_keeper = if let Some(gate_keeper) = gate_keeper_account {
        assert_signer(gate_keeper)?;
        Some(*gate_keeper.key)
    } else {
        None
    };

    pass_book.init(InitPassBook {
        mint: *mint_info.key,
        name: args.name,
        description: args.description,
        uri: args.uri,
        authority: *authority_info.key,
        mutable: args.mutable,
        duration: args.duration,
        access: args.access,
        max_supply: args.max_supply,
        blur_hash: args.blur_hash,
        created_at: clock.unix_timestamp as u64,
        price: args.price,
        price_mint: *price_mint_info.key,
        gate_keeper: gate_keeper,
    });

    pass_book.puff_out_data_fields();

    PassBook::pack(pass_book, *pass_book_info.data.borrow_mut())?;
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

pub fn create_payout_account<'a>(
    program_id: &Pubkey,
    metadata: &Metadata,
    remaining_accounts: &mut Iter<AccountInfo<'a>>,
    payer_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    price_mint_info: &AccountInfo<'a>,
) -> ProgramResult {
    // set up creator payout account
    match &metadata.data.creators {
        Some(creators) => {
            for creator in creators {
                let current_creator_payout_info = next_account_info(remaining_accounts)?;
                let treasury_holder_info = next_account_info(remaining_accounts)?;
                let (payout_key, payout_bump_seed) =
                    find_payout_program_address(program_id, &creator.address, price_mint_info.key);
                assert_account_key(
                    current_creator_payout_info,
                    &payout_key,
                    Some(NFTPassError::InvalidPayoutKey),
                )?;

                let payout_signer_seeds = &[
                    PREFIX.as_bytes(),
                    program_id.as_ref(),
                    &creator.address.to_bytes(),
                    &price_mint_info.key.to_bytes(),
                    Payout::PREFIX.as_bytes(),
                    &[payout_bump_seed],
                ];

                let unpack = Payout::unpack(&current_creator_payout_info.data.borrow_mut());
                let result: Result<(), ProgramError> = match unpack {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        let is_native = *price_mint_info.key == system_id;

                        if is_native {
                            if treasury_holder_info.key != current_creator_payout_info.key {
                                return Err(ProgramError::InvalidAccountData);
                            }
                        } else {
                            assert_owned_by(treasury_holder_info, &spl_token::id())?;
                            spl_initialize_account(
                                treasury_holder_info,
                                price_mint_info,
                                current_creator_payout_info,
                                rent_sysvar_info,
                            )?;
                        }
                        let mut data = current_creator_payout_info.data.borrow_mut();
                        if data.len() == 0 {
                            // create payout account
                            create_or_allocate_account_raw(
                                *program_id,
                                current_creator_payout_info,
                                rent_sysvar_info,
                                system_program_info,
                                payer_info,
                                Payout::LEN,
                                payout_signer_seeds,
                            )?;
                        }

                        let mut payout = Payout::unpack_unchecked(
                            &data,
                        )?;

                        payout.init(
                            *current_creator_payout_info.key,
                            *price_mint_info.key,
                            *treasury_holder_info.key,
                        );
                        Payout::pack(payout, *data)?;
                        Ok(())
                    }
                };
                result.unwrap()
            }
            Ok(())
        }
        None => Ok(()),
    }
}
